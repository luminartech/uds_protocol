use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum CommunicationType {
    ISOSAEReserved,
    Normal,
    NetworkManagement,
    NormalAndNetworkManagement,
}

impl From<CommunicationType> for u8 {
    fn from(value: CommunicationType) -> Self {
        match value {
            CommunicationType::ISOSAEReserved => 0x00,
            CommunicationType::Normal => 0x01,
            CommunicationType::NetworkManagement => 0x02,
            CommunicationType::NormalAndNetworkManagement => 0x03,
        }
    }
}

impl TryFrom<u8> for CommunicationType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(Self::ISOSAEReserved),
            0x01 => Ok(Self::Normal),
            0x02 => Ok(CommunicationType::NetworkManagement),
            0x03 => Ok(CommunicationType::NormalAndNetworkManagement),
            _ => panic!("Invalid communication type: {value}"),
        }
    }
}
