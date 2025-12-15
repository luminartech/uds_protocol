use crate::{
    CommunicationControlResponse, CommunicationControlType, ControlDTCSettingsResponse,
    DiagnosticDefinition, DiagnosticSessionControlResponse, DiagnosticSessionType, DtcSettings,
    EcuResetResponse, Error, NegativeResponse, NegativeResponseCode, ReadDTCInfoResponse,
    ReadDataByIdentifierResponse, RequestDownloadResponse, RequestFileTransferResponse, ResetType,
    RoutineControlResponse, SecurityAccessResponse, SecurityAccessType, SingleValueWireFormat,
    TesterPresentResponse, TransferDataResponse, UdsServiceType, WireFormat,
    WriteDataByIdentifierResponse,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub struct UdsResponse {
    pub service: UdsServiceType,
    pub data: Vec<u8>,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
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
    /// Response to a [`RequestDownload`](crate::RequestDownload)
    RequestDownload(RequestDownloadResponse),
    /// Response to a [`RequestFileTransfer`](crate::RequestFileTransfer)
    RequestFileTransfer(RequestFileTransferResponse),
    /// Response to a [`RequestTransferExit`](crate::RequestTransferExit)
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
    #[must_use]
    pub fn clear_diagnostic_info() -> Self {
        Response::ClearDiagnosticInfo
    }
    #[must_use]
    pub fn communication_control(control_type: CommunicationControlType) -> Self {
        Response::CommunicationControl(CommunicationControlResponse::new(control_type))
    }

    #[must_use]
    pub fn control_dtc_settings(setting: DtcSettings) -> Self {
        Response::ControlDTCSettings(ControlDTCSettingsResponse::new(setting))
    }

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

    #[must_use]
    pub fn ecu_reset(reset_type: ResetType, power_down_time: u8) -> Self {
        Response::EcuReset(EcuResetResponse::new(reset_type, power_down_time))
    }

    #[must_use]
    pub fn negative_response(request_service: UdsServiceType, nrc: NegativeResponseCode) -> Self {
        Response::NegativeResponse(NegativeResponse::new(request_service, nrc))
    }

    #[must_use]
    pub fn read_data_by_identifier<I>(payload: I) -> Self
    where
        I: IntoIterator<Item = D::DiagnosticPayload>,
    {
        Response::ReadDataByIdentifier(ReadDataByIdentifierResponse::new(payload))
    }

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

    #[must_use]
    pub fn request_file_transfer() -> Self {
        todo!()
    }

    pub fn routine_control(
        routine_control_type: crate::RoutineControlSubFunction,
        data: D::RoutinePayload,
    ) -> Self {
        Response::RoutineControl(RoutineControlResponse::new(routine_control_type, data))
    }

    #[must_use]
    pub fn security_access(access_type: SecurityAccessType, security_seed: Vec<u8>) -> Self {
        Response::SecurityAccess(SecurityAccessResponse::new(access_type, security_seed))
    }

    #[must_use]
    pub fn tester_present() -> Self {
        Response::TesterPresent(TesterPresentResponse::new())
    }

    #[must_use]
    pub fn transfer_data(block_sequence_counter: u8, data: Vec<u8>) -> Self {
        Response::TransferData(TransferDataResponse::new(block_sequence_counter, data))
    }

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
    #[allow(clippy::too_many_lines)]
    fn decode<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let service = UdsServiceType::response_from_byte(reader.read_u8()?);
        Ok(Some(match service {
            UdsServiceType::CommunicationControl => Self::CommunicationControl(
                CommunicationControlResponse::decode_single_value(reader)?,
            ),
            UdsServiceType::ControlDTCSettings => {
                Self::ControlDTCSettings(ControlDTCSettingsResponse::decode_single_value(reader)?)
            }
            UdsServiceType::DiagnosticSessionControl => Self::DiagnosticSessionControl(
                DiagnosticSessionControlResponse::decode_single_value(reader)?,
            ),
            UdsServiceType::EcuReset => {
                Self::EcuReset(EcuResetResponse::decode_single_value(reader)?)
            }
            UdsServiceType::ReadDataByIdentifier => Self::ReadDataByIdentifier(
                ReadDataByIdentifierResponse::decode_single_value(reader)?,
            ),
            UdsServiceType::ReadDTCInfo => {
                Self::ReadDTCInfo(ReadDTCInfoResponse::decode_single_value(reader)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(RequestDownloadResponse::decode_single_value(reader)?)
            }
            UdsServiceType::RequestFileTransfer => {
                Self::RequestFileTransfer(RequestFileTransferResponse::decode_single_value(reader)?)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::RoutineControl => {
                Self::RoutineControl(RoutineControlResponse::decode_single_value(reader)?)
            }
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(SecurityAccessResponse::decode_single_value(reader)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(TesterPresentResponse::decode_single_value(reader)?)
            }
            UdsServiceType::NegativeResponse => {
                Self::NegativeResponse(NegativeResponse::decode_single_value(reader)?)
            }
            UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(
                WriteDataByIdentifierResponse::decode_single_value(reader)?,
            ),
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
                Self::TransferData(TransferDataResponse::decode_single_value(reader)?)
            }
            UdsServiceType::UnsupportedDiagnosticService => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::UnsupportedDiagnosticService,
                ));
            }
        }))
    }

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

impl<D: DiagnosticDefinition> SingleValueWireFormat for Response<D> {}
