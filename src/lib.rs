#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README"))]
mod error;
pub use error::Error;

mod request;
pub use request::UdsRequest;

mod services;
pub use services::{
    CommunicationControlRequest, ControlDTCSettingsRequest, DiagnosticSessionControlRequest,
    ReadDataByIdentifier, RequestDownload, RoutineControl, TransferData, WriteDataByIdentifier,
};

mod response;
pub use response::UdsResponse;

mod service;
pub use service::UdsServiceType;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const SUCCESS: u8 = 0x80;
pub const PENDING: u8 = 0x78;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum SessionType {
    Default,
    Programming,
    Extended,
    Safety,
}

impl From<SessionType> for u8 {
    fn from(value: SessionType) -> Self {
        match value {
            SessionType::Default => 0x01,
            SessionType::Programming => 0x02,
            SessionType::Extended => 0x03,
            SessionType::Safety => 0x04,
        }
    }
}

impl From<u8> for SessionType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => SessionType::Default,
            0x02 => SessionType::Programming,
            0x03 => SessionType::Extended,
            0x04 => SessionType::Safety,
            _ => panic!("Invalid session type: {}", value),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum RoutineControlSubFunction {
    StartRoutine,
    StopRoutine,
    RequestRoutineResults,
}

impl From<RoutineControlSubFunction> for u8 {
    fn from(value: RoutineControlSubFunction) -> Self {
        match value {
            RoutineControlSubFunction::StartRoutine => 0x01,
            RoutineControlSubFunction::StopRoutine => 0x02,
            RoutineControlSubFunction::RequestRoutineResults => 0x03,
        }
    }
}

impl From<u8> for RoutineControlSubFunction {
    fn from(value: u8) -> Self {
        match value {
            0x01 => RoutineControlSubFunction::StartRoutine,
            0x02 => RoutineControlSubFunction::StopRoutine,
            0x03 => RoutineControlSubFunction::RequestRoutineResults,
            _ => panic!("Invalid routine control subfunction: {}", value),
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum CommunicationEnable {
    EnableRxAndTx,
    EnableRxAndDisableTx,
    DisableRxAndEnableTx,
    DisableRxAndTx,
}

impl From<CommunicationEnable> for u8 {
    fn from(value: CommunicationEnable) -> Self {
        match value {
            CommunicationEnable::EnableRxAndTx => 0x00,
            CommunicationEnable::EnableRxAndDisableTx => 0x01,
            CommunicationEnable::DisableRxAndEnableTx => 0x02,
            CommunicationEnable::DisableRxAndTx => 0x03,
        }
    }
}

impl From<u8> for CommunicationEnable {
    fn from(value: u8) -> Self {
        match value {
            0x00 => CommunicationEnable::EnableRxAndTx,
            0x01 => CommunicationEnable::EnableRxAndDisableTx,
            0x02 => CommunicationEnable::DisableRxAndEnableTx,
            0x03 => CommunicationEnable::DisableRxAndTx,
            _ => panic!("Invalid communication enable: {}", value),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum CommunicationType {
    Normal,
    NetworkManagement,
    NormalAndNetworkManagement,
}

impl From<CommunicationType> for u8 {
    fn from(value: CommunicationType) -> Self {
        match value {
            CommunicationType::Normal => 0x01,
            CommunicationType::NetworkManagement => 0x02,
            CommunicationType::NormalAndNetworkManagement => 0x03,
        }
    }
}

impl From<u8> for CommunicationType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::Normal,
            0x02 => CommunicationType::NetworkManagement,
            0x03 => CommunicationType::NormalAndNetworkManagement,
            _ => panic!("Invalid communication type: {}", value),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum DtcSettings {
    On,
    Off,
}

impl From<DtcSettings> for u8 {
    fn from(value: DtcSettings) -> Self {
        match value {
            DtcSettings::On => 0x01,
            DtcSettings::Off => 0x02,
        }
    }
}

impl From<u8> for DtcSettings {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::On,
            0x02 => Self::Off,
            _ => panic!("Invalid DTC setting: {}", value),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum EcuResetType {
    HardReset,
}

impl From<EcuResetType> for u8 {
    fn from(value: EcuResetType) -> Self {
        match value {
            EcuResetType::HardReset => 0x01,
        }
    }
}

impl From<u8> for EcuResetType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::HardReset,
            _ => panic!("Invalid ECU reset type: {}", value),
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
enum SecurityAccessOperation {
    RequestSeed,
    SendKey,
}

impl From<SecurityAccessOperation> for u8 {
    fn from(value: SecurityAccessOperation) -> Self {
        match value {
            SecurityAccessOperation::RequestSeed => 0x01,
            SecurityAccessOperation::SendKey => 0x02,
        }
    }
}

impl From<u8> for SecurityAccessOperation {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::RequestSeed,
            0x02 => Self::SendKey,
            _ => panic!("Invalid security access operation: {}", value),
        }
    }
}
