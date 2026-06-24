use crate::{
    CommunicationControlResponse, ControlDTCSettingsResponse, Decode,
    DiagnosticSessionControlResponse, EcuResetResponse, Encode, Error, NegativeResponse,
    ReadDTCInfoResponse, RequestDownloadResponse, RequestFileTransferResponse,
    RequestTransferExitResponse, RoutineControlResponse, SecurityAccessResponse,
    TesterPresentResponse, TransferDataResponse, UdsServiceType, WriteDataByIdentifierResponse,
};

/// Parsed zero-copy UDS response. Borrows from the wire buffer.
///
/// Variable-length payloads are stored as raw `&'a [u8]` slices that can be
/// further parsed on demand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Response<'a> {
    /// Positive response to `ClearDiagnosticInfo`.
    ClearDiagnosticInfo,
    /// Positive response to `CommunicationControl`.
    CommunicationControl(CommunicationControlResponse),
    /// Positive response to `ControlDTCSettings`.
    ControlDTCSettings(ControlDTCSettingsResponse),
    /// Positive response to `DiagnosticSessionControl`.
    DiagnosticSessionControl(DiagnosticSessionControlResponse),
    /// Positive response to `EcuReset`.
    EcuReset(EcuResetResponse),
    /// Negative response to any request.
    NegativeResponse(NegativeResponse),
    /// Positive response to `ReadDataByIdentifier`. Raw payload bytes.
    ReadDataByIdentifier(&'a [u8]),
    /// Positive response to `ReadDTCInformation` with lazy iterators.
    ReadDTCInfo(ReadDTCInfoResponse<'a>),
    /// Positive response to `RequestDownload`.
    RequestDownload(RequestDownloadResponse<'a>),
    /// Positive response to `RequestFileTransfer`.
    RequestFileTransfer(RequestFileTransferResponse<'a>),
    /// Positive response to `RequestTransferExit`.
    RequestTransferExit(RequestTransferExitResponse<'a>),
    /// Positive response to `RoutineControl`.
    RoutineControl(RoutineControlResponse<'a>),
    /// Positive response to `SecurityAccess`.
    SecurityAccess(SecurityAccessResponse<'a>),
    /// Positive response to `TesterPresent`.
    TesterPresent(TesterPresentResponse),
    /// Positive response to `TransferData`.
    TransferData(TransferDataResponse<'a>),
    /// Positive response to `WriteDataByIdentifier`. Contains the echoed DID.
    WriteDataByIdentifier(WriteDataByIdentifierResponse),
    /// A known-but-unmodeled (or unrecognized) service response. Carries the raw service
    /// byte and the raw payload bytes following the service identifier.
    ///
    /// Re-encoding is lossless for every service byte: the raw `sid` is echoed verbatim.
    Other {
        /// The raw service identifier byte from the wire.
        sid: u8,
        /// Raw payload bytes after the service byte.
        data: &'a [u8],
    },
}

impl<'a> Decode<'a> for Response<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let service = UdsServiceType::response_from_byte(buf[0]);
        let payload = &buf[1..];

        let response = match service {
            UdsServiceType::ClearDiagnosticInfo => Self::ClearDiagnosticInfo,
            UdsServiceType::CommunicationControl => Self::CommunicationControl(
                <CommunicationControlResponse as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::ControlDTCSettings => Self::ControlDTCSettings(
                <ControlDTCSettingsResponse as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::DiagnosticSessionControl => Self::DiagnosticSessionControl(
                <DiagnosticSessionControlResponse as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::EcuReset => {
                Self::EcuReset(<EcuResetResponse as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::NegativeResponse => {
                Self::NegativeResponse(<NegativeResponse as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::ReadDataByIdentifier => Self::ReadDataByIdentifier(payload),
            UdsServiceType::ReadDTCInfo => {
                Self::ReadDTCInfo(<ReadDTCInfoResponse as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(<RequestDownloadResponse as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestFileTransfer => Self::RequestFileTransfer(
                <RequestFileTransferResponse as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit(
                <RequestTransferExitResponse as Decode>::decode_exact(payload)?,
            ),
            UdsServiceType::RoutineControl => {
                Self::RoutineControl(<RoutineControlResponse as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(<SecurityAccessResponse as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(<TesterPresentResponse as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::TransferData => {
                Self::TransferData(<TransferDataResponse as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(
                <WriteDataByIdentifierResponse as Decode>::decode_exact(payload)?,
            ),
            _ => Self::Other {
                sid: buf[0],
                data: payload,
            },
        };
        Ok((response, &[]))
    }
}

impl Response<'_> {
    /// The [`UdsServiceType`] this response frame addresses.
    ///
    /// For `NegativeResponse` this returns [`UdsServiceType::NegativeResponse`] (the frame's
    /// own type); the *failed* request service is `NegativeResponse.request_service`.
    #[must_use]
    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::Other { sid, .. } => UdsServiceType::response_from_byte(*sid),
            other => UdsServiceType::response_from_byte(other.response_sid()),
        }
    }

    /// Returns the response service-ID byte that frames this response on the wire.
    fn response_sid(&self) -> u8 {
        match self {
            Self::ClearDiagnosticInfo => UdsServiceType::ClearDiagnosticInfo.response_to_byte(),
            Self::CommunicationControl(_) => {
                UdsServiceType::CommunicationControl.response_to_byte()
            }
            Self::ControlDTCSettings(_) => UdsServiceType::ControlDTCSettings.response_to_byte(),
            Self::DiagnosticSessionControl(_) => {
                UdsServiceType::DiagnosticSessionControl.response_to_byte()
            }
            Self::EcuReset(_) => UdsServiceType::EcuReset.response_to_byte(),
            Self::NegativeResponse(_) => UdsServiceType::NegativeResponse.response_to_byte(),
            Self::ReadDataByIdentifier(_) => {
                UdsServiceType::ReadDataByIdentifier.response_to_byte()
            }
            Self::ReadDTCInfo(_) => UdsServiceType::ReadDTCInfo.response_to_byte(),
            Self::RequestDownload(_) => UdsServiceType::RequestDownload.response_to_byte(),
            Self::RequestFileTransfer(_) => UdsServiceType::RequestFileTransfer.response_to_byte(),
            Self::RequestTransferExit(_) => UdsServiceType::RequestTransferExit.response_to_byte(),
            Self::RoutineControl(_) => UdsServiceType::RoutineControl.response_to_byte(),
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess.response_to_byte(),
            Self::TesterPresent(_) => UdsServiceType::TesterPresent.response_to_byte(),
            Self::TransferData(_) => UdsServiceType::TransferData.response_to_byte(),
            Self::WriteDataByIdentifier(_) => {
                UdsServiceType::WriteDataByIdentifier.response_to_byte()
            }
            Self::Other { sid, .. } => *sid,
        }
    }
}

impl Encode for Response<'_> {
    fn encoded_size(&self) -> usize {
        let payload = match self {
            Self::ClearDiagnosticInfo => 0,
            Self::RequestTransferExit(resp) => resp.encoded_size(),
            Self::Other { data, .. } => data.len(),
            Self::CommunicationControl(resp) => resp.encoded_size(),
            Self::ControlDTCSettings(resp) => resp.encoded_size(),
            Self::DiagnosticSessionControl(resp) => resp.encoded_size(),
            Self::EcuReset(resp) => resp.encoded_size(),
            Self::NegativeResponse(resp) => resp.encoded_size(),
            Self::ReadDataByIdentifier(bytes) => bytes.len(),
            Self::WriteDataByIdentifier(resp) => resp.encoded_size(),
            Self::ReadDTCInfo(resp) => resp.encoded_size(),
            Self::RequestDownload(resp) => resp.encoded_size(),
            Self::RequestFileTransfer(resp) => resp.encoded_size(),
            Self::RoutineControl(resp) => resp.encoded_size(),
            Self::SecurityAccess(resp) => resp.encoded_size(),
            Self::TesterPresent(resp) => resp.encoded_size(),
            Self::TransferData(resp) => resp.encoded_size(),
        };
        1 + payload
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[self.response_sid()])
            .map_err(Error::io)?;
        let payload = match self {
            Self::ClearDiagnosticInfo => 0,
            Self::RequestTransferExit(resp) => resp.encode(writer)?,
            Self::CommunicationControl(resp) => resp.encode(writer)?,
            Self::ControlDTCSettings(resp) => resp.encode(writer)?,
            Self::DiagnosticSessionControl(resp) => resp.encode(writer)?,
            Self::EcuReset(resp) => resp.encode(writer)?,
            Self::NegativeResponse(resp) => resp.encode(writer)?,
            Self::ReadDataByIdentifier(bytes) => {
                writer.write_all(bytes).map_err(Error::io)?;
                bytes.len()
            }
            Self::WriteDataByIdentifier(resp) => resp.encode(writer)?,
            Self::ReadDTCInfo(resp) => resp.encode(writer)?,
            Self::RequestDownload(resp) => resp.encode(writer)?,
            Self::RequestFileTransfer(resp) => resp.encode(writer)?,
            Self::RoutineControl(resp) => resp.encode(writer)?,
            Self::SecurityAccess(resp) => resp.encode(writer)?,
            Self::TesterPresent(resp) => resp.encode(writer)?,
            Self::TransferData(resp) => resp.encode(writer)?,
            Self::Other { data, .. } => {
                writer.write_all(data).map_err(Error::io)?;
                data.len()
            }
        };
        Ok(1 + payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_data_by_identifier_response_roundtrips() {
        // SID 0x6E, echoed DID 0xF190
        let wire = [0x6E, 0xF1, 0x90];
        let (resp, remaining) = Response::decode(&wire).unwrap();
        assert!(remaining.is_empty());
        assert!(matches!(resp, Response::WriteDataByIdentifier(_)));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn routine_control_response_roundtrips() {
        // SID 0x71, sub 0x01, RID 0xFF00, status 0x10
        let wire = [0x71, 0x01, 0xFF, 0x00, 0x10];
        let (resp, remaining) = Response::decode(&wire).unwrap();
        assert!(remaining.is_empty());
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn unmodeled_response_decodes_to_other() {
        // 0x63 = ReadMemoryByAddress positive response, not modeled.
        let frame = [0x63, 0x01, 0x02];
        let (resp, remaining) = Response::decode(&frame).unwrap();
        assert!(remaining.is_empty());
        match resp {
            Response::Other { sid, data } => {
                assert_eq!(sid, 0x63);
                assert_eq!(data, &[0x01, 0x02]);
            }
            other => panic!("expected Other, got {other:?}"),
        }
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &frame);
    }

    #[test]
    fn unknown_response_byte_round_trips_losslessly() {
        let frame = [0x99, 0x01, 0x02];
        let (resp, _) = Response::decode(&frame).unwrap();
        assert!(matches!(resp, Response::Other { sid: 0x99, .. }));
        assert_eq!(resp.service(), UdsServiceType::response_from_byte(0x99));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &frame); // previously became 0x7F (NegativeResponse)
    }
}
