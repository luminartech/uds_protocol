use crate::{
    CommunicationControlResponse, CommunicationControlType, ControlDTCSettingsResponse,
    DiagnosticSessionControlResponse, DiagnosticSessionType, DtcSettings, EcuResetResponse, Error,
    Identifier, IterableWireFormat, NegativeResponse, NegativeResponseCode, ReadDTCInfoResponse,
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

#[derive(Clone, Debug, PartialEq)]
pub enum Response<RoutineIdentifier, UserIdentifier, RoutinePayload, UserPayload> {
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
    ReadDataByIdentifier(ReadDataByIdentifierResponse<UserPayload>),
    ReadDTCInfo(ReadDTCInfoResponse<UserPayload>),
    /// Response to a [`RequestDownload`](crate::Request::RequestDownload)
    RequestDownload(RequestDownloadResponse),
    RequestFileTransfer(RequestFileTransferResponse),
    /// Response to a [`RequestTransferExit`](crate::Request::RequestTransferExit)
    RequestTransferExit,
    /// Response to a [`RoutineControl` request](crate::RoutineControlRequest)
    RoutineControl(RoutineControlResponse<RoutineIdentifier, RoutinePayload>),
    SecurityAccess(SecurityAccessResponse),
    TesterPresent(TesterPresentResponse),
    TransferData(TransferDataResponse),
    WriteDataByIdentifier(WriteDataByIdentifierResponse<UserIdentifier>),
}

impl<
        RoutineIdentifier: Identifier,
        UserIdentifier: Identifier,
        RoutinePayload: WireFormat,
        UserPayload: IterableWireFormat,
    > Response<RoutineIdentifier, UserIdentifier, RoutinePayload, UserPayload>
{
    pub fn clear_diagnostic_info() -> Self {
        Response::ClearDiagnosticInfo
    }
    pub fn communication_control(control_type: CommunicationControlType) -> Self {
        Response::CommunicationControl(CommunicationControlResponse::new(control_type))
    }

    pub fn control_dtc_settings(setting: DtcSettings) -> Self {
        Response::ControlDTCSettings(ControlDTCSettingsResponse::new(setting))
    }

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

    pub fn ecu_reset(reset_type: ResetType, power_down_time: u8) -> Self {
        Response::EcuReset(EcuResetResponse::new(reset_type, power_down_time))
    }

    pub fn negative_response(request_service: UdsServiceType, nrc: NegativeResponseCode) -> Self {
        Response::NegativeResponse(NegativeResponse::new(request_service, nrc))
    }

    pub fn request_download(
        length_format_identifier: u8,
        max_number_of_block_length: Vec<u8>,
    ) -> Self {
        Response::RequestDownload(RequestDownloadResponse::new(
            length_format_identifier,
            max_number_of_block_length,
        ))
    }

    pub fn request_file_transfer() -> Self {
        todo!()
    }

    pub fn routine_control(
        routine_control_type: crate::RoutineControlSubFunction,
        routine_id: RoutineIdentifier,
        data: RoutinePayload,
    ) -> Self {
        Response::RoutineControl(RoutineControlResponse::new(
            routine_control_type,
            routine_id,
            data,
        ))
    }

    pub fn security_access(access_type: SecurityAccessType, security_seed: Vec<u8>) -> Self {
        Response::SecurityAccess(SecurityAccessResponse::new(access_type, security_seed))
    }

    pub fn tester_present() -> Self {
        Response::TesterPresent(TesterPresentResponse::new())
    }

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

impl<
        RoutineIdentifier: Identifier,
        UserIdentifier: Identifier,
        RoutinePayload: WireFormat,
        UserPayload: IterableWireFormat,
    > WireFormat for Response<RoutineIdentifier, UserIdentifier, RoutinePayload, UserPayload>
{
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let service = UdsServiceType::response_from_byte(reader.read_u8()?);
        Ok(Some(match service {
            UdsServiceType::CommunicationControl => {
                Self::CommunicationControl(CommunicationControlResponse::from_reader(reader)?)
            }
            UdsServiceType::ControlDTCSettings => {
                Self::ControlDTCSettings(ControlDTCSettingsResponse::from_reader(reader)?)
            }
            UdsServiceType::DiagnosticSessionControl => Self::DiagnosticSessionControl(
                DiagnosticSessionControlResponse::from_reader(reader)?,
            ),
            UdsServiceType::EcuReset => Self::EcuReset(EcuResetResponse::from_reader(reader)?),
            UdsServiceType::ReadDataByIdentifier => {
                Self::ReadDataByIdentifier(ReadDataByIdentifierResponse::from_reader(reader)?)
            }
            UdsServiceType::ReadDTCInfo => {
                Self::ReadDTCInfo(ReadDTCInfoResponse::from_reader(reader)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(RequestDownloadResponse::from_reader(reader)?)
            }
            UdsServiceType::RequestFileTransfer => {
                Self::RequestFileTransfer(RequestFileTransferResponse::from_reader(reader)?)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::RoutineControl => {
                Self::RoutineControl(RoutineControlResponse::from_reader(reader)?)
            }
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(SecurityAccessResponse::from_reader(reader)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(TesterPresentResponse::from_reader(reader)?)
            }
            UdsServiceType::NegativeResponse => {
                Self::NegativeResponse(NegativeResponse::from_reader(reader)?)
            }
            UdsServiceType::WriteDataByIdentifier => {
                Self::WriteDataByIdentifier(WriteDataByIdentifierResponse::from_reader(reader)?)
            }
            UdsServiceType::Authentication => todo!(),
            UdsServiceType::AccessTimingParameters => todo!(),
            UdsServiceType::SecuredDataTransmission => todo!(),
            UdsServiceType::ResponseOnEvent => todo!(),
            UdsServiceType::LinkControl => todo!(),
            UdsServiceType::ReadMemoryByAddress => todo!(),
            UdsServiceType::ReadScalingDataByIdentifier => todo!(),
            UdsServiceType::ReadDataByIdentifierPeriodic => todo!(),
            UdsServiceType::DynamicallyDefinedDataIdentifier => todo!(),
            UdsServiceType::WriteMemoryByAddress => todo!(),
            UdsServiceType::ClearDiagnosticInfo => todo!(),
            UdsServiceType::InputOutputControlByIdentifier => todo!(),
            UdsServiceType::RequestUpload => todo!(),
            UdsServiceType::TransferData => {
                Self::TransferData(TransferDataResponse::from_reader(reader)?)
            }
            UdsServiceType::UnsupportedDiagnosticService => todo!(),
        }))
    }

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

    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Write the service byte
        writer.write_u8(self.service().response_to_byte())?;
        // Write the payload
        Ok(1 + match self {
            Self::ClearDiagnosticInfo => Ok(0),
            Self::CommunicationControl(cc) => cc.to_writer(writer),
            Self::ControlDTCSettings(dtc) => dtc.to_writer(writer),
            Self::DiagnosticSessionControl(ds) => ds.to_writer(writer),
            Self::EcuReset(reset) => reset.to_writer(writer),
            Self::NegativeResponse(nr) => nr.to_writer(writer),
            Self::ReadDataByIdentifier(rd) => rd.to_writer(writer),
            Self::ReadDTCInfo(rd) => rd.to_writer(writer),
            Self::RequestDownload(rd) => rd.to_writer(writer),
            Self::RequestFileTransfer(rft) => rft.to_writer(writer),
            Self::RequestTransferExit => Ok(0),
            Self::RoutineControl(rc) => rc.to_writer(writer),
            Self::SecurityAccess(sa) => sa.to_writer(writer),
            Self::TesterPresent(tp) => tp.to_writer(writer),
            Self::TransferData(td) => td.to_writer(writer),
            Self::WriteDataByIdentifier(wdbi) => wdbi.to_writer(writer),
        }?)
    }
}

impl<
        RoutineIdentifier: Identifier,
        UserIdentifier: Identifier,
        RoutinePayload: WireFormat,
        UserPayload: IterableWireFormat,
    > SingleValueWireFormat
    for Response<RoutineIdentifier, UserIdentifier, RoutinePayload, UserPayload>
{
}
