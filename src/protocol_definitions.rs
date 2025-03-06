use crate::{iterable_identifier, Error, IterableWireFormat, UDSIdentifier, WireFormat};
use byteorder::{BigEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Trait for types that can be used as identifiers (ie Data Identifiers and Routine Identifiers)
pub trait Identifier: TryFrom<u16> + Into<u16> {}

/// Impl Identifier for all types that implement TryFrom<u16> and Into<u16>
impl<T> Identifier for T where T: TryFrom<u16> + Into<u16> {}

/// Protocol Identifier provides an implementation of Diagnostics Identifiers that only supports Diagnostic Identifiers defined by UDS
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProtocolIdentifier {
    identifier: UDSIdentifier,
}

impl ProtocolIdentifier {
    pub fn new(identifier: UDSIdentifier) -> Self {
        ProtocolIdentifier { identifier }
    }
}

impl TryFrom<u16> for ProtocolIdentifier {
    type Error = Error;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self {
            identifier: UDSIdentifier::try_from(value)?,
        })
    }
}

impl From<ProtocolIdentifier> for u16 {
    fn from(value: ProtocolIdentifier) -> Self {
        u16::from(value.identifier)
    }
}

impl Deref for ProtocolIdentifier {
    type Target = UDSIdentifier;
    fn deref(&self) -> &UDSIdentifier {
        &self.identifier
    }
}

// Implementing the WireFormat and IterableWireFormat traits for ProtocolIdentifier
iterable_identifier!(ProtocolIdentifier);

/// The UDS protocol does not define the structure of any payload, so this struct will always return an error when attempting to read from a reader
/// It cannot be constructed, and therefore the write method is unreachable
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ProtocolPayload;

impl WireFormat for ProtocolPayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        };
        Err(Error::InvalidDiagnosticIdentifier(u16::from_be_bytes(
            identifier_data,
        )))
    }

    fn required_size(&self) -> usize {
        0
    }

    fn to_writer<T: std::io::Write>(&self, _: &mut T) -> Result<usize, Error> {
        unreachable!(
            "This type cannot be constructed, and therefore cannot be serialized to a writer."
        )
    }
}

impl IterableWireFormat for ProtocolPayload {}
