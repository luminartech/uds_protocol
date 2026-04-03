use crate::{
    CommunicationControlResponse, ControlDTCSettingsResponse, Decode,
    DiagnosticSessionControlResponse, EcuResetResponse, Error, NegativeResponse,
    ReadDTCInfoResponseRx, RequestDownloadResponseTx, SecurityAccessResponseTx,
    TesterPresentResponse, TransferDataResponseTx, UdsServiceType,
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
    ReadDTCInfo(ReadDTCInfoResponseRx<'a>),
    /// Positive response to `RequestDownload`.
    RequestDownload(RequestDownloadResponseTx<'a>),
    /// Positive response to `RequestTransferExit`.
    RequestTransferExit,
    /// Positive response to `RoutineControl`. Raw status record bytes.
    RoutineControl {
        /// The routine control sub-function echo.
        routine_control_type: u8,
        /// Raw routine status record bytes.
        raw_status_record: &'a [u8],
    },
    /// Positive response to `SecurityAccess`.
    SecurityAccess(SecurityAccessResponseTx<'a>),
    /// Positive response to `TesterPresent`.
    TesterPresent(TesterPresentResponse),
    /// Positive response to `TransferData`.
    TransferData(TransferDataResponseTx<'a>),
    /// Positive response to `WriteDataByIdentifier`. Contains the echoed DID bytes.
    WriteDataByIdentifier(&'a [u8]),
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
            UdsServiceType::CommunicationControl => {
                let (resp, _) = <CommunicationControlResponse as Decode>::decode(payload)?;
                Self::CommunicationControl(resp)
            }
            UdsServiceType::ControlDTCSettings => {
                let (resp, _) = <ControlDTCSettingsResponse as Decode>::decode(payload)?;
                Self::ControlDTCSettings(resp)
            }
            UdsServiceType::DiagnosticSessionControl => {
                let (resp, _) =
                    <DiagnosticSessionControlResponse as Decode>::decode(payload)?;
                Self::DiagnosticSessionControl(resp)
            }
            UdsServiceType::EcuReset => {
                let (resp, _) = <EcuResetResponse as Decode>::decode(payload)?;
                Self::EcuReset(resp)
            }
            UdsServiceType::NegativeResponse => {
                let (resp, _) = <NegativeResponse as Decode>::decode(payload)?;
                Self::NegativeResponse(resp)
            }
            UdsServiceType::ReadDataByIdentifier => Self::ReadDataByIdentifier(payload),
            UdsServiceType::ReadDTCInfo => {
                let (resp, _) = <ReadDTCInfoResponseRx as Decode>::decode(payload)?;
                Self::ReadDTCInfo(resp)
            }
            UdsServiceType::RequestDownload => {
                let (resp, _) = <RequestDownloadResponseTx as Decode>::decode(payload)?;
                Self::RequestDownload(resp)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::RoutineControl => {
                if payload.is_empty() {
                    return Err(Error::InsufficientData(2));
                }
                Self::RoutineControl {
                    routine_control_type: payload[0],
                    raw_status_record: &payload[1..],
                }
            }
            UdsServiceType::SecurityAccess => {
                let (resp, _) = <SecurityAccessResponseTx as Decode>::decode(payload)?;
                Self::SecurityAccess(resp)
            }
            UdsServiceType::TesterPresent => {
                let (resp, _) = <TesterPresentResponse as Decode>::decode(payload)?;
                Self::TesterPresent(resp)
            }
            UdsServiceType::TransferData => {
                let (resp, _) = <TransferDataResponseTx as Decode>::decode(payload)?;
                Self::TransferData(resp)
            }
            UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(payload),
            _ => return Err(Error::ServiceNotImplemented(service)),
        };
        Ok((response, &[]))
    }
}

/// Zero-copy raw RX response. Borrows from the wire buffer.
#[derive(Clone, Debug)]
pub struct UdsResponse<'a> {
    /// The service this response corresponds to.
    pub service: UdsServiceType,
    /// The raw payload bytes following the service identifier.
    pub data: &'a [u8],
}

impl<'a> Decode<'a> for UdsResponse<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        Ok((
            Self {
                service: UdsServiceType::response_from_byte(buf[0]),
                data: &buf[1..],
            },
            &[],
        ))
    }
}
