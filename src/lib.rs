#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

mod common;
pub use common::*;

mod error;
pub use error::Error;

mod request;
pub use request::Request;

mod services;
pub use services::*;

mod response;
pub use response::{Response, UdsResponse};

mod service;
pub use service::UdsServiceType;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const SUCCESS: u8 = 0x80;
pub const PENDING: u8 = 0x78;

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
            _ => panic!("Invalid routine control subfunction: {value}"),
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
            _ => panic!("Invalid DTC setting: {value}"),
        }
    }
}
