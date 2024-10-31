use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum DiagnosticSessionType {
    Default,
    Programming,
    Extended,
    Safety,
}

impl From<DiagnosticSessionType> for u8 {
    fn from(value: DiagnosticSessionType) -> Self {
        match value {
            DiagnosticSessionType::Default => 0x01,
            DiagnosticSessionType::Programming => 0x02,
            DiagnosticSessionType::Extended => 0x03,
            DiagnosticSessionType::Safety => 0x04,
        }
    }
}

impl From<u8> for DiagnosticSessionType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => DiagnosticSessionType::Default,
            0x02 => DiagnosticSessionType::Programming,
            0x03 => DiagnosticSessionType::Extended,
            0x04 => DiagnosticSessionType::Safety,
            _ => panic!("Invalid session type: {}", value),
        }
    }
}
