#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![warn(clippy::pedantic, missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
pub use automotive_wire_codec::{Incomplete, InvalidWidth, TrailingBytes};
pub use error::Error;

#[cfg(test)]
mod test_util;

pub use automotive_wire_codec::{Decode, DecodeIter, Encode};

mod dtc;
pub use dtc::{
    CLEAR_ALL_DTCS, DtcExtDataRecordNumber, DtcFormatIdentifier, DtcRecord, DtcSeverityMask,
    DtcSnapshotRecordNumber, DtcStatusMask, DtcStoredDataRecordNumber, FunctionalGroupIdentifier,
};

mod shared;
pub use shared::{DataFormatIdentifier, NegativeResponseCode, UdsIdentifier, UdsRoutineIdentifier};

mod request;
pub use request::Request;

mod response;
pub use response::Response;

mod service;
pub use service::UdsServiceType;

mod services;
pub use services::{
    ClearDiagnosticInfoRequest, ClearDiagnosticInfoResponse, CommunicationControlRequest,
    CommunicationControlResponse, CommunicationControlType, CommunicationType,
    ControlDtcSettingRequest, ControlDtcSettingResponse, DiagnosticSessionControlRequest,
    DiagnosticSessionControlResponse, DiagnosticSessionType, DirSizePayload, DtcAndStatusIter,
    DtcFaultDetectionCounterRecord, DtcFaultDetectionIter, DtcSettingType, EcuResetRequest,
    EcuResetResponse, FileOperationMode, FileSizePayload, NamePayload, NegativeResponse,
    PositionPayload, ReadDataByIdentifierRequest, ReadDataByIdentifierResponse, ReadDtcInfoRequest,
    ReadDtcInfoResponse, ReadDtcInfoSubFunction, RequestDownloadRequest, RequestDownloadResponse,
    RequestFileTransferRequest, RequestFileTransferResponse, RequestTransferExitRequest,
    RequestTransferExitResponse, RequestUploadRequest, RequestUploadResponse, ResetType,
    RoutineControlRequest, RoutineControlResponse, RoutineControlSubFunction, SecurityAccessLevel,
    SecurityAccessRequest, SecurityAccessResponse, SecurityAccessType, SentDataPayload,
    SizePayload, SubnetNumber, TesterPresentRequest, TesterPresentResponse, TransferDataRequest,
    TransferDataResponse, WriteDataByIdentifierRequest, WriteDataByIdentifierResponse,
    WwhObdDtcSeverityIter,
};

#[cfg(test)]
mod no_std_api_tests {
    use super::*;

    #[test]
    fn encode_decode_tester_present_roundtrip() {
        let req = TesterPresentRequest::new(false);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);

        let (decoded, rest) = <TesterPresentRequest as Decode>::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, req);
        assert!(rest.is_empty());
    }

    #[test]
    fn encode_decode_transfer_data_tx_roundtrip() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let req = TransferDataRequest::new(0x05, &data);
        let mut buf = [0u8; 16];
        let written = req.encode_to_slice(&mut buf).unwrap();
        assert_eq!(written, 5);

        let (decoded, _) = <TransferDataRequest as Decode>::decode(&buf[..written]).unwrap();
        assert_eq!(decoded.block_sequence_counter, 0x05);
        assert_eq!(decoded.data, &[0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn decode_response_tester_present() {
        // TesterPresent response: SID=0x7E, sub=0x00
        let wire = [0x7E, 0x00];
        let (resp, _) = Response::decode(&wire).unwrap();
        assert!(matches!(resp, Response::TesterPresent(_)));
    }

    #[test]
    fn decode_response_negative() {
        // NegativeResponse: SID=0x7F, service=0x10, NRC=0x12
        let wire = [0x7F, 0x10, 0x12];
        let (resp, _) = Response::decode(&wire).unwrap();
        assert!(matches!(resp, Response::NegativeResponse(_)));
    }

    #[test]
    fn decode_request_ecu_reset() {
        // EcuReset request: SID=0x11, sub=0x01 (HardReset)
        let wire = [0x11, 0x01];
        let (req, _) = Request::decode(&wire).unwrap();
        assert!(matches!(req, Request::EcuReset(_)));
        assert_eq!(req.service(), UdsServiceType::EcuReset);
    }

    #[test]
    fn dtc_and_status_iter_roundtrip() {
        // 2 DTC records: (0x01,0x02,0x03, status=0x0A), (0x04,0x05,0x06, status=0x0B)
        //
        // Deliberately not gated on `alloc`, and written with `next()` rather than `collect()`
        // for that reason. While this was gated, mutating the status byte to a constant left
        // `--no-default-features` fully green — so the assertion below existed but did not
        // protect the config the crate targets first.
        let data = [0x01, 0x02, 0x03, 0x0A, 0x04, 0x05, 0x06, 0x0B];
        let mut iter = DtcAndStatusIter::new(&data);
        assert_eq!(iter.len(), 2);

        let (dtc, status) = iter.next().unwrap().unwrap();
        assert_eq!(u32::from(dtc), 0x01_0203);
        // The status byte matters as much as the DTC: 0x02 and 0x0A-0x0E are the most-used
        // sub-functions, and nothing asserted it, so "every DTC reports status 0x00" passed.
        assert_eq!(status.bits(), 0x0A);

        let (dtc, status) = iter.next().unwrap().unwrap();
        assert_eq!(u32::from(dtc), 0x04_0506);
        assert_eq!(status.bits(), 0x0B);

        assert!(iter.next().is_none());
    }

    #[test]
    fn fault_detection_counter_record_is_nameable_from_crate_root() {
        // `DtcFaultDetectionCounterRecord` is the `Item` of `DtcFaultDetectionIter`. Without a
        // crate-root path, callers can iterate but cannot name the type — no `Vec<T>`, no struct
        // field, no helper signature. This pins the re-export.
        let data = [0x01, 0x02, 0x03, 0x2A];
        let record: DtcFaultDetectionCounterRecord =
            DtcFaultDetectionIter::new(&data).next().unwrap().unwrap();
        assert_eq!(u32::from(record.dtc_record), 0x01_0203);
        assert_eq!(record.dtc_fault_detection_counter, 0x2A);
    }

    #[test]
    fn request_frame_roundtrip_prepends_sid() {
        // EcuReset request: SID=0x11, sub=0x01
        let wire = [0x11, 0x01];
        let (req, _) = Request::decode(&wire).unwrap();
        let mut buf = [0u8; 8];
        let written = req.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &wire);
        assert_eq!(written, req.encoded_size().unwrap());
    }

    #[test]
    fn response_frame_roundtrip_prepends_sid() {
        // NegativeResponse: SID=0x7F, service=0x10, NRC=0x12
        let wire = [0x7F, 0x10, 0x12];
        let (resp, _) = Response::decode(&wire).unwrap();
        let mut buf = [0u8; 8];
        let written = resp.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &wire);
        assert_eq!(written, resp.encoded_size().unwrap());
    }

    #[test]
    fn request_file_transfer_frame_roundtrip() {
        // RequestFileTransfer: SID=0x38, DeleteFile(0x02), name_len=0x0003, "abc"
        let wire = [0x38, 0x02, 0x00, 0x03, b'a', b'b', b'c'];
        let (req, _) = Request::decode(&wire).unwrap();
        assert_eq!(req.service(), UdsServiceType::RequestFileTransfer);
        let mut buf = [0u8; 16];
        let written = req.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn request_upload_frames_roundtrip() {
        // RequestUpload request: SID=0x35, DFI=0x00, ALFID=0x12 (size 1 byte, addr 2 bytes),
        // addr=0xBEEF, size=0x10
        let wire = [0x35, 0x00, 0x12, 0xBE, 0xEF, 0x10];
        let (req, _) = Request::decode(&wire).unwrap();
        assert_eq!(req.service(), UdsServiceType::RequestUpload);
        assert!(matches!(req, Request::RequestUpload(_)));
        let mut buf = [0u8; 16];
        let written = req.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &wire);
        assert_eq!(written, req.encoded_size().unwrap());

        // Positive response: SID=0x75, LFID=0x20 (2-byte block length), 0x0800
        let wire = [0x75, 0x20, 0x08, 0x00];
        let (resp, _) = Response::decode(&wire).unwrap();
        assert_eq!(resp.service(), UdsServiceType::RequestUpload);
        match resp {
            Response::RequestUpload(ref up) => {
                assert_eq!(up.max_number_of_block_length(), &[0x08, 0x00]);
            }
            other => panic!("expected RequestUpload, got {other:?}"),
        }
        let written = resp.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn request_upload_is_distinct_from_request_download() {
        // Same payload, different SID: the two must not collapse into one variant.
        let payload = [0x00, 0x12, 0xBE, 0xEF, 0x10];
        let mut down = [0x34u8; 6];
        down[1..].copy_from_slice(&payload);
        let mut up = [0x35u8; 6];
        up[1..].copy_from_slice(&payload);

        let (d, _) = Request::decode(&down).unwrap();
        let (u, _) = Request::decode(&up).unwrap();
        assert!(matches!(d, Request::RequestDownload(_)));
        assert!(matches!(u, Request::RequestUpload(_)));
        assert_eq!(d.service(), UdsServiceType::RequestDownload);
        assert_eq!(u.service(), UdsServiceType::RequestUpload);

        let mut buf = [0u8; 8];
        let n = d.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..n], &down);
        let n = u.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..n], &up);
    }

    #[test]
    fn read_dtc_info_response_frame_roundtrip() {
        // ReadDtcInfo response: SID=0x59, sub=0x02, mask=0xFF, then DTC records
        let wire = [0x59, 0x02, 0xFF, 0x01, 0x02, 0x03, 0x0A];
        let (resp, _) = Response::decode(&wire).unwrap();
        let mut buf = [0u8; 16];
        let written = resp.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn read_dtc_info_request_encodes_through_public_api() {
        // Public-surface construction: types reached via crate root, not shared::/services::.
        let req = ReadDtcInfoRequest::new(
            false,
            ReadDtcInfoSubFunction::ReportDtcByStatusMask(DtcStatusMask::from(0xFF)),
        );
        let mut buf = [0u8; 8];
        let written = req.encode_to_slice(&mut buf).unwrap();
        // sub=0x02 ReportDtcByStatusMask, mask=0xFF
        assert_eq!(&buf[..written], &[0x02, 0xFF]);
        assert_eq!(written, req.encoded_size().unwrap());
    }

    #[test]
    fn write_data_by_identifier_response_roundtrips_through_public_api() {
        // Reachability check: the WDBI response codec works through the crate-root public API.
        let resp = WriteDataByIdentifierResponse::new(0xBEEF);
        let mut buf = [0u8; 4];
        let written = resp.encode_to_slice(&mut buf).unwrap();
        let (decoded, remainder) =
            <WriteDataByIdentifierResponse as Decode>::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, resp);
        assert!(remainder.is_empty());
    }

    #[test]
    fn data_format_identifier_reachable_for_request_construction() {
        // `DataFormatIdentifier` must be nameable from the crate root: it is a required
        // argument to `RequestDownloadRequest::new` and appears in `RequestFileTransfer`
        // request/response variants, so without a public path those are unconstructible.
        let dfi = DataFormatIdentifier::new(0x00, 0x00).unwrap();
        let req = RequestDownloadRequest::new(dfi, 0x1234, 0x10).unwrap();
        let mut buf = [0u8; 16];
        let written = req.encode_to_slice(&mut buf).unwrap();
        assert_eq!(written, req.encoded_size().unwrap());
    }

    #[test]
    fn const_construction() {
        // Verify const construction works at compile time
        const _REQ: TransferDataRequest<'static> = TransferDataRequest::new(1, &[0x01, 0x02, 0x03]);
        const _SEC: SecurityAccessRequest<'static> = SecurityAccessRequest::new(
            false,
            SecurityAccessType::RequestSeed(match SecurityAccessLevel::new(0x01) {
                Ok(level) => level,
                Err(_) => panic!("0x01 is a valid security access level"),
            }),
            &[0xAA, 0xBB],
        );
    }

    #[test]
    fn wire_primitives_are_const_constructible() {
        // These are the types a caller puts in a `const` table: DTC constants, record
        // numbers, and the format identifier. Each must be reachable in const context.
        const DTC: DtcRecord = DtcRecord::new(0x01, 0x02, 0x03);
        const SNAPSHOT: DtcSnapshotRecordNumber = DtcSnapshotRecordNumber::new(0x02);
        const EXT_DATA: DtcExtDataRecordNumber = DtcExtDataRecordNumber::new(0x90);
        const STORED: DtcStoredDataRecordNumber = DtcStoredDataRecordNumber::new(0x02);
        const DFI: DataFormatIdentifier = match DataFormatIdentifier::new(0x01, 0x02) {
            Ok(dfi) => dfi,
            Err(_) => panic!("both nibbles are in range"),
        };

        // Getting the byte back out must be const too, or a `const` dispatch table can be
        // built but not read. These four had only a non-const `From`, so each of these lines
        // was previously an E0015.
        const DTC_U32: u32 = DTC.to_u32();
        const DFI_BYTE: u8 = DFI.value();
        const NRC_BYTE: u8 = NegativeResponseCode::ConditionsNotCorrect.value();
        const FORMAT_BYTE: u8 = DtcFormatIdentifier::Iso14229_1DtcFormat.value();
        const CONTROL_BYTE: u8 = CommunicationControlType::DisableRxAndTx.value();

        assert_eq!(DTC_U32, 0x01_0203);
        assert_eq!(u32::from(DTC), DTC_U32);
        assert_eq!(SNAPSHOT.value(), 0x02);
        assert_eq!(EXT_DATA.value(), 0x90);
        assert_eq!(STORED.value(), 0x02);
        assert_eq!(DFI_BYTE, 0x12);
        assert_eq!(u8::from(DFI), DFI_BYTE);
        assert_eq!(NRC_BYTE, 0x22);
        assert_eq!(FORMAT_BYTE, 0x01);
        assert_eq!(CONTROL_BYTE, 0x03);
    }

    #[test]
    fn communication_control_requests_are_const_constructible() {
        // Both constructors were the crate's only non-`const` `new`s. The blocker was
        // `u8::from(control_type)` in their error payload, which an inherent `const fn value()`
        // on the enum removes.
        const REQ: CommunicationControlRequest = match CommunicationControlRequest::new(
            false,
            CommunicationControlType::DisableRxAndTx,
            CommunicationType::NormalAndNetworkManagement,
        ) {
            Ok(req) => req,
            Err(_) => panic!("DisableRxAndTx takes no node id"),
        };
        const WITH_ID: CommunicationControlRequest =
            match CommunicationControlRequest::new_with_node_id(
                false,
                CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo,
                CommunicationType::Normal,
                0x000A,
            ) {
                Ok(req) => req,
                Err(_) => panic!("the enhanced variant requires a node id"),
            };

        assert_eq!(REQ.control_type(), CommunicationControlType::DisableRxAndTx);
        assert_eq!(WITH_ID.node_id(), Some(0x000A));
    }

    #[test]
    fn sid_conversions_and_negative_responses_are_const() {
        // The SID map is the crate's most reusable lookup; a server dispatch table wants it
        // in const context. `NegativeResponse::new` builds on it, so it follows.
        const SID: u8 = UdsServiceType::EcuReset.to_request_sid();
        const SERVICE: UdsServiceType = UdsServiceType::from_request_sid(0x11);
        const RESP_SID: u8 = UdsServiceType::EcuReset.to_response_sid();
        const RESP_SERVICE: UdsServiceType = UdsServiceType::from_response_sid(0x51);
        const NACK: NegativeResponse = NegativeResponse::new(
            UdsServiceType::EcuReset,
            NegativeResponseCode::ConditionsNotCorrect,
        );

        assert_eq!(SID, 0x11);
        assert_eq!(SERVICE, UdsServiceType::EcuReset);
        assert_eq!(RESP_SID, 0x51);
        assert_eq!(RESP_SERVICE, UdsServiceType::EcuReset);
        assert_eq!(NACK.request_service_sid(), 0x11);
    }

    #[test]
    fn transfer_setup_requests_are_const_constructible() {
        const DOWNLOAD: RequestDownloadRequest =
            match RequestDownloadRequest::new(DataFormatIdentifier::NONE, 0x1234, 0x10) {
                Ok(req) => req,
                Err(_) => panic!("address fits in 5 bytes"),
            };
        const UPLOAD: RequestUploadRequest =
            match RequestUploadRequest::new(DataFormatIdentifier::NONE, 0x1234, 0x10) {
                Ok(req) => req,
                Err(_) => panic!("address fits in 5 bytes"),
            };

        assert_eq!(DOWNLOAD.memory_address(), 0x1234);
        assert_eq!(UPLOAD.memory_size(), 0x10);
    }
}
