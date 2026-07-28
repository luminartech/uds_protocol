//! Module for making and handling UDS Requests
use crate::{
    Decode, Encode, Error, Incomplete, NegativeResponseCode,
    services::{
        ClearDiagnosticInfoRequest, CommunicationControlRequest, ControlDtcSettingRequest,
        DiagnosticSessionControlRequest, EcuResetRequest, ReadDataByIdentifierRequest,
        ReadDtcInfoRequest, RequestDownloadRequest, RequestFileTransferRequest,
        RequestTransferExitRequest, RequestUploadRequest, RoutineControlRequest,
        SecurityAccessRequest, TesterPresentRequest, TransferDataRequest,
        WriteDataByIdentifierRequest,
    },
};
use automotive_wire_codec::{write_all, write_u8};

use super::service::UdsServiceType;

/// Zero-copy parsed request. Borrows from the wire buffer.
///
/// Variable-length payloads are stored as raw `&'a [u8]` slices that can be
/// further parsed on demand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Request<'a> {
    /// Clear diagnostic information request.
    ClearDiagnosticInfo(ClearDiagnosticInfoRequest),
    /// Communication control request.
    CommunicationControl(CommunicationControlRequest),
    /// Control DTC settings request.
    ControlDtcSetting(ControlDtcSettingRequest),
    /// Diagnostic session control request.
    DiagnosticSessionControl(DiagnosticSessionControlRequest),
    /// ECU reset request.
    EcuReset(EcuResetRequest),
    /// Read data by identifier request.
    ReadDataByIdentifier(ReadDataByIdentifierRequest<'a>),
    /// Read DTC information request.
    ReadDtcInfo(ReadDtcInfoRequest),
    /// Request download.
    RequestDownload(RequestDownloadRequest),
    /// Request file transfer.
    RequestFileTransfer(RequestFileTransferRequest<'a>),
    /// Request transfer exit.
    RequestTransferExit(RequestTransferExitRequest<'a>),
    /// Request upload.
    RequestUpload(RequestUploadRequest),
    /// Routine control request.
    RoutineControl(RoutineControlRequest<'a>),
    /// Security access request.
    SecurityAccess(SecurityAccessRequest<'a>),
    /// Tester present request.
    TesterPresent(TesterPresentRequest),
    /// Transfer data request.
    TransferData(TransferDataRequest<'a>),
    /// Write data by identifier request.
    WriteDataByIdentifier(WriteDataByIdentifierRequest<'a>),
    /// A known-but-unmodeled (or unrecognized) service. Carries the raw service byte and
    /// the raw payload bytes following the service identifier, for pass-through.
    ///
    /// Re-encoding is lossless for every service byte: the raw `sid` is echoed verbatim.
    Other {
        /// The raw service identifier byte from the wire.
        sid: u8,
        /// Raw payload bytes after the service byte.
        data: &'a [u8],
    },
}

impl<'a> Decode<'a> for Request<'a> {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        let service = UdsServiceType::from_request_sid(buf[0]);
        let payload = &buf[1..];

        let request = match service {
            UdsServiceType::ClearDiagnosticInfo => Self::ClearDiagnosticInfo(
                <ClearDiagnosticInfoRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::CommunicationControl => Self::CommunicationControl(
                <CommunicationControlRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::ControlDtcSetting => Self::ControlDtcSetting(
                <ControlDtcSettingRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::DiagnosticSessionControl => Self::DiagnosticSessionControl(
                <DiagnosticSessionControlRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::EcuReset => {
                Self::EcuReset(<EcuResetRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::ReadDataByIdentifier => Self::ReadDataByIdentifier(
                <ReadDataByIdentifierRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::ReadDtcInfo => {
                Self::ReadDtcInfo(<ReadDtcInfoRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(<RequestDownloadRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestFileTransfer => Self::RequestFileTransfer(
                <RequestFileTransferRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit(
                <RequestTransferExitRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::RequestUpload => {
                Self::RequestUpload(<RequestUploadRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RoutineControl => {
                Self::RoutineControl(<RoutineControlRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(<SecurityAccessRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(<TesterPresentRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::TransferData => {
                Self::TransferData(<TransferDataRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(
                <WriteDataByIdentifierRequest as Decode>::decode_exact(payload)?,
            ),
            _ => Self::Other {
                sid: buf[0],
                data: payload,
            },
        };
        Ok((request, &[]))
    }
}

impl Encode for Request<'_> {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let sid = match self {
            Self::Other { sid, .. } => *sid,
            other => other.service().to_request_sid(),
        };
        let sid_len = write_u8(writer, sid).map_err(Error::io)?;
        let payload = match self {
            Self::ClearDiagnosticInfo(req) => req.encode(writer)?,
            Self::CommunicationControl(req) => req.encode(writer)?,
            Self::ControlDtcSetting(req) => req.encode(writer)?,
            Self::DiagnosticSessionControl(req) => req.encode(writer)?,
            Self::EcuReset(req) => req.encode(writer)?,
            Self::ReadDataByIdentifier(req) => req.encode(writer)?,
            Self::ReadDtcInfo(req) => req.encode(writer)?,
            Self::WriteDataByIdentifier(req) => req.encode(writer)?,
            Self::RequestDownload(req) => req.encode(writer)?,
            Self::RequestFileTransfer(req) => req.encode(writer)?,
            Self::RequestTransferExit(req) => req.encode(writer)?,
            Self::RequestUpload(req) => req.encode(writer)?,
            Self::Other { data, .. } => write_all(writer, data).map_err(Error::io)?,
            Self::RoutineControl(req) => req.encode(writer)?,
            Self::SecurityAccess(req) => req.encode(writer)?,
            Self::TesterPresent(req) => req.encode(writer)?,
            Self::TransferData(req) => req.encode(writer)?,
        };
        Ok(sid_len + payload)
    }
}

impl Request<'_> {
    /// Whether the positive response for this request is suppressed (SPRMIB).
    #[must_use]
    pub fn is_positive_response_suppressed(&self) -> bool {
        match self {
            Self::CommunicationControl(req) => req.suppress_positive_response,
            Self::ControlDtcSetting(req) => req.suppress_positive_response,
            Self::DiagnosticSessionControl(req) => req.suppress_positive_response,
            Self::EcuReset(req) => req.suppress_positive_response,
            Self::RoutineControl(req) => req.suppress_positive_response,
            Self::SecurityAccess(req) => req.suppress_positive_response,
            Self::TesterPresent(req) => req.suppress_positive_response,
            _ => false,
        }
    }

    /// The [`NegativeResponseCode`]s this request's service is allowed to be answered with.
    ///
    /// Each request type also exposes this as an associated function (e.g.
    /// [`EcuResetRequest::allowed_nack_codes`]); this dispatches to it for a decoded
    /// [`Request`], so a server does not have to re-match every variant to reach it.
    ///
    /// Returns an empty slice for [`Request::Other`], which covers services the crate does not
    /// model. That means "the NRC set is unknown", not "no codes apply" — consult ISO 14229-1
    /// for those services.
    #[must_use]
    pub fn allowed_nack_codes(&self) -> &'static [NegativeResponseCode] {
        match self {
            Self::ClearDiagnosticInfo(_) => ClearDiagnosticInfoRequest::allowed_nack_codes(),
            Self::CommunicationControl(_) => CommunicationControlRequest::allowed_nack_codes(),
            Self::ControlDtcSetting(_) => ControlDtcSettingRequest::allowed_nack_codes(),
            Self::DiagnosticSessionControl(_) => {
                DiagnosticSessionControlRequest::allowed_nack_codes()
            }
            Self::EcuReset(_) => EcuResetRequest::allowed_nack_codes(),
            Self::ReadDataByIdentifier(_) => ReadDataByIdentifierRequest::allowed_nack_codes(),
            Self::ReadDtcInfo(_) => ReadDtcInfoRequest::allowed_nack_codes(),
            Self::RequestDownload(_) => RequestDownloadRequest::allowed_nack_codes(),
            Self::RequestFileTransfer(_) => RequestFileTransferRequest::allowed_nack_codes(),
            Self::RequestTransferExit(_) => RequestTransferExitRequest::allowed_nack_codes(),
            Self::RequestUpload(_) => RequestUploadRequest::allowed_nack_codes(),
            Self::RoutineControl(_) => RoutineControlRequest::allowed_nack_codes(),
            Self::SecurityAccess(_) => SecurityAccessRequest::allowed_nack_codes(),
            Self::TesterPresent(_) => TesterPresentRequest::allowed_nack_codes(),
            Self::TransferData(_) => TransferDataRequest::allowed_nack_codes(),
            Self::WriteDataByIdentifier(_) => WriteDataByIdentifierRequest::allowed_nack_codes(),
            Self::Other { .. } => &[],
        }
    }

    /// Returns the [`UdsServiceType`] corresponding to this request variant.
    #[must_use]
    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::ClearDiagnosticInfo(_) => UdsServiceType::ClearDiagnosticInfo,
            Self::CommunicationControl(_) => UdsServiceType::CommunicationControl,
            Self::ControlDtcSetting(_) => UdsServiceType::ControlDtcSetting,
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::ReadDataByIdentifier(_) => UdsServiceType::ReadDataByIdentifier,
            Self::ReadDtcInfo(_) => UdsServiceType::ReadDtcInfo,
            Self::RequestDownload(_) => UdsServiceType::RequestDownload,
            Self::RequestFileTransfer(_) => UdsServiceType::RequestFileTransfer,
            Self::RequestTransferExit(_) => UdsServiceType::RequestTransferExit,
            Self::RequestUpload(_) => UdsServiceType::RequestUpload,
            Self::RoutineControl(_) => UdsServiceType::RoutineControl,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent(_) => UdsServiceType::TesterPresent,
            Self::TransferData(_) => UdsServiceType::TransferData,
            Self::WriteDataByIdentifier(_) => UdsServiceType::WriteDataByIdentifier,
            Self::Other { sid, .. } => UdsServiceType::from_request_sid(*sid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ResetType, service::UdsServiceType};

    #[test]
    fn decode_rejects_trailing_bytes() {
        // ECU reset is a fixed 1-byte payload; an extra trailing byte is a
        // malformed frame and must be rejected rather than silently dropped.
        let mut frame = [0u8; 3];
        frame[0] = UdsServiceType::EcuReset.to_request_sid();
        frame[1] = u8::from(ResetType::HardReset);
        frame[2] = 0xAA; // trailing junk
        let result = Request::decode(&frame);
        assert!(matches!(result, Err(Error::TrailingBytes(_))));
    }

    #[test]
    fn suppression_forwards_to_inner_request() {
        let suppressed = Request::EcuReset(EcuResetRequest::new(true, ResetType::HardReset));
        assert!(suppressed.is_positive_response_suppressed());

        let not_suppressed = Request::EcuReset(EcuResetRequest::new(false, ResetType::HardReset));
        assert!(!not_suppressed.is_positive_response_suppressed());
    }

    #[test]
    fn write_data_by_identifier_request_roundtrips() {
        // SID 0x2E, DID 0xF190, one data byte 0x01
        let wire = [0x2E, 0xF1, 0x90, 0x01];
        let (req, rest) = Request::decode(&wire).unwrap();
        assert!(rest.is_empty());
        assert!(matches!(req, Request::WriteDataByIdentifier(_)));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn routine_control_request_roundtrips_with_suppress_bit() {
        // SID 0x31, sub 0x81 (StartRoutine + SPRMIB), RID 0xFF00, param 0xAA
        let wire = [0x31, 0x81, 0xFF, 0x00, 0xAA];
        let (req, rest) = Request::decode(&wire).unwrap();
        assert!(rest.is_empty());
        assert!(req.is_positive_response_suppressed());
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn allowed_nack_codes_dispatches_for_every_modeled_variant() {
        // Every one of the 15 request types has an inherent `allowed_nack_codes()`, but
        // without this dispatcher a caller holding a *decoded* `Request` had to re-match all
        // 15 variants to reach it — on a `#[non_exhaustive]` enum they cannot match
        // exhaustively. Frames are minimal-but-valid for each service.
        let frames: [&[u8]; 16] = [
            &[0x14, 0xFF, 0xFF, 0xFF, 0x00], // ClearDiagnosticInfo (groupOfDTC + memorySelection)
            &[0x28, 0x00, 0x01],             // CommunicationControl
            &[0x85, 0x01],                   // ControlDtcSetting
            &[0x10, 0x01],                   // DiagnosticSessionControl
            &[0x11, 0x01],                   // EcuReset
            &[0x22, 0xF1, 0x90],             // ReadDataByIdentifier
            &[0x19, 0x02, 0xFF],             // ReadDtcInfo
            &[0x34, 0x00, 0x12, 0xBE, 0xEF, 0x10], // RequestDownload
            &[0x38, 0x02, 0x00, 0x01, b'a'], // RequestFileTransfer
            &[0x35, 0x00, 0x12, 0xBE, 0xEF, 0x10], // RequestUpload
            &[0x37],                         // RequestTransferExit
            &[0x31, 0x01, 0xFF, 0x00],       // RoutineControl
            &[0x27, 0x01, 0xAA],             // SecurityAccess
            &[0x3E, 0x00],                   // TesterPresent
            &[0x36, 0x01, 0xAA],             // TransferData
            &[0x2E, 0xF1, 0x90, 0x01],       // WriteDataByIdentifier
        ];
        for frame in frames {
            let (req, _) = Request::decode(frame).unwrap_or_else(|e| {
                panic!("frame {frame:02X?} should decode, got {e:?}");
            });
            assert!(
                !matches!(req, Request::Other { .. }),
                "frame {frame:02X?} decoded to Other; the table needs updating"
            );
            assert!(
                !req.allowed_nack_codes().is_empty(),
                "{:?} returned no NRCs",
                req.service()
            );
        }
    }

    #[test]
    fn allowed_nack_codes_agrees_with_the_inherent_method() {
        let (req, _) = Request::decode(&[0x11, 0x01]).unwrap();
        assert_eq!(
            req.allowed_nack_codes(),
            EcuResetRequest::allowed_nack_codes()
        );
    }

    #[test]
    fn allowed_nack_codes_is_empty_for_pass_through() {
        // 0x23 ReadMemoryByAddress is enumerated but unmodeled, so the crate has no NRC table
        // for it. An empty slice says "unknown", not "none apply".
        let (req, _) = Request::decode(&[0x23, 0xAA]).unwrap();
        assert!(matches!(req, Request::Other { .. }));
        assert!(req.allowed_nack_codes().is_empty());
    }

    #[test]
    fn unmodeled_service_decodes_to_other() {
        // 0x23 = ReadMemoryByAddress, enumerated but not modeled.
        let frame = [0x23, 0xAA, 0xBB];
        let (req, rest) = Request::decode(&frame).unwrap();
        assert!(rest.is_empty());
        match req {
            Request::Other { sid, data } => {
                assert_eq!(sid, 0x23);
                assert_eq!(data, &[0xAA, 0xBB]);
            }
            other => panic!("expected Other, got {other:?}"),
        }
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &frame);
    }

    #[test]
    fn unknown_request_byte_round_trips_losslessly() {
        // 0x40 is not in the ISO request table; it must survive a decode→encode round-trip.
        let frame = [0x40, 0xAA, 0xBB];
        let (req, rest) = Request::decode(&frame).unwrap();
        assert!(rest.is_empty());
        match req {
            Request::Other { sid, data } => {
                assert_eq!(sid, 0x40);
                assert_eq!(data, &[0xAA, 0xBB]);
            }
            other => panic!("expected Other, got {other:?}"),
        }
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &frame); // previously re-encoded as 0x7F
    }
}
