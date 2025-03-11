//! Blanket/Common types and traits for identifiers (Data Identifiers and Routine Identifiers)
use crate::{Error, WireFormat};
use byteorder::{BigEndian, WriteBytesExt};
use serde::Serialize;

/// Trait for types that can be used as identifiers (ie Data Identifiers and Routine Identifiers)
///
/// Prefer using the [`#[derive(Identifier)]`](uds_protocol_derive::Identifier) derive macro to implement this trait
pub trait Identifier: TryFrom<u16> + Into<u16> + Clone + Copy + Serialize {}

/// Blanket implementation of the [WireFormat] trait for types that implement the [Identifier] trait
impl<T> WireFormat for T
where
    T: Identifier,
{
    fn option_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        };

        match Self::try_from(u16::from_be_bytes(identifier_data)) {
            Ok(identifier) => Ok(Some(identifier)),
            Err(_) => Err(Error::InvalidDiagnosticIdentifier(u16::from_be_bytes(
                identifier_data,
            ))),
        }
    }

    fn required_size(&self) -> usize {
        2
    }

    fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        writer.write_u16::<BigEndian>((*self).into())?;
        Ok(2)
    }
}
