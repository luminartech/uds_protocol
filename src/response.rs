use crate::{
    CommunicationControlResponse, CommunicationControlType, ControlDTCSettingsResponse, Decode,
    DiagnosticDefinition, DiagnosticSessionControlResponse, DiagnosticSessionType, DtcSettings,
    EcuResetResponse, Error, NegativeResponse, NegativeResponseCode, ReadDTCInfoResponse,
    ReadDTCInfoResponseRx, ReadDataByIdentifierResponse, RequestDownloadResponse,
    RequestDownloadResponseTx, RequestFileTransferResponse, ResetType, RoutineControlResponse,
    SecurityAccessResponse, SecurityAccessResponseTx, SecurityAccessType, SingleValueWireFormat,
    TesterPresentResponse, TransferDataResponse, TransferDataResponseTx, UdsServiceType,
    WireFormat, WriteDataByIdentifierResponse,
};
use byteorder_embedded_io::io::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// A raw UDS response consisting of the service type and its unparsed payload bytes.
#[non_exhaustive]
pub struct UdsResponse {
    /// The service this response corresponds to.
    pub service: UdsServiceType,
    /// The raw payload bytes following the service identifier.
    pub data: Vec<u8>,
}

/// Parsed UDS response. Each variant corresponds to a different UDS service response.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Response<D: DiagnosticDefinition> {
    /// Response to a [`ClearDiagnosticInfoRequest`](crate::ClearDiagnosticInfoRequest)
    ClearDiagnosticInfo,
    /// Response to a [`CommunicationControlRequest`](crate::CommunicationControlRequest)
    CommunicationControl(CommunicationControlResponse),
    /// Response to a [`ControlDTCSettingsRequest`](crate::ControlDTCSettingsRequest)
    ControlDTCSettings(ControlDTCSettingsResponse),
    /// Response to a [`DiagnosticSessionControlRequest`](crate::DiagnosticSessionControlRequest)
    DiagnosticSessionControl(DiagnosticSessionControlResponse),
    /// Response to a [`EcuResetRequest`](crate::EcuResetRequest)
    EcuReset(EcuResetResponse),
    /// Negative response to any request
    NegativeResponse(NegativeResponse),
    /// Response to a [`ReadDataByIdentifierRequest`](crate::ReadDataByIdentifierRequest)
    ReadDataByIdentifier(ReadDataByIdentifierResponse<D::DiagnosticPayload>),
    /// Response to a [`ReadDTCInfoRequest`](crate::ReadDTCInfoRequest)
    ReadDTCInfo(ReadDTCInfoResponse<D::DiagnosticPayload>),
    /// Response to a [`RequestDownloadRequest`](crate::RequestDownloadRequest)
    RequestDownload(RequestDownloadResponse),
    /// Response to a [`RequestFileTransferRequest`](crate::RequestFileTransferRequest)
    RequestFileTransfer(RequestFileTransferResponse),
    /// Response to a `RequestTransferExit` request
    RequestTransferExit,
    /// Response to a [`RoutineControl` request](crate::RoutineControlRequest)
    RoutineControl(RoutineControlResponse<D::RoutinePayload>),
    /// Response to a [`SecurityAccessRequest`](crate::SecurityAccessRequest)
    SecurityAccess(SecurityAccessResponse),
    /// Response to a [`TesterPresentRequest`](crate::TesterPresentRequest)
    TesterPresent(TesterPresentResponse),
    /// Response to a [`TransferDataRequest`](crate::TransferDataRequest)
    TransferData(TransferDataResponse),
    /// Response to a [`WriteDataByIdentifierRequest`](crate::WriteDataByIdentifierRequest)
    WriteDataByIdentifier(WriteDataByIdentifierResponse<D::DID>),
}

impl<D: DiagnosticDefinition> Response<D> {
    /// Create a `ClearDiagnosticInfo` positive response.
    #[must_use]
    pub fn clear_diagnostic_info() -> Self {
        Response::ClearDiagnosticInfo
    }
    /// Create a `CommunicationControl` positive response.
    #[must_use]
    pub fn communication_control(control_type: CommunicationControlType) -> Self {
        Response::CommunicationControl(CommunicationControlResponse::new(control_type))
    }

    /// Create a `ControlDTCSettings` positive response.
    #[must_use]
    pub fn control_dtc_settings(setting: DtcSettings) -> Self {
        Response::ControlDTCSettings(ControlDTCSettingsResponse::new(setting))
    }

    /// Create a `DiagnosticSessionControl` positive response with timing parameters.
    #[must_use]
    pub fn diagnostic_session_control(
        session_type: DiagnosticSessionType,
        p2_max: u16,
        p2_star_max: u16,
    ) -> Self {
        Response::DiagnosticSessionControl(DiagnosticSessionControlResponse::new(
            session_type,
            p2_max,
            p2_star_max,
        ))
    }

    /// Create an `EcuReset` positive response.
    #[must_use]
    pub fn ecu_reset(reset_type: ResetType, power_down_time: u8) -> Self {
        Response::EcuReset(EcuResetResponse::new(reset_type, power_down_time))
    }

    /// Create a negative response for the given service and response code.
    #[must_use]
    pub fn negative_response(request_service: UdsServiceType, nrc: NegativeResponseCode) -> Self {
        Response::NegativeResponse(NegativeResponse::new(request_service, nrc))
    }

    /// Create a `ReadDataByIdentifier` positive response from an iterator of payloads.
    #[must_use]
    pub fn read_data_by_identifier<I>(payload: I) -> Self
    where
        I: IntoIterator<Item = D::DiagnosticPayload>,
    {
        Response::ReadDataByIdentifier(ReadDataByIdentifierResponse::new(payload))
    }

    /// Create a `RequestDownload` positive response.
    #[must_use]
    pub fn request_download(
        length_format_identifier: u8,
        max_number_of_block_length: Vec<u8>,
    ) -> Self {
        Response::RequestDownload(RequestDownloadResponse::new(
            length_format_identifier,
            max_number_of_block_length,
        ))
    }

    /// Create a `RequestFileTransfer` positive response. Not yet implemented.
    #[must_use]
    pub fn request_file_transfer() -> Self {
        todo!()
    }

    /// Create a `RoutineControl` positive response.
    pub fn routine_control(
        routine_control_type: crate::RoutineControlSubFunction,
        data: D::RoutinePayload,
    ) -> Self {
        Response::RoutineControl(RoutineControlResponse::new(routine_control_type, data))
    }

    /// Create a `SecurityAccess` positive response carrying the security seed.
    #[must_use]
    pub fn security_access(access_type: SecurityAccessType, security_seed: Vec<u8>) -> Self {
        Response::SecurityAccess(SecurityAccessResponse::new(access_type, security_seed))
    }

    /// Create a `TesterPresent` positive response.
    #[must_use]
    pub fn tester_present() -> Self {
        Response::TesterPresent(TesterPresentResponse::new())
    }

    /// Create a `TransferData` positive response.
    #[must_use]
    pub fn transfer_data(block_sequence_counter: u8, data: Vec<u8>) -> Self {
        Response::TransferData(TransferDataResponse::new(block_sequence_counter, data))
    }

    /// Returns the [`UdsServiceType`] corresponding to this response variant.
    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::ClearDiagnosticInfo => UdsServiceType::ClearDiagnosticInfo,
            Self::CommunicationControl(_) => UdsServiceType::CommunicationControl,
            Self::ControlDTCSettings(_) => UdsServiceType::ControlDTCSettings,
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::NegativeResponse(_) => UdsServiceType::NegativeResponse,
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
        }
    }
}

impl<D: DiagnosticDefinition> WireFormat for Response<D> {
    #[allow(clippy::match_same_arms)]
    fn required_size(&self) -> usize {
        1 + match self {
            Self::ClearDiagnosticInfo => 0,
            Self::CommunicationControl(cc) => cc.required_size(),
            Self::ControlDTCSettings(dtc) => dtc.required_size(),
            Self::DiagnosticSessionControl(ds) => ds.required_size(),
            Self::EcuReset(reset) => reset.required_size(),
            Self::NegativeResponse(nr) => nr.required_size(),
            Self::ReadDataByIdentifier(rd) => rd.required_size(),
            Self::ReadDTCInfo(rd) => rd.required_size(),
            Self::RequestDownload(rd) => rd.required_size(),
            Self::RequestFileTransfer(rft) => rft.required_size(),
            Self::RequestTransferExit => 0,
            Self::RoutineControl(rc) => rc.required_size(),
            Self::SecurityAccess(sa) => sa.required_size(),
            Self::TesterPresent(tp) => tp.required_size(),
            Self::TransferData(td) => td.required_size(),
            Self::WriteDataByIdentifier(wdbi) => wdbi.required_size(),
        }
    }

    #[allow(clippy::match_same_arms)]
    fn encode<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Write the service byte
        writer.write_u8(self.service().response_to_byte())?;
        // Write the payload
        Ok(1 + match self {
            Self::ClearDiagnosticInfo => Ok(0),
            Self::CommunicationControl(cc) => cc.encode(writer),
            Self::ControlDTCSettings(dtc) => dtc.encode(writer),
            Self::DiagnosticSessionControl(ds) => ds.encode(writer),
            Self::EcuReset(reset) => reset.encode(writer),
            Self::NegativeResponse(nr) => nr.encode(writer),
            Self::ReadDataByIdentifier(rd) => rd.encode(writer),
            Self::ReadDTCInfo(rd) => rd.encode(writer),
            Self::RequestDownload(rd) => rd.encode(writer),
            Self::RequestFileTransfer(rft) => rft.encode(writer),
            Self::RequestTransferExit => Ok(0),
            Self::RoutineControl(rc) => rc.encode(writer),
            Self::SecurityAccess(sa) => sa.encode(writer),
            Self::TesterPresent(tp) => tp.encode(writer),
            Self::TransferData(td) => td.encode(writer),
            Self::WriteDataByIdentifier(wdbi) => wdbi.encode(writer),
        }?)
    }
}

impl<D: DiagnosticDefinition> SingleValueWireFormat for Response<D> {
    #[allow(clippy::too_many_lines)]
    fn decode<T: Read>(reader: &mut T) -> Result<Self, Error> {
        let service = UdsServiceType::response_from_byte(reader.read_u8()?);
        Ok(match service {
            UdsServiceType::CommunicationControl => {
                Self::CommunicationControl(<CommunicationControlResponse as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::ControlDTCSettings => {
                Self::ControlDTCSettings(<ControlDTCSettingsResponse as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::DiagnosticSessionControl => {
                Self::DiagnosticSessionControl(<DiagnosticSessionControlResponse as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::EcuReset => Self::EcuReset(<EcuResetResponse as SingleValueWireFormat>::decode(reader)?),
            UdsServiceType::ReadDataByIdentifier => {
                Self::ReadDataByIdentifier(<ReadDataByIdentifierResponse<D::DiagnosticPayload> as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::ReadDTCInfo => Self::ReadDTCInfo(<ReadDTCInfoResponse<D::DiagnosticPayload> as SingleValueWireFormat>::decode(reader)?),
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(<RequestDownloadResponse as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::RequestFileTransfer => {
                Self::RequestFileTransfer(<RequestFileTransferResponse as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::RoutineControl => {
                Self::RoutineControl(<RoutineControlResponse<D::RoutinePayload> as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(<SecurityAccessResponse as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(<TesterPresentResponse as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::NegativeResponse => {
                Self::NegativeResponse(<NegativeResponse as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::WriteDataByIdentifier => {
                Self::WriteDataByIdentifier(<WriteDataByIdentifierResponse<D::DID> as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::Authentication => {
                return Err(Error::ServiceNotImplemented(UdsServiceType::Authentication));
            }
            UdsServiceType::AccessTimingParameters => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::AccessTimingParameters,
                ));
            }
            UdsServiceType::SecuredDataTransmission => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::SecuredDataTransmission,
                ));
            }
            UdsServiceType::ResponseOnEvent => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ResponseOnEvent,
                ));
            }
            UdsServiceType::LinkControl => {
                return Err(Error::ServiceNotImplemented(UdsServiceType::LinkControl));
            }
            UdsServiceType::ReadMemoryByAddress => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ReadMemoryByAddress,
                ));
            }
            UdsServiceType::ReadScalingDataByIdentifier => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ReadScalingDataByIdentifier,
                ));
            }
            UdsServiceType::ReadDataByIdentifierPeriodic => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ReadDataByIdentifierPeriodic,
                ));
            }
            UdsServiceType::DynamicallyDefinedDataIdentifier => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::DynamicallyDefinedDataIdentifier,
                ));
            }
            UdsServiceType::WriteMemoryByAddress => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::WriteMemoryByAddress,
                ));
            }
            UdsServiceType::ClearDiagnosticInfo => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ClearDiagnosticInfo,
                ));
            }
            UdsServiceType::InputOutputControlByIdentifier => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::InputOutputControlByIdentifier,
                ));
            }
            UdsServiceType::RequestUpload => {
                return Err(Error::ServiceNotImplemented(UdsServiceType::RequestUpload));
            }
            UdsServiceType::TransferData => {
                Self::TransferData(<TransferDataResponse as SingleValueWireFormat>::decode(reader)?)
            }
            UdsServiceType::UnsupportedDiagnosticService => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::UnsupportedDiagnosticService,
                ));
            }
        })
    }
}

// ---------------------------------------------------------------------------
// no_std RX response enum (zero-copy, no DiagnosticDefinition needed)
// ---------------------------------------------------------------------------

/// Zero-copy RX response. Borrows from the wire buffer.
///
/// Unlike [`Response<D>`], this enum does not require a [`DiagnosticDefinition`]
/// generic parameter — variable-length payloads are stored as raw `&'a [u8]`
/// slices that can be further parsed on demand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ResponseRx<'a> {
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

impl<'a> Decode<'a> for ResponseRx<'a> {
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
///
/// Replaces the allocating [`UdsResponse`] for `no_std` use.
#[derive(Clone, Debug)]
pub struct UdsResponseRx<'a> {
    /// The service this response corresponds to.
    pub service: UdsServiceType,
    /// The raw payload bytes following the service identifier.
    pub data: &'a [u8],
}

impl<'a> Decode<'a> for UdsResponseRx<'a> {
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
