//! Module for making and handling UDS Requests

use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::Error;

use super::{
    service::UdsServiceType, CommunicationEnable, CommunicationType, DtcSettings, EcuResetType,
    RoutineControlSubFunction, SessionType, SUCCESS,
};

pub struct CommunicationControl {
    pub communication_enable: CommunicationEnable,
    pub communication_type: CommunicationType,
    pub suppress_response: bool,
    /// Stop external code from creating instances of this struct directly
    _private: (),
}

impl CommunicationControl {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let enable_byte = buffer.read_u8()?;
        let communication_enable = CommunicationEnable::from(enable_byte & !SUCCESS);
        let suppress_response = enable_byte & SUCCESS == SUCCESS;
        let communication_type = CommunicationType::from(buffer.read_u8()?);
        Ok(Self {
            communication_enable,
            communication_type,
            suppress_response,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        let communication_enable_byte =
            u8::from(self.communication_enable) | if self.suppress_response { SUCCESS } else { 0 };
        buffer.write_u8(communication_enable_byte)?;
        buffer.write_u8(u8::from(self.communication_type))?;
        Ok(())
    }
}

/// The ControlDTCSettings service is used to control the DTC settings of the ECU.
#[derive(Clone, Copy, Debug)]
pub struct ControlDTCSettings {
    /// The requested DTC logging setting
    pub setting: DtcSettings,
    /// Whether the ECU should suppress a response
    pub suppress_response: bool,
    /// Stop external code from creating instances of this struct directly
    _private: (),
}

impl ControlDTCSettings {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let request_byte = buffer.read_u8()?;
        let setting = DtcSettings::from(request_byte & !SUCCESS);
        let suppress_response = request_byte & SUCCESS != 0;
        Ok(Self {
            setting,
            suppress_response,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        let request_byte =
            u8::from(self.setting) | if self.suppress_response { SUCCESS } else { 0 };
        buffer.write_u8(request_byte)?;
        Ok(())
    }
}

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
pub struct TransferData {
    pub sequence: u8,
    pub data: Vec<u8>,
    _private: (),
}


pub struct ReadDataByIdentifier {
    pub did: u16,
    _private: (),
}

impl ReadDataByIdentifier {
    fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let did = buffer.read_u16::<BigEndian>()?;
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
    fn new(memory_address: u32, memory_size: u32) -> Self {
        Self {
            data_format_identifier: 0x00,
            address_and_length_format_identifier: 0x44,
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
    RequestTransferExit(RequestTransferExit),
    RoutineControl(RoutineControl),
    TesterPresent(TesterPresent),
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
        UdsRequestType::RequestTransferExit(RequestTransferExit { _private: () })
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
        UdsRequestType::TesterPresent(TesterPresent { _private: () })
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
}
