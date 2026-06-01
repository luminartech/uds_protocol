use crate::{
    CommunicationControlResponse, ControlDTCSettingsResponse, Decode,
    DiagnosticSessionControlResponse, EcuResetResponse, Encode, Error, NegativeResponse,
    ReadDTCInfoResponseRx, RequestDownloadResponseTx, RequestFileTransferResponseTx,
    SecurityAccessResponseTx, TesterPresentResponse, TransferDataResponseTx, UdsServiceType,
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
    /// Positive response to `RequestFileTransfer`.
    RequestFileTransfer(RequestFileTransferResponseTx<'a>),
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
                Self::ReadDTCInfo(<ReadDTCInfoResponseRx as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(<RequestDownloadResponseTx as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::RequestFileTransfer => Self::RequestFileTransfer(
                <RequestFileTransferResponseTx as Decode>::decode_exact(payload)?,
            ),
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
                Self::SecurityAccess(<SecurityAccessResponseTx as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(<TesterPresentResponse as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::TransferData => {
                Self::TransferData(<TransferDataResponseTx as Decode>::decode_exact(payload)?)
            }
            UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(payload),
            _ => return Err(Error::ServiceNotImplemented(service)),
        };
        Ok((response, &[]))
    }
}

impl Response<'_> {
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
            Self::RequestTransferExit => UdsServiceType::RequestTransferExit.response_to_byte(),
            Self::RoutineControl { .. } => UdsServiceType::RoutineControl.response_to_byte(),
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess.response_to_byte(),
            Self::TesterPresent(_) => UdsServiceType::TesterPresent.response_to_byte(),
            Self::TransferData(_) => UdsServiceType::TransferData.response_to_byte(),
            Self::WriteDataByIdentifier(_) => {
                UdsServiceType::WriteDataByIdentifier.response_to_byte()
            }
        }
    }
}

impl Encode for Response<'_> {
    fn encoded_size(&self) -> usize {
        let payload = match self {
            Self::ClearDiagnosticInfo | Self::RequestTransferExit => 0,
            Self::CommunicationControl(resp) => resp.encoded_size(),
            Self::ControlDTCSettings(resp) => resp.encoded_size(),
            Self::DiagnosticSessionControl(resp) => resp.encoded_size(),
            Self::EcuReset(resp) => resp.encoded_size(),
            Self::NegativeResponse(resp) => resp.encoded_size(),
            Self::ReadDataByIdentifier(bytes) | Self::WriteDataByIdentifier(bytes) => bytes.len(),
            Self::ReadDTCInfo(resp) => resp.encoded_size(),
            Self::RequestDownload(resp) => resp.encoded_size(),
            Self::RequestFileTransfer(resp) => resp.encoded_size(),
            Self::RoutineControl {
                raw_status_record, ..
            } => 1 + raw_status_record.len(),
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
            Self::ClearDiagnosticInfo | Self::RequestTransferExit => 0,
            Self::CommunicationControl(resp) => resp.encode(writer)?,
            Self::ControlDTCSettings(resp) => resp.encode(writer)?,
            Self::DiagnosticSessionControl(resp) => resp.encode(writer)?,
            Self::EcuReset(resp) => resp.encode(writer)?,
            Self::NegativeResponse(resp) => resp.encode(writer)?,
            Self::ReadDataByIdentifier(bytes) | Self::WriteDataByIdentifier(bytes) => {
                writer.write_all(bytes).map_err(Error::io)?;
                bytes.len()
            }
            Self::ReadDTCInfo(resp) => resp.encode(writer)?,
            Self::RequestDownload(resp) => resp.encode(writer)?,
            Self::RequestFileTransfer(resp) => resp.encode(writer)?,
            Self::RoutineControl {
                routine_control_type,
                raw_status_record,
            } => {
                writer
                    .write_all(&[*routine_control_type])
                    .map_err(Error::io)?;
                writer.write_all(raw_status_record).map_err(Error::io)?;
                1 + raw_status_record.len()
            }
            Self::SecurityAccess(resp) => resp.encode(writer)?,
            Self::TesterPresent(resp) => resp.encode(writer)?,
            Self::TransferData(resp) => resp.encode(writer)?,
        };
        Ok(1 + payload)
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
