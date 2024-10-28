//! Module for making and handling UDS Requests

mod communication_control;
pub use communication_control::CommunicationControl;

mod control_dtc_settings;
pub use control_dtc_settings::ControlDTCSettings;
use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::Error;

use super::{
    service::UdsServiceType, CommunicationEnable, CommunicationType, DtcSettings, EcuResetType,
    RoutineControlSubFunction, SessionType, SUCCESS,
};

pub struct DiagnosticsSessionControl {
    pub session_type: SessionType,
    _private: (),
}

impl DiagnosticsSessionControl {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let session_type = SessionType::from(buffer.read_u8()?);
        Ok(Self {
            session_type,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.session_type))?;
        Ok(())
    }
}

pub struct EcuReset {
    pub reset_type: EcuResetType,
    _private: (),
}

impl EcuReset {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let reset_type = EcuResetType::from(buffer.read_u8()?);
        Ok(Self {
            reset_type,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.reset_type))?;
        Ok(())
    }
}

pub struct ReadDataByIdentifier {
    pub did: u16,
    _private: (),
}

impl ReadDataByIdentifier {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let did = buffer.read_u16::<BigEndian>()?;
        Ok(Self { did, _private: () })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u16::<BigEndian>(self.did)?;
        Ok(())
    }
}

pub struct RequestDownload {
    pub data_format_identifier: u8,
    pub address_and_length_format_identifier: u8,
    pub memory_address: u32,
    pub memory_size: u32,
    _private: (),
}

impl RequestDownload {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let data_format_identifier = buffer.read_u8()?;
        let address_and_length_format_identifier = buffer.read_u8()?;
        let memory_address = buffer.read_u32::<BigEndian>()?;
        let memory_size = buffer.read_u32::<BigEndian>()?;
        Ok(Self {
            data_format_identifier,
            address_and_length_format_identifier,
            memory_address,
            memory_size,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(self.data_format_identifier)?;
        buffer.write_u8(self.address_and_length_format_identifier)?;
        buffer.write_u32::<BigEndian>(self.memory_address)?;
        buffer.write_u32::<BigEndian>(self.memory_size)?;
        Ok(())
    }
}

pub struct RoutineControl {
    pub sub_function: RoutineControlSubFunction,
    pub routine_id: u16,
    pub data: Vec<u8>,
    _private: (),
}

impl RoutineControl {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let sub_function = RoutineControlSubFunction::from(buffer.read_u8()?);
        let routine_id = buffer.read_u16::<BigEndian>()?;
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;
        Ok(Self {
            sub_function,
            routine_id,
            data,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.sub_function))?;
        buffer.write_u16::<BigEndian>(self.routine_id)?;
        buffer.write_all(&self.data)?;
        Ok(())
    }
}

pub struct TransferData {
    pub sequence: u8,
    pub data: Vec<u8>,
    _private: (),
}

impl TransferData {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let sequence = buffer.read_u8()?;
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;
        Ok(Self {
            sequence,
            data,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(self.sequence)?;
        buffer.write_all(&self.data)?;
        Ok(())
    }
}

pub struct WriteDataByIdentifier {
    pub did: u16,
    pub data: Vec<u8>,
    _private: (),
}

impl WriteDataByIdentifier {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let did = buffer.read_u16::<BigEndian>()?;
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;
        Ok(Self {
            did,
            data,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u16::<BigEndian>(self.did)?;
        buffer.write_all(&self.data)?;
        Ok(())
    }
}

pub enum UdsRequestType {
    CommunicationControl(CommunicationControl),
    ControlDTCSettings(ControlDTCSettings),
    DiagnosticSessionControl(DiagnosticsSessionControl),
    EcuReset(EcuReset),
    ReadDataByIdentifier(ReadDataByIdentifier),
    RequestDownload(RequestDownload),
    RequestTransferExit,
    RoutineControl(RoutineControl),
    TesterPresent,
    TransferData(TransferData),
    WriteDataByIdentifier(WriteDataByIdentifier),
}

impl UdsRequestType {
    /// Create a communication control request
    pub fn communication_control(
        communication_enable: CommunicationEnable,
        communication_type: CommunicationType,
        suppress_response: bool,
    ) -> Self {
        UdsRequestType::CommunicationControl(CommunicationControl {
            communication_enable,
            communication_type,
            suppress_response,
            _private: (),
        })
    }

    /// Create a new ControlDTCSettings request
    pub fn control_dtc_settings(setting: DtcSettings, suppress_response: bool) -> Self {
        UdsRequestType::ControlDTCSettings(ControlDTCSettings {
            setting,
            suppress_response,
            _private: (),
        })
    }

    pub fn diagnostic_session_control(session_type: SessionType) -> Self {
        UdsRequestType::DiagnosticSessionControl(DiagnosticsSessionControl {
            session_type,
            _private: (),
        })
    }

    pub fn ecu_reset(reset_type: EcuResetType) -> Self {
        UdsRequestType::EcuReset(EcuReset {
            reset_type,
            _private: (),
        })
    }

    pub fn read_data_by_identifier(did: u16) -> Self {
        UdsRequestType::ReadDataByIdentifier(ReadDataByIdentifier { did, _private: () })
    }

    // TODO:: Figure out if the format and length identifiers should be configurable
    pub fn request_download(memory_address: u32, memory_size: u32) -> Self {
        UdsRequestType::RequestDownload(RequestDownload {
            data_format_identifier: 0x00,
            address_and_length_format_identifier: 0x44,
            memory_address,
            memory_size,
            _private: (),
        })
    }

    pub fn request_transfer_exit() -> Self {
        Self::RequestTransferExit
    }

    pub fn routine_control(
        sub_function: RoutineControlSubFunction,
        routine_id: u16,
        data: Vec<u8>,
    ) -> Self {
        UdsRequestType::RoutineControl(RoutineControl {
            sub_function,
            routine_id,
            data,
            _private: (),
        })
    }

    pub fn tester_present() -> Self {
        Self::TesterPresent
    }

    pub fn transfer_data(sequence: u8, data: Vec<u8>) -> Self {
        UdsRequestType::TransferData(TransferData {
            sequence,
            data,
            _private: (),
        })
    }

    pub fn write_data_by_identifier(did: u16, data: Vec<u8>) -> Self {
        UdsRequestType::WriteDataByIdentifier(WriteDataByIdentifier {
            did,
            data,
            _private: (),
        })
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
            Self::TesterPresent => UdsServiceType::TesterPresent,
            Self::TransferData(_) => UdsServiceType::TransferData,
            Self::WriteDataByIdentifier(_) => UdsServiceType::WriteDataByIdentifier,
        }
    }

    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self, Error> {
        let service = UdsServiceType::service_from_request_byte(reader.read_u8()?);
        Ok(match service {
            UdsServiceType::CommunicationControl => {
                Self::CommunicationControl(CommunicationControl::read(reader)?)
            }
            UdsServiceType::ControlDTCSettings => {
                Self::ControlDTCSettings(ControlDTCSettings::read(reader)?)
            }
            UdsServiceType::DiagnosticSessionControl => {
                Self::DiagnosticSessionControl(DiagnosticsSessionControl::read(reader)?)
            }
            UdsServiceType::EcuReset => Self::EcuReset(EcuReset::read(reader)?),
            UdsServiceType::ReadDataByIdentifier => {
                Self::ReadDataByIdentifier(ReadDataByIdentifier::read(reader)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(RequestDownload::read(reader)?)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::RoutineControl => Self::RoutineControl(RoutineControl::read(reader)?),
            UdsServiceType::TesterPresent => Self::TesterPresent,
            UdsServiceType::TransferData => Self::TransferData(TransferData::read(reader)?),
            UdsServiceType::WriteDataByIdentifier => {
                Self::WriteDataByIdentifier(WriteDataByIdentifier::read(reader)?)
            }
            UdsServiceType::SecurityAccess => todo!(),
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
            Self::TesterPresent => Ok(()),
            Self::TransferData(td) => td.write(writer),
            Self::WriteDataByIdentifier(wd) => wd.write(writer),
        }
    }
}
