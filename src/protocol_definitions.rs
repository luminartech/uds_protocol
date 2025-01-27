use crate::{Error, IterableWireFormat, UDSIdentifier, WireFormat};
use byteorder::WriteBytesExt;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

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

impl Deref for ProtocolIdentifier {
    type Target = UDSIdentifier;
    fn deref(&self) -> &UDSIdentifier {
        &self.identifier
    }
}

impl WireFormat for ProtocolIdentifier {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        };

        // This seems wrong or something it doesn't account for
        let identifier = u16::from_be_bytes(identifier_data);
        Ok(Some(Self {
            identifier: UDSIdentifier::try_from(identifier)?,
        }))
    }

    fn required_size(&self) -> usize {
        2
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u16::<byteorder::BigEndian>(u16::from(self.identifier))?;
        Ok(2)
    }
}

impl IterableWireFormat for ProtocolIdentifier {}

/// The UDS protocol does not define the structure of any payload, so this struct will always return an error when attempting to read from a reader
/// It cannot be constructed, and therefore the write method is unreachable
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
