#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![warn(clippy::pedantic, missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
pub use error::Error;

#[cfg(test)]
mod test_util;

mod traits;
pub use traits::{Decode, DecodeIter, Encode};

mod dtc;
pub use dtc::{
    CLEAR_ALL_DTCS, DTCExtDataRecordNumber, DTCFormatIdentifier, DTCRecord, DTCSeverityMask,
    DTCSeverityRecord, DTCSnapshotRecordNumber, DTCStatusMask, DTCStoredDataRecordNumber,
    FunctionalGroupIdentifier,
};

mod shared;
pub use shared::{
    NegativeResponseCode, UDSIdentifier, UDSRoutineIdentifier, param_length_u16, param_length_u32,
    param_length_u64, param_length_u128,
};

mod request;
pub use request::Request;

mod response;
pub use response::Response;

mod service;
pub use service::UdsServiceType;

mod services;
pub use services::{
    ClearDiagnosticInfoRequest, CommunicationControlRequest, CommunicationControlResponse,
    CommunicationControlType, CommunicationType, ControlDTCSettingsRequest,
    ControlDTCSettingsResponse, DiagnosticSessionControlRequest, DiagnosticSessionControlResponse,
    DiagnosticSessionType, DirSizePayload, DtcAndStatusIter, DtcFaultDetectionIter, DtcSettings,
    DtcSeverityAndStatusIter, EcuResetRequest, EcuResetResponse, FileOperationMode,
    FileSizePayload, NamePayload, NegativeResponse, PositionPayload, ReadDTCInfoRequest,
    ReadDTCInfoResponse, ReadDTCInfoSubFunction, ReadDataByIdentifierRequest,
    RequestDownloadRequest, RequestDownloadResponse, RequestFileTransferRequest,
    RequestFileTransferResponse, RequestTransferExitRequest, RequestTransferExitResponse,
    ResetType, RoutineControlRequest, RoutineControlResponse, RoutineControlSubFunction,
    SecurityAccessRequest, SecurityAccessResponse, SecurityAccessType, SentDataPayload,
    SizePayload, TesterPresentRequest, TesterPresentResponse, TransferDataRequest,
    TransferDataResponse, WriteDataByIdentifierRequest, WriteDataByIdentifierResponse,
};

#[cfg(test)]
mod no_std_api_tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

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
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
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

    #[cfg(feature = "alloc")]
    #[test]
    fn dtc_and_status_iter_roundtrip() {
        // 2 DTC records: (0x01,0x02,0x03, status=0x0A), (0x04,0x05,0x06, status=0x0B)
        let data = [0x01, 0x02, 0x03, 0x0A, 0x04, 0x05, 0x06, 0x0B];
        let iter = DtcAndStatusIter::new(&data);
        assert_eq!(iter.len(), 2);

        let records: Vec<_> = iter.map(|r| r.unwrap()).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(u32::from(records[0].0), 0x010203);
        assert_eq!(u32::from(records[1].0), 0x040506);
    }

    #[test]
    fn request_frame_roundtrip_prepends_sid() {
        // EcuReset request: SID=0x11, sub=0x01
        let wire = [0x11, 0x01];
        let (req, _) = Request::decode(&wire).unwrap();
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
        assert_eq!(written, req.encoded_size());
    }

    #[test]
    fn response_frame_roundtrip_prepends_sid() {
        // NegativeResponse: SID=0x7F, service=0x10, NRC=0x12
        let wire = [0x7F, 0x10, 0x12];
        let (resp, _) = Response::decode(&wire).unwrap();
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
        assert_eq!(written, resp.encoded_size());
    }

    #[test]
    fn request_file_transfer_frame_roundtrip() {
        // RequestFileTransfer: SID=0x38, DeleteFile(0x02), name_len=0x0003, "abc"
        let wire = [0x38, 0x02, 0x00, 0x03, b'a', b'b', b'c'];
        let (req, _) = Request::decode(&wire).unwrap();
        assert_eq!(req.service(), UdsServiceType::RequestFileTransfer);
        let mut buf = [0u8; 16];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn read_dtc_info_response_frame_roundtrip() {
        // ReadDTCInfo response: SID=0x59, sub=0x02, mask=0xFF, then DTC records
        let wire = [0x59, 0x02, 0xFF, 0x01, 0x02, 0x03, 0x0A];
        let (resp, _) = Response::decode(&wire).unwrap();
        let mut buf = [0u8; 16];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn read_dtc_info_request_encodes_through_public_api() {
        // Public-surface construction: types reached via crate root, not shared::/services::.
        let req = ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportDTC_ByStatusMask(
            DTCStatusMask::from(0xFF),
        ));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        // sub=0x02 ReportDTC_ByStatusMask, mask=0xFF
        assert_eq!(&buf[..written], &[0x02, 0xFF]);
        assert_eq!(written, req.encoded_size());
    }

    #[test]
    fn write_data_by_identifier_response_roundtrips_through_public_api() {
        // Reachability check: the WDBI response codec works through the crate-root public API.
        let resp = WriteDataByIdentifierResponse::new(0xBEEF);
        let mut buf = [0u8; 4];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        let (decoded, remainder) =
            <WriteDataByIdentifierResponse as Decode>::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, resp);
        assert!(remainder.is_empty());
    }

    #[test]
    fn const_construction() {
        // Verify const construction works at compile time
        const _REQ: TransferDataRequest<'static> = TransferDataRequest::new(1, &[0x01, 0x02, 0x03]);
        const _SEC: SecurityAccessRequest<'static> =
            SecurityAccessRequest::new(false, SecurityAccessType::RequestSeed(0x01), &[0xAA, 0xBB]);
    }
}
