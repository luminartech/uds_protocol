#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![warn(clippy::pedantic, missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
pub use error::Error;

mod traits;
pub use traits::{
    Decode, DecodeIter, DiagnosticDefinition, DiagnosticDefinitionTx, Encode, Identifier,
    IterableWireFormat, RoutineIdentifier, SingleValueWireFormat, WireFormat,
};

mod common;
pub use common::*;

mod protocol_definitions;
pub use protocol_definitions::{
    ProtocolIdentifier, ProtocolPayload, ProtocolPayloadTx, ProtocolRoutinePayload,
    ProtocolRoutinePayloadTx,
};

mod request;
pub use request::{Request, RequestRx};

mod response;
pub use response::{Response, ResponseRx, UdsResponse, UdsResponseRx};

mod service;
pub use service::UdsServiceType;

mod services;
pub use services::*;

/// UDS positive-response service-ID offset. Added to the request SID to form the response SID.
pub const SUCCESS: u8 = 0x80;
/// UDS `requestCorrectlyReceivedResponsePending` negative response code (`0x78`).
/// Signals that the server received the request but needs additional time to process it.
pub const PENDING: u8 = 0x78;

/// Basic UDS implementation of the [`DiagnosticDefinition`] trait.
///
/// This is an example of a simple data spec that can be used with UDS requests and responses.
/// It should **not** be used directly in production code, but rather as a base for more complex data specifiers.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UdsSpec;
impl DiagnosticDefinition for UdsSpec {
    type RID = UDSRoutineIdentifier;
    type DID = ProtocolIdentifier;
    type RoutinePayload = ProtocolRoutinePayload;
    type DiagnosticPayload = ProtocolPayload;
}

impl DiagnosticDefinitionTx for UdsSpec {
    type RID = UDSRoutineIdentifier;
    type DID = ProtocolIdentifier;
    type RoutinePayload = ProtocolRoutinePayloadTx<'static>;
    type DiagnosticPayload = ProtocolPayloadTx<'static>;
}

/// Type alias for a UDS Request type that only implements the messages explicitly defined by the UDS specification.
pub type ProtocolRequest = Request<UdsSpec>;

/// Type alias for a UDS Response type that only implements the messages explicitly defined by the UDS specification.
pub type ProtocolResponse = Response<UdsSpec>;

/// What type of routine control to perform for a [`RoutineControlRequest`].
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RoutineControlSubFunction {
    /// Routine will be started sometime between completion of the `StartRoutine` request and the completion of the 1st response message
    /// which indicates that the routine has already been performed, or is in progress
    ///
    /// It might be necessary to switch the server to a specific Diagnostic Session via [`DiagnosticSessionControlRequest`] before starting the routine,
    /// or unlock the server using [`SecurityAccessRequest`] before starting the routine.
    StartRoutine,

    /// The server routine shall be stopped in the server's memory sometime between the completion of the `StopRoutine` request and the completion of the 1st response message
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

impl TryFrom<u8> for RoutineControlSubFunction {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(RoutineControlSubFunction::StartRoutine),
            0x02 => Ok(RoutineControlSubFunction::StopRoutine),
            0x03 => Ok(RoutineControlSubFunction::RequestRoutineResults),
            _ => Err(Error::IncorrectMessageLengthOrInvalidFormat),
        }
    }
}

#[cfg(feature = "alloc")]
impl WireFormat for Vec<u8> {
    fn required_size(&self) -> usize {
        self.len()
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_all(self)?;
        Ok(self.len())
    }
}

#[cfg(feature = "alloc")]
impl SingleValueWireFormat for Vec<u8> {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(data)
    }
}

#[cfg(feature = "alloc")]
impl IterableWireFormat for Vec<u8> {
    fn decode_next<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Some(data))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
/// Controls whether the server should enable or disable DTC status-bit updates.
///
/// Used by [`ControlDTCSettingsRequest`] to instruct the server.
pub enum DtcSettings {
    /// Re-enable DTC status-bit updates.
    On,
    /// Disable DTC status-bit updates.
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

impl TryFrom<u8> for DtcSettings {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::On),
            0x02 => Ok(Self::Off),
            _ => Err(Error::IncorrectMessageLengthOrInvalidFormat),
        }
    }
}
