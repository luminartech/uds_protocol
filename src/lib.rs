#![warn(clippy::pedantic)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
mod common;
pub use common::*;

mod error;
pub use error::Error;

use num_enum::{IntoPrimitive, TryFromPrimitive};
// Export the Identifier derive macro
pub use uds_protocol_derive::Identifier;

mod protocol_definitions;
pub use protocol_definitions::{ProtocolIdentifier, ProtocolPayload};

mod request;
pub use request::Request;

mod response;
pub use response::{Response, UdsResponse};

mod service;
pub use service::UdsServiceType;

mod services;
pub use services::*;

mod traits;
pub use traits::{
    DiagnosticDefinition, Identifier, IterableWireFormat, RoutineIdentifier, SingleValueWireFormat,
    WireFormat,
};

pub const SUCCESS: u8 = 0x80;
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
    type RoutinePayload = ProtocolPayload;
    type DiagnosticPayload = ProtocolPayload;
}

/// Type alias for a UDS Request type that only implements the messages explicitly defined by the UDS specification.
pub type ProtocolRequest = Request<UdsSpec>;

/// Type alias for a UDS Response type that only implements the messages explicitly defined by the UDS specification.
pub type ProtocolResponse = Response<UdsSpec>;

/// What type of routine control to perform for a [`RoutineControlRequest`].
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = crate::Error, constructor = Error::InvalidUDSMessageValue))]
#[repr(u8)]
pub enum RoutineControlSubFunction {
    /// Routine will be started sometime between completion of the `StartRoutine` request and the completion of the 1st response message which indicates that the routine has already been performed, or is in progress
    ///
    /// It might be necessary to switch the server to a specific Diagnostic Session via [`DiagnosticSessionControlRequest`] before starting the routine,
    /// or unlock the server using [`SecurityAccessRequest`] before starting the routine.
    StartRoutine = 0x01,

    /// The server routine shall be stopped in the server's memory sometime between the completion of the `StopRoutine` request and the completion of the 1st response message
    /// which indicates that the routine has already been stopped, or is in progress
    StopRoutine = 0x02,

    /// Request results for the specified routineIdentifier
    RequestRoutineResults = 0x03,
}

impl WireFormat for Vec<u8> {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Ok(Some(data))
    }

    fn required_size(&self) -> usize {
        self.len()
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_all(self)?;
        Ok(self.len())
    }
}

impl SingleValueWireFormat for Vec<u8> {}
impl IterableWireFormat for Vec<u8> {}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[num_enum(error_type(name = crate::Error, constructor = Error::InvalidUDSMessageValue))]
#[repr(u8)]
pub enum DtcSettings {
    On = 0x01,
    Off = 0x02,
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_correct_discriminant() {
        let raw_disciminant: u8 = 0x01;
        let discriminant_from_id: u8 = RoutineControlSubFunction::StartRoutine.into();

        let id_from_discriminant: RoutineControlSubFunction =
            RoutineControlSubFunction::try_from(raw_disciminant).unwrap();

        assert_eq!(discriminant_from_id, raw_disciminant);
        assert_eq!(
            id_from_discriminant,
            RoutineControlSubFunction::StartRoutine
        );
    }

    #[test]
    fn test_incorrect_discriminant() {
        let raw_disciminant: u8 = 0x04;

        let id_from_discriminant = RoutineControlSubFunction::try_from(raw_disciminant)
            .err()
            .unwrap()
            .to_string();

        assert_eq!(
            id_from_discriminant,
            Error::InvalidUDSMessageValue(raw_disciminant).to_string()
        );
    }
}
