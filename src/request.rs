//! Module for making and handling UDS Requests
use crate::{
    services::{
        CommunicationControlRequest, ControlDTCSettingsRequest, DiagnosticSessionControlRequest,
        EcuResetRequest, ReadDataByIdentifierRequest, RequestDownloadRequest,
        RoutineControlRequest, SecurityAccessRequest, TesterPresentRequest, TransferDataRequest,
        WriteDataByIdentifierRequest,
    },
    Error, NegativeResponseCode, ResetType, SecurityAccessType,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use super::{
    service::UdsServiceType, CommunicationControlType, CommunicationType, DiagnosticSessionType,
    DtcSettings, RoutineControlSubFunction,
};

/// UDS Request types
/// Each variant corresponds to a request for a different UDS service
/// The variants contain all request data for each service
pub enum Request {
    CommunicationControl(CommunicationControlRequest),
    ControlDTCSettings(ControlDTCSettingsRequest),
    DiagnosticSessionControl(DiagnosticSessionControlRequest),
    EcuReset(EcuResetRequest),
    ReadDataByIdentifier(ReadDataByIdentifierRequest),
    RequestDownload(RequestDownloadRequest),
    RequestTransferExit,
    RoutineControl(RoutineControlRequest),
    SecurityAccess(SecurityAccessRequest),
    TesterPresent(TesterPresentRequest),
    TransferData(TransferDataRequest),
    WriteDataByIdentifier(WriteDataByIdentifierRequest),
}

impl Request {
    /// Create a communication control request
    pub fn communication_control(
        communication_enable: CommunicationControlType,
        communication_type: CommunicationType,
        suppress_response: bool,
    ) -> Self {
        Request::CommunicationControl(CommunicationControlRequest::new(
            communication_enable,
            communication_type,
            suppress_response,
        ))
    }

    /// Create a new ControlDTCSettings request
    pub fn control_dtc_settings(setting: DtcSettings, suppress_response: bool) -> Self {
        Request::ControlDTCSettings(ControlDTCSettingsRequest::new(setting, suppress_response))
    }

    pub fn diagnostic_session_control(
        suppress_positive_response: bool,
        session_type: DiagnosticSessionType,
    ) -> Self {
        Request::DiagnosticSessionControl(DiagnosticSessionControlRequest::new(
            suppress_positive_response,
            session_type,
        ))
    }

    pub fn ecu_reset(suppress_positive_response: bool, reset_type: ResetType) -> Self {
        Request::EcuReset(EcuResetRequest::new(suppress_positive_response, reset_type))
    }

    pub fn read_data_by_identifier(did: u16) -> Self {
        Request::ReadDataByIdentifier(ReadDataByIdentifierRequest::new(did))
    }

    // TODO:: Figure out if the format and length identifiers should be configurable
    pub fn request_download(memory_address: u32, memory_size: u32) -> Self {
        Request::RequestDownload(RequestDownloadRequest::new(
            0x00,
            0x44,
            memory_address,
            memory_size,
        ))
    }

    pub fn request_transfer_exit() -> Self {
        Self::RequestTransferExit
    }

    pub fn routine_control(
        sub_function: RoutineControlSubFunction,
        routine_id: u16,
        data: Vec<u8>,
    ) -> Self {
        Request::RoutineControl(RoutineControlRequest::new(sub_function, routine_id, data))
    }

    pub fn security_access(
        suppress_positive_response: bool,
        access_type: SecurityAccessType,
        data_record: Vec<u8>,
    ) -> Self {
        Request::SecurityAccess(SecurityAccessRequest::new(
            suppress_positive_response,
            access_type,
            data_record,
        ))
    }

    pub fn tester_present() -> Self {
        Request::TesterPresent(TesterPresentRequest::new())
    }

    pub fn transfer_data(sequence: u8, data: Vec<u8>) -> Self {
        Request::TransferData(TransferDataRequest::new(sequence, data))
    }

    pub fn write_data_by_identifier(did: u16, data: Vec<u8>) -> Self {
        Request::WriteDataByIdentifier(WriteDataByIdentifierRequest::new(did, data))
    }

    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::CommunicationControl(_) => UdsServiceType::CommunicationControl,
            Self::ControlDTCSettings(_) => UdsServiceType::ControlDTCSettings,
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::ReadDataByIdentifier(_) => UdsServiceType::ReadDataByIdentifier,
            Self::RequestDownload(_) => UdsServiceType::RequestDownload,
            Self::RequestTransferExit => UdsServiceType::RequestTransferExit,
            Self::RoutineControl(_) => UdsServiceType::RoutineControl,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent(_) => UdsServiceType::TesterPresent,
            Self::TransferData(_) => UdsServiceType::TransferData,
            Self::WriteDataByIdentifier(_) => UdsServiceType::WriteDataByIdentifier,
        }
    }

    pub fn allowed_nack_codes(&self) -> &'static [NegativeResponseCode] {
        match self {
            Self::DiagnosticSessionControl(_) => {
                DiagnosticSessionControlRequest::allowed_nack_codes()
            }
            Self::EcuReset(_) => EcuResetRequest::allowed_nack_codes(),
            Self::SecurityAccess(_) => SecurityAccessRequest::allowed_nack_codes(),
            _ => &[NegativeResponseCode::ServiceNotSupported],
        }
    }

    /// Deserialization function to read a [`Request`] from a [`Reader`](std::io::Read)
    /// This function reads the service byte and then calls the appropriate
    /// deserialization function for the service in question
    ///
    /// *Note*:
    ///
    /// Some services allow for custom byte arrays at the end of the request
    /// It is important that only the request data is passed to this function
    /// or the deserialization could read unexpected data
    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self, Error> {
        let service = UdsServiceType::service_from_request_byte(reader.read_u8()?);
        Ok(match service {
            UdsServiceType::CommunicationControl => {
                Self::CommunicationControl(CommunicationControlRequest::read(reader)?)
            }
            UdsServiceType::ControlDTCSettings => {
                Self::ControlDTCSettings(ControlDTCSettingsRequest::read(reader)?)
            }
            UdsServiceType::DiagnosticSessionControl => {
                Self::DiagnosticSessionControl(DiagnosticSessionControlRequest::read(reader)?)
            }
            UdsServiceType::EcuReset => Self::EcuReset(EcuResetRequest::read(reader)?),
            UdsServiceType::ReadDataByIdentifier => {
                Self::ReadDataByIdentifier(ReadDataByIdentifierRequest::read(reader)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(RequestDownloadRequest::read(reader)?)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::RoutineControl => {
                Self::RoutineControl(RoutineControlRequest::read(reader)?)
            }
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(SecurityAccessRequest::read(reader)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(TesterPresentRequest::read(reader)?)
            }
            UdsServiceType::TransferData => Self::TransferData(TransferDataRequest::read(reader)?),
            UdsServiceType::WriteDataByIdentifier => {
                Self::WriteDataByIdentifier(WriteDataByIdentifierRequest::read(reader)?)
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
            UdsServiceType::ReadDTCInfo => todo!(),
            UdsServiceType::InputOutputControlByIdentifier => todo!(),
            UdsServiceType::RequestUpload => todo!(),
            UdsServiceType::RequestFileTransfer => todo!(),
            UdsServiceType::NegativeResponse => todo!(),
            UdsServiceType::UnsupportedDiagnosticService => todo!(),
        })
    }

    pub fn to_writer<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
        // Write the service byte
        writer.write_u8(self.service().request_service_to_byte())?;
        // Write the payload
        match self {
            Self::CommunicationControl(cc) => cc.write(writer),
            Self::ControlDTCSettings(ct) => ct.write(writer),
            Self::DiagnosticSessionControl(ds) => ds.write(writer),
            Self::EcuReset(er) => er.write(writer),
            Self::ReadDataByIdentifier(rd) => rd.write(writer),
            Self::RequestDownload(rd) => rd.write(writer),
            Self::RequestTransferExit => Ok(()),
            Self::RoutineControl(rc) => rc.write(writer),
            Self::SecurityAccess(sa) => sa.write(writer),
            Self::TesterPresent(tp) => tp.write(writer),
            Self::TransferData(td) => td.write(writer),
            Self::WriteDataByIdentifier(wd) => wd.write(writer),
        }
    }
}
