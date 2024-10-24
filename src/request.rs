use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::Error;

const POSITIVE_RESPONSE: u8 = 0x80;

use super::{
    service::{UdsService, UdsServiceType},
    CommunicationEnable, CommunicationType, DtcSettings, EcuResetType, RoutineControlSubFunction,
    SessionType, SUCCESS,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct UdsRequest {
    data: Vec<u8>,
}

impl UdsRequest {
    pub fn from_service_type(service_type: UdsRequestType) -> Self {
        match service_type {
            UdsRequestType::CommunicationControl(communication_control) => {
                let suppression_byte = match communication_control.suppress_response {
                    true => POSITIVE_RESPONSE,
                    false => 0x00,
                };
                let enable_byte: u8 = communication_control.communication_enable.into();
                Self {
                    data: vec![
                        communication_control.get_service_type().request_to_byte(),
                        enable_byte | suppression_byte,
                        communication_control.communication_type.into(),
                    ],
                }
            }
            UdsRequestType::ControlDTCSettings(dtc_settings) => {
                let suppression_byte = match dtc_settings.suppress_response {
                    true => POSITIVE_RESPONSE,
                    false => 0x00,
                };
                let dtc_setting_byte: u8 = dtc_settings.setting.into();
                Self {
                    data: vec![
                        dtc_settings.get_service_type().request_to_byte(),
                        dtc_setting_byte | suppression_byte,
                    ],
                }
            }
            UdsRequestType::DiagnosticSessionControl(session_control) => Self {
                data: vec![
                    session_control.get_service_type().request_to_byte(),
                    session_control.session_type.into(),
                ],
            },
            UdsRequestType::EcuReset(ecu_reset) => Self {
                data: vec![
                    ecu_reset.get_service_type().request_to_byte(),
                    ecu_reset.reset_type.into(),
                ],
            },
            UdsRequestType::ReadDataByIdentifier(read_data_by_identifier) => {
                let did_bytes = read_data_by_identifier.did.to_be_bytes();
                Self {
                    data: vec![
                        read_data_by_identifier.get_service_type().request_to_byte(),
                        did_bytes[0],
                        did_bytes[1],
                    ],
                }
            }
            UdsRequestType::RequestDownload(request_download) => {
                let mut data = vec![
                    request_download.get_service_type().request_to_byte(),
                    request_download.data_format_identifier,
                    request_download.address_and_length_format_identifier,
                ];
                data.extend_from_slice(request_download.memory_address.to_be_bytes().as_slice());
                data.extend_from_slice(request_download.memory_size.to_be_bytes().as_slice());
                Self { data }
            }
            UdsRequestType::RequestTransferExit(_) => Self {
                data: vec![UdsServiceType::RequestTransferExit.request_to_byte()],
            },
            UdsRequestType::RoutineControl(routine_control) => {
                let mut data = vec![routine_control.get_service_type().request_to_byte()];
                data.push(routine_control.sub_function.into());
                data.extend_from_slice(&routine_control.routine_id.to_be_bytes());
                data.extend_from_slice(routine_control.data.as_slice());
                Self { data }
            }
            UdsRequestType::TesterPresent(tester_present) => Self {
                data: (vec![tester_present.get_service_type().request_to_byte(), SUCCESS]),
            },
            UdsRequestType::TransferData(transfer_data) => {
                let mut data = vec![transfer_data.get_service_type().request_to_byte()];
                data.push(transfer_data.sequence);
                data.extend_from_slice(transfer_data.data.as_slice());
                Self { data }
            }
            UdsRequestType::WriteDataByIdentifier(write_data_by_identifier) => {
                let mut data = vec![write_data_by_identifier
                    .get_service_type()
                    .request_to_byte()];
                data.extend_from_slice(&write_data_by_identifier.did.to_be_bytes());
                data.extend_from_slice(write_data_by_identifier.data.as_slice());
                Self { data }
            }
        }
    }

    pub fn service_type(&self) -> UdsServiceType {
        UdsServiceType::service_from_request_byte(self.data[0])
    }

    pub fn to_network(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn expected_response_byte(&self) -> u8 {
        self.service_type().response_to_byte()
    }
}

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
        let communication_enable = CommunicationEnable::from(enable_byte & !POSITIVE_RESPONSE);
        let suppress_response = enable_byte & POSITIVE_RESPONSE == POSITIVE_RESPONSE;
        let communication_type = CommunicationType::from(buffer.read_u8()?);
        Ok(Self {
            communication_enable,
            communication_type,
            suppress_response,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        let communication_enable_byte = u8::from(self.communication_enable)
            | if self.suppress_response {
                POSITIVE_RESPONSE
            } else {
                0
            };
        buffer.write_u8(communication_enable_byte)?;
        buffer.write_u8(u8::from(self.communication_type))?;
        Ok(())
    }
}

impl UdsService for CommunicationControl {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::CommunicationControl
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
        let setting = DtcSettings::from(request_byte & !POSITIVE_RESPONSE);
        let suppress_response = request_byte & POSITIVE_RESPONSE != 0;
        Ok(Self {
            setting,
            suppress_response,
            _private: (),
        })
    }
    fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        let request_byte = u8::from(self.setting) | if self.suppress_response { POSITIVE_RESPONSE } else { 0 };
        buffer.write_u8(request_byte)?;
        Ok(())
    }
}

impl UdsService for ControlDTCSettings {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::ControlDTCSettings
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

impl UdsService for DiagnosticsSessionControl {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::DiagnosticSessionControl
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

impl UdsService for EcuReset {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::EcuReset
    }
}

pub struct TransferData {
    pub sequence: u8,
    pub data: Vec<u8>,
    _private: (),
}

impl UdsService for TransferData {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::TransferData
    }
}

pub struct ReadDataByIdentifier {
    pub did: u16,
    _private: (),
}

impl UdsService for ReadDataByIdentifier {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::ReadDataByIdentifier
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
        }
    }
}
impl UdsService for RequestDownload {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::RequestDownload
    }
}

pub struct RequestTransferExit {
    _private: (),
}

impl UdsService for RequestTransferExit {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::RequestTransferExit
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

impl UdsService for RoutineControl {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::RoutineControl
    }
}

pub struct TesterPresent {
    _private: (),
}

impl UdsService for TesterPresent {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::TesterPresent
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

impl UdsService for WriteDataByIdentifier {
    fn get_service_type(&self) -> UdsServiceType {
        UdsServiceType::WriteDataByIdentifier
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
}
