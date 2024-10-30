use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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
