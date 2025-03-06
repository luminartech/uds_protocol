#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

mod common;
pub use common::*;

mod error;
pub use error::Error;

mod protocol_definitions;
pub use protocol_definitions::{Identifier, ProtocolIdentifier, ProtocolPayload};

mod request;
pub use request::Request;

mod response;
pub use response::{Response, UdsResponse};

mod service;
pub use service::UdsServiceType;

mod services;
pub use services::*;

mod traits;
pub use traits::{IterableWireFormat, SingleValueWireFormat, WireFormat};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub const SUCCESS: u8 = 0x80;
pub const PENDING: u8 = 0x78;

/// Type alias for a UDS Request type that only implements the messages explicitly defined by the UDS specification.
pub type ProtocolRequest = Request<ProtocolIdentifier, ProtocolPayload>;

/// Type alias for a UDS Response type that only implements the messages explicitly defined by the UDS specification.
pub type ProtocolResponse = Response<ProtocolIdentifier, ProtocolPayload>;

/// What type of routine control to perform for a [RoutineControlRequest].
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum RoutineControlSubFunction {
    /// Routine will be started sometime between completion of the StartRoutine request and the completion of the 1st response message
    /// which indicates that the routine has already been performed, or is in progress
    ///
    /// It might be necessary to switch the server to a specific Diagnostic Session via [DiagnosticSessionControlRequest] before starting the routine,
    /// or unlock the server using [`SecurityAccessRequest`] before starting the routine.
    StartRoutine,

    /// The server routine shall be stopped in the server's memory sometime between the completion of the StopRoutine request and the completion of the 1st response message
    /// which indicates that the routine has already been stopped, or is in progress
    StopRoutine,

    /// Request results for the specified routineIdentifier
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
