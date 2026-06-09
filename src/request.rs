//! Module for making and handling UDS Requests
use crate::{
    Decode, Encode, Error,
    services::{
        ClearDiagnosticInfoRequest, CommunicationControlRequest, ControlDTCSettingsRequest,
        DiagnosticSessionControlRequest, EcuResetRequest, ReadDTCInfoRequest,
        RequestDownloadRequest, RequestFileTransferRequest, RoutineControlRequest,
        SecurityAccessRequest, TesterPresentRequest, TransferDataRequest,
        WriteDataByIdentifierRequest,
    },
};

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
    ControlDTCSettings(ControlDTCSettingsRequest),
    /// Diagnostic session control request.
    DiagnosticSessionControl(DiagnosticSessionControlRequest),
    /// ECU reset request.
    EcuReset(EcuResetRequest),
    /// Read data by identifier request. Raw DID bytes.
    ReadDataByIdentifier(&'a [u8]),
    /// Read DTC information request.
    ReadDTCInfo(ReadDTCInfoRequest),
    /// Request download.
    RequestDownload(RequestDownloadRequest),
    /// Request file transfer.
    RequestFileTransfer(RequestFileTransferRequest<'a>),
    /// Request transfer exit.
    RequestTransferExit,
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
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let service = UdsServiceType::service_from_request_byte(buf[0]);
        let payload = &buf[1..];

        let request = match service {
            UdsServiceType::ClearDiagnosticInfo => Self::ClearDiagnosticInfo(
                <ClearDiagnosticInfoRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::CommunicationControl => Self::CommunicationControl(
                <CommunicationControlRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::ControlDTCSettings => Self::ControlDTCSettings(
                <ControlDTCSettingsRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::DiagnosticSessionControl => Self::DiagnosticSessionControl(
                <DiagnosticSessionControlRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::EcuReset => {
                Self::EcuReset(<EcuResetRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::ReadDataByIdentifier => Self::ReadDataByIdentifier(payload),
            UdsServiceType::ReadDTCInfo => {
                Self::ReadDTCInfo(<ReadDTCInfoRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(<RequestDownloadRequest as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestFileTransfer => Self::RequestFileTransfer(
                <RequestFileTransferRequest as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
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
    fn encoded_size(&self) -> usize {
        let payload = match self {
            Self::ClearDiagnosticInfo(req) => req.encoded_size(),
            Self::CommunicationControl(req) => req.encoded_size(),
            Self::ControlDTCSettings(req) => req.encoded_size(),
            Self::DiagnosticSessionControl(req) => req.encoded_size(),
            Self::EcuReset(req) => req.encoded_size(),
            Self::ReadDataByIdentifier(bytes) => bytes.len(),
            Self::ReadDTCInfo(req) => req.encoded_size(),
            Self::WriteDataByIdentifier(req) => req.encoded_size(),
            Self::RequestDownload(req) => req.encoded_size(),
            Self::RequestFileTransfer(req) => req.encoded_size(),
            Self::RequestTransferExit => 0,
            Self::Other { data, .. } => data.len(),
            Self::RoutineControl(req) => req.encoded_size(),
            Self::SecurityAccess(req) => req.encoded_size(),
            Self::TesterPresent(req) => req.encoded_size(),
            Self::TransferData(req) => req.encoded_size(),
        };
        1 + payload
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let sid = match self {
            Self::Other { sid, .. } => *sid,
            other => other.service().request_service_to_byte(),
        };
        writer.write_all(&[sid]).map_err(Error::io)?;
        let payload = match self {
            Self::ClearDiagnosticInfo(req) => req.encode(writer)?,
            Self::CommunicationControl(req) => req.encode(writer)?,
            Self::ControlDTCSettings(req) => req.encode(writer)?,
            Self::DiagnosticSessionControl(req) => req.encode(writer)?,
            Self::EcuReset(req) => req.encode(writer)?,
            Self::ReadDataByIdentifier(bytes) => {
                writer.write_all(bytes).map_err(Error::io)?;
                bytes.len()
            }
            Self::ReadDTCInfo(req) => req.encode(writer)?,
            Self::WriteDataByIdentifier(req) => req.encode(writer)?,
            Self::RequestDownload(req) => req.encode(writer)?,
            Self::RequestFileTransfer(req) => req.encode(writer)?,
            Self::RequestTransferExit => 0,
            Self::Other { data, .. } => {
                writer.write_all(data).map_err(Error::io)?;
                data.len()
            }
            Self::RoutineControl(req) => req.encode(writer)?,
            Self::SecurityAccess(req) => req.encode(writer)?,
            Self::TesterPresent(req) => req.encode(writer)?,
            Self::TransferData(req) => req.encode(writer)?,
        };
        Ok(1 + payload)
    }
}

impl Request<'_> {
    /// Whether the positive response for this request is suppressed (SPRMIB).
    #[must_use]
    pub fn is_positive_response_suppressed(&self) -> bool {
        match self {
            Self::CommunicationControl(req) => req.suppress_positive_response(),
            Self::ControlDTCSettings(req) => req.suppress_positive_response(),
            Self::DiagnosticSessionControl(req) => req.suppress_positive_response(),
            Self::EcuReset(req) => req.suppress_positive_response(),
            Self::RoutineControl(req) => req.suppress_positive_response(),
            Self::SecurityAccess(req) => req.suppress_positive_response(),
            Self::TesterPresent(req) => req.suppress_positive_response(),
            _ => false,
        }
    }

    /// Returns the [`UdsServiceType`] corresponding to this request variant.
    #[must_use]
    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::ClearDiagnosticInfo(_) => UdsServiceType::ClearDiagnosticInfo,
            Self::CommunicationControl(_) => UdsServiceType::CommunicationControl,
            Self::ControlDTCSettings(_) => UdsServiceType::ControlDTCSettings,
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::ReadDataByIdentifier(_) => UdsServiceType::ReadDataByIdentifier,
            Self::ReadDTCInfo(_) => UdsServiceType::ReadDTCInfo,
            Self::RequestDownload(_) => UdsServiceType::RequestDownload,
            Self::RequestFileTransfer(_) => UdsServiceType::RequestFileTransfer,
            Self::RequestTransferExit => UdsServiceType::RequestTransferExit,
            Self::RoutineControl(_) => UdsServiceType::RoutineControl,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent(_) => UdsServiceType::TesterPresent,
            Self::TransferData(_) => UdsServiceType::TransferData,
            Self::WriteDataByIdentifier(_) => UdsServiceType::WriteDataByIdentifier,
            Self::Other { sid, .. } => UdsServiceType::service_from_request_byte(*sid),
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
        frame[0] = UdsServiceType::EcuReset.request_service_to_byte();
        frame[1] = u8::from(ResetType::HardReset);
        frame[2] = 0xAA; // trailing junk
        let result = Request::decode(&frame);
        assert!(matches!(
            result,
            Err(Error::IncorrectMessageLengthOrInvalidFormat)
        ));
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
