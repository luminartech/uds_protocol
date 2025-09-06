use crate::{Error, Identifier, IterableWireFormat, UDSIdentifier, WireFormat};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use utoipa::ToSchema;

/// Protocol Identifier provides an implementation of Diagnostics Identifiers that only supports Diagnostic Identifiers defined by UDS
#[derive(
    Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Identifier, utoipa::ToSchema,
)]
pub struct ProtocolIdentifier {
    identifier: UDSIdentifier,
}

impl ProtocolIdentifier {
    #[must_use]
    pub fn new(identifier: UDSIdentifier) -> Self {
        ProtocolIdentifier { identifier }
    }

    pub fn identifiers<I>(list: I) -> Vec<Self>
    where
        I: IntoIterator<Item = UDSIdentifier>,
    {
        list.into_iter().map(Self::new).collect()
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

/// The UDS protocol does not define the structure of any payload, so this struct will always return an error when attempting to read from a reader
/// It cannot be constructed, and therefore the write method is unreachable
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, ToSchema)]
#[non_exhaustive]
pub struct ProtocolPayload([u8; 2], Vec<u8>);

impl WireFormat for ProtocolPayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        }
        // Reads the entire payload, but does not have the ability to determine the amount of bytes to read
        // depending on the Identifier, so all data is read until EOF
        let mut entire_payload: Vec<u8> = Vec::new();
        reader.read_to_end(&mut entire_payload)?;
        Ok(Some(ProtocolPayload(identifier_data, entire_payload)))
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

impl std::fmt::Debug for ProtocolPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // show the hex value
        write!(
            f,
            "{:#06X} => {}",
            u16::from(self.0[1]) | u16::from(self.0[0]) << 8,
            self.1
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}
