use crate::Error;
use byteorder::{BigEndian, WriteBytesExt};
use serde::Serialize;

/// A trait for types that can be deserialized from a
/// [`Reader`](https://doc.rust-lang.org/std/io/trait.Read.html) and serialized
/// to a [`Writer`](https://doc.rust-lang.org/std/io/trait.Write.html).
///
/// `WireFormat` acts as the base trait for all types that can be serialized and deserialized
/// as part of the UDS Protocol ecosystem.
///
/// Some types need the ability to be deserialized without knowing the size of the data in advance.
/// To support this, the `option_from_reader` function returns an `Option<Self>`.
/// If the reader contains a complete value, it returns `Some(value)`.
/// If the reader is completely empty, it returns `None`.
/// Many types will never return `None`, and for these types, the `SingleValueWireFormat`,
/// trait can be implemented, providing a more ergonomic API.
pub trait WireFormat: Sized {
    /// Deserialize a value from a byte stream.
    /// Returns Ok(`Some(value)`) if the stream contains a complete value.
    /// Returns Ok(`None`) if the stream is empty.
    /// # Errors
    /// - if the stream is not in the expected format
    /// - if the stream contains partial data
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error>;

    /// Returns the number of bytes required to serialize this value.
    fn required_size(&self) -> usize;

    /// Serialize a value to a byte stream.
    /// Returns the number of bytes written.
    /// # Errors
    /// - If the data cannot be written to the stream
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error>;
}

struct WireFormatIterator<'a, T, R> {
    reader: &'a mut R,
    _phantom: std::marker::PhantomData<T>,
}

/// For types that can appear in lists of unknown length, this trait provides an iterator
/// that can be used to deserialize a stream of values.
impl<T: WireFormat, R: std::io::Read> Iterator for WireFormatIterator<'_, T, R> {
    type Item = Result<T, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match T::option_from_reader(self.reader.by_ref()) {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

pub trait IterableWireFormat: WireFormat {
    fn from_reader_iterable<T: std::io::Read>(
        reader: &mut T,
    ) -> impl Iterator<Item = Result<Self, Error>> {
        WireFormatIterator {
            reader,
            _phantom: std::marker::PhantomData,
        }
    }
}

pub trait SingleValueWireFormat: WireFormat {
    fn from_reader<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        Ok(Self::option_from_reader(reader)?.expect(
            "SingleValueWireFormat is only valid to implement on types which never return none",
        ))
    }
}

/// Trait for types that can be used as identifiers (ie Data Identifiers and Routine Identifiers)
///
/// Prefer using the [`#[derive(Identifier)]`](uds_protocol_derive::Identifier) derive macro to implement this trait
pub trait Identifier: TryFrom<u16> + Into<u16> + Clone + Copy + Serialize {
    /// Returns a Vec<Self> from a reader that contains a list of Identifier values
    fn parse_from_list<R: std::io::Read>(reader: &mut R) -> Result<Vec<Self>, Error> {
        // Create an iterator to collect. Will use the blanket implementation of WireFormat for Identifier
        // to read the values from the reader
        WireFormatIterator {
            reader,
            _phantom: std::marker::PhantomData,
        }
        .collect()
    }

    /// Intended to be used in a payload where the identifier is the first value and not a list of identifiers
    /// IE DataIdentifer (DID) payloads and RoutineIdentifier (RID) payloads
    ///
    /// Returns the identifier, or None if the reader is empty
    ///
    /// ## Example reading a payload that has multiple identifiers
    /// ```rust,ignore
    /// while let Some(identifier) = MyIdentifier::parse_from_payload(&mut buffer).unwrap() {
    ///     match identifier {
    ///        MyIdentifier::Identifier1 | MyIdentifier::Identifier2 => {
    ///           let payload = MyPayload::from_reader(&mut buffer).unwrap();
    ///         }
    ///        // No payload for Identifier3
    ///        MyIdentifier::MyIdentifier3 => (),
    ///        MyIdentifier::UDSIdentifier(_) => (),
    ///     }
    /// }
    /// ```
    fn parse_from_payload<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        Self::option_from_reader(reader)
    }
}

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

/// A trait that defines the user-defined diagnostic definitions/specifiers for UDS requests and responses.
///
/// Used to specify the types of the identifiers and payloads used in UDS requests and responses.
/// It allows for flexibility in defining custom data types while adhering to the UDS protocol.
pub trait DiagnosticDefinition {
    /// UDS Data Identifier
    ///
    /// Requests : [ReadDataByIdentifierRequest], [WriteDataByIdentifierRequest], and [ReadDTCInfoRequest]
    /// Responses: [ReadDataByIdentifierResponse], [WriteDataByIdentifierResponse], and [ReadDTCInfoResponse]
    type DID: Identifier;
    /// Response payload for [ReadDataByIdentifierRequest]
    type DiagnosticPayload: IterableWireFormat;

    /// UDS Routine Identifier
    ///
    /// This is used to identify the routine to be controlled in a [RoutineControlRequest]
    type RID: Identifier;
    /// Payload for both requests and responses of [RoutineControlRequest] and [RoutineControlResponse]
    type RoutinePayload: WireFormat;
}

/// tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Identifier, UDSIdentifier};
    use byteorder::ReadBytesExt;
    use std::io::Cursor;

    #[derive(Clone, Copy, Serialize, PartialEq, Eq, Debug, Identifier)]
    #[repr(u16)]
    pub enum MyIdentifier {
        Identifier1 = 0x0101,
        Identifier2 = 0x0202,
        Identifier3 = 0x0303,
        UDSIdentifier(UDSIdentifier),
    }

    impl From<u16> for MyIdentifier {
        fn from(value: u16) -> Self {
            match value {
                0x0101 => MyIdentifier::Identifier1,
                0x0202 => MyIdentifier::Identifier2,
                0x0303 => MyIdentifier::Identifier3,
                _ => MyIdentifier::UDSIdentifier(UDSIdentifier::try_from(value).unwrap()),
            }
        }
    }

    impl From<MyIdentifier> for u16 {
        fn from(value: MyIdentifier) -> Self {
            match value {
                MyIdentifier::Identifier1 => 0x0101,
                MyIdentifier::Identifier2 => 0x0202,
                MyIdentifier::Identifier3 => 0x0303,
                MyIdentifier::UDSIdentifier(identifier) => u16::from(identifier),
            }
        }
    }

    #[derive(Debug)]
    pub struct MyPayload {
        identifier: MyIdentifier,
        u8_value: u8,
    }

    #[test]
    fn test_identifier() {
        let mut buffer = Cursor::new(vec![0u8; 2]);
        let identifier = MyIdentifier::Identifier1;
        identifier.to_writer(&mut buffer).unwrap();
        buffer.set_position(0);
        let read_identifier = MyIdentifier::parse_from_list(&mut buffer).unwrap();
        assert_eq!(identifier, read_identifier[0]);
    }

    #[test]
    fn test_payload() {
        let mut buffer = Cursor::new(vec![0x01, 0x01, 0xFF, 0x02, 0x02, 0xFF, 0x03, 0x03]);
        // Read until the end of the buffer
        while let Some(identifier) = MyIdentifier::parse_from_payload(&mut buffer).unwrap() {
            match identifier {
                MyIdentifier::Identifier1 | MyIdentifier::Identifier2 => {
                    let payload = MyPayload {
                        identifier,
                        u8_value: buffer.read_u8().unwrap(),
                    };
                    assert!(matches!(
                        payload.identifier,
                        MyIdentifier::Identifier1 | MyIdentifier::Identifier2
                    ));
                    assert_eq!(payload.u8_value, 0xFF);
                }
                MyIdentifier::Identifier3 => (),
                MyIdentifier::UDSIdentifier(_) => (),
            }
        }
        println!("Testing printing");
    }
}
