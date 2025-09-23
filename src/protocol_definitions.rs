use tracing::error;

use crate::{Error, Identifier, IterableWireFormat, UDSIdentifier, WireFormat};
use std::ops::Deref;

/// Protocol Identifier provides an implementation of Diagnostics Identifiers that only supports Diagnostic Identifiers defined by UDS
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, Identifier, PartialEq)]
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

/// The UDS protocol does not define the structure of any payload, but exists as a container for diagnostic implementations that use the generic UDS identifiers
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct ProtocolPayload {
    pub identifier: UDSIdentifier,
    pub payload: Vec<u8>,
}

impl ProtocolPayload {
    /// Creates a new `ProtocolPayload` with the given identifier and payload
    #[must_use]
    pub fn new(identifier: UDSIdentifier, payload: Vec<u8>) -> Self {
        ProtocolPayload {
            identifier,
            payload,
        }
    }
}
impl WireFormat for ProtocolPayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => {
                error!(
                    "Only read 1 byte of identifier, need 2: read byte was: {}",
                    identifier_data[0]
                );
                return Err(Error::IncorrectMessageLengthOrInvalidFormat);
            }
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        }
        let identifier = UDSIdentifier::try_from(u16::from_be_bytes(identifier_data))?;
        // Reads the entire payload, but does not have the ability to determine the amount of bytes to read
        // depending on the Identifier, so all data is read until EOF
        //
        // TODO: We could be more clever, we do know the response size of some identifiers
        let mut payload: Vec<u8> = Vec::new();
        reader.read_to_end(&mut payload)?;
        Ok(Some(ProtocolPayload {
            identifier,
            payload,
        }))
    }

    fn required_size(&self) -> usize {
        2 + self.payload.len()
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        self.identifier.to_writer(writer)?;
        writer.write_all(&self.payload)?;
        Ok(self.required_size())
    }
}

impl IterableWireFormat for ProtocolPayload {}

impl std::fmt::Debug for ProtocolPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} => {}",
            self.identifier,
            self.payload
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_construction_and_debug_format() {
        let payload = ProtocolPayload::new(UDSIdentifier::ActiveDiagnosticSession, vec![0x01]);
        assert_eq!(format!("{payload:?}"), "0xF186 => 01");
        let mut buffer = Vec::new();
        assert_eq!(3, payload.to_writer(&mut buffer).unwrap());
    }

    #[test]
    fn test_read_and_write() {
        let payload = ProtocolPayload::new(UDSIdentifier::ActiveDiagnosticSession, vec![0x03]);
        let mut buffer = Vec::new();
        assert_eq!(3, payload.to_writer(&mut buffer).unwrap());
        let deserialized_payload = ProtocolPayload::option_from_reader(&mut buffer.as_slice())
            .unwrap()
            .unwrap();
        assert_eq!(payload, deserialized_payload);
    }
}
