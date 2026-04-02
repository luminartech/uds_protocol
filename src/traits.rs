use crate::Error;
use byteorder_embedded_io::BigEndian;
use byteorder_embedded_io::io::WriteBytesExt;

// ---------------------------------------------------------------------------
// New no_std-compatible traits (TX: Encode, RX: Decode / DecodeIter)
// ---------------------------------------------------------------------------

/// TX-side trait: encode a value into an [`embedded_io::Write`] implementor.
///
/// This is the `no_std` replacement for the encoding half of [`WireFormat`].
pub trait Encode {
    /// Number of bytes this value will write.
    fn encoded_size(&self) -> usize;

    /// Serialize into `writer`, returning the number of bytes written.
    ///
    /// # Errors
    /// Returns [`Error::IoError`] if the writer fails.
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error>;

    /// Whether the positive response for this message is suppressed (SPRMIB).
    fn is_positive_response_suppressed(&self) -> bool {
        false
    }
}

/// RX-side trait: zero-copy decode from a byte slice.
///
/// Implementations borrow directly from the input buffer where possible.
/// Returns the decoded value together with the unconsumed remainder of the
/// buffer.
pub trait Decode<'a>: Sized {
    /// Decode from `buf`, returning `(value, remaining_bytes)`.
    ///
    /// # Errors
    /// Returns an error if `buf` is too short or contains invalid data.
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error>;
}

/// RX-side trait: streaming / iterable zero-copy decode.
///
/// Used for variable-length sequences where the number of items is not known
/// ahead of time. Each call consumes one item and returns the remainder, or
/// `Ok(None)` when the buffer is exhausted.
pub trait DecodeIter<'a>: Sized {
    /// Try to decode the next item from `buf`.
    ///
    /// Returns `Ok(None)` when the buffer is empty (sequence exhausted).
    ///
    /// # Errors
    /// Returns an error if the buffer contains a partial or invalid item.
    fn decode_next(buf: &'a [u8]) -> Result<Option<(Self, &'a [u8])>, Error>;
}

/// Base trait for types that can be serialized to a byte stream.
///
/// `WireFormat` provides the encoding half of the serialization contract.
/// Decoding is split into two separate traits with distinct return types:
/// - [`SingleValueWireFormat`] for types whose decode always produces a value
/// - [`IterableWireFormat`] for types that may return `None` when a stream is exhausted
///
/// This split enforces at compile time the distinction between types that always
/// decode successfully (given valid data) and types that can signal "no more items."
pub trait WireFormat: Sized {
    /// Returns the number of bytes required to serialize this value.
    fn required_size(&self) -> usize;

    /// Serialize a value to a byte stream.
    /// Returns the number of bytes written.
    /// # Errors
    /// - If the data cannot be written to the stream
    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error>;

    /// For some UDS messages, positive replies can be suppressed via the SPRMIB (bit 7 position) of the request.
    ///
    /// Default to false, meaning that the positive response is not suppressed. Some services do not support this feature,
    /// so this function should not be used to assume that a positive response can be suppressed.
    fn is_positive_response_suppressed(&self) -> bool {
        false
    }
}

/// Types whose decode always produces a value. An empty stream is an error, not `None`.
///
/// This trait enforces at compile time that `decode` cannot return `None`.
/// The return type is `Result<Self, Error>` rather than `Result<Option<Self>, Error>`.
pub trait SingleValueWireFormat: WireFormat {
    /// Deserialize a value from a byte stream.
    /// # Errors
    /// - if the stream is empty
    /// - if the stream is not in the expected format
    /// - if the stream contains partial data
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error>;
}

struct WireFormatIterator<'a, T, R> {
    reader: &'a mut R,
    _phantom: std::marker::PhantomData<T>,
}

/// For types that can appear in lists of unknown length, this trait provides an iterator
/// that can be used to deserialize a stream of values.
impl<T: IterableWireFormat, R: std::io::Read> Iterator for WireFormatIterator<'_, T, R> {
    type Item = Result<T, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match T::decode_next(self.reader.by_ref()) {
            Ok(Some(value)) => Some(Ok(value)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// Types that can be decoded from a stream of unknown length.
///
/// `decode_next` returns `Ok(None)` when the stream is exhausted, allowing
/// iteration over variable-length sequences without prior knowledge of their size.
pub trait IterableWireFormat: WireFormat {
    /// Attempt to decode the next value from the stream.
    /// Returns `Ok(None)` if the stream is exhausted.
    /// # Errors
    /// - if the stream contains partial or invalid data
    fn decode_next<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error>;

    /// Return an iterator that decodes successive values from the stream until exhausted.
    fn decode_iter<T: std::io::Read>(reader: &mut T) -> impl Iterator<Item = Result<Self, Error>> {
        WireFormatIterator {
            reader,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[cfg(feature = "serde")]
mod maybe_serde {
    // When `serde` feature is ON, require Serialize + Deserialize
    pub trait Bound: serde::Serialize + for<'de> serde::Deserialize<'de> {}
    impl<T> Bound for T where T: serde::Serialize + for<'de> serde::Deserialize<'de> {}
}
#[cfg(not(feature = "serde"))]
mod maybe_serde {
    // When `serde` feature is OFF, require nothing
    pub trait Bound {}
    impl<T> Bound for T {}
}

#[cfg(feature = "utoipa")]
mod maybe_utoipa {
    // When `utoipa` feature is ON, require ToSchema
    pub trait Bound: utoipa::ToSchema {}
    impl<T> Bound for T where T: utoipa::ToSchema {}
}

#[cfg(not(feature = "utoipa"))]
mod maybe_utoipa {
    // When `utoipa` feature is OFF, require nothing
    pub trait Bound {}
    impl<T> Bound for T {}
}

/// Trait for types that can be used as identifiers (ie Data Identifiers and Routine Identifiers)
///
/// Use the [`impl_identifier!`](crate::impl_identifier) macro to implement this trait for your types.
pub trait Identifier: TryFrom<u16> + Into<u16> + Clone + Copy + maybe_serde::Bound {
    /// Returns a `Vec<Self>` from a reader that contains a list of Identifier values
    /// # Errors
    /// - if the list is not in the expected format
    /// - if the list contains partial data
    fn parse_from_list<R: std::io::Read>(reader: &mut R) -> Result<Vec<Self>, Error> {
        // Create an iterator to collect. Will use the blanket implementation of IterableWireFormat for Identifier
        // to read the values from the reader
        WireFormatIterator {
            reader,
            _phantom: std::marker::PhantomData,
        }
        .collect()
    }

    /// Intended to be used in a payload where the identifier is the first value and not a list of identifiers
    /// IE `DataIdentifier` (DID) payloads and `RoutineIdentifier` (RID) payloads
    ///
    /// Returns the identifier, or None if the reader is empty
    ///
    /// ## Example reading a payload that has multiple identifiers
    /// ```rust,ignore
    /// while let Some(identifier) = MyIdentifier::parse_from_payload(&mut buffer).unwrap() {
    ///     match identifier {
    ///        MyIdentifier::Identifier1 | MyIdentifier::Identifier2 => {
    ///           let payload = MyPayload::decode(&mut buffer).unwrap();
    ///         }
    ///        // No payload for Identifier3
    ///        MyIdentifier::MyIdentifier3 => (),
    ///        MyIdentifier::UDSIdentifier(_) => (),
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    /// - if the stream is not in the expected format
    /// - if the stream contains partial data
    fn parse_from_payload<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        <Self as IterableWireFormat>::decode_next(reader)
    }
}

/// Implement the [`Identifier`] trait for a type.
///
/// The type must already implement `TryFrom<u16>`, `Into<u16>`, `Clone`, and `Copy`.
///
/// # Example
/// ```rust,ignore
/// impl_identifier!(MyIdentifierEnum);
/// ```
#[macro_export]
macro_rules! impl_identifier {
    ($($t:ty),+ $(,)?) => {
        $(impl $crate::Identifier for $t {})+
    };
}

/// Marker subtrait of [`Identifier`] to distinguish routine identifiers from data identifiers.
pub trait RoutineIdentifier: Identifier {}

/// Blanket implementation of [`Encode`] for types that implement [`Identifier`]
impl<T> Encode for T
where
    T: Identifier,
{
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&<u16 as Into<u16>>::into((*self).into()).to_be_bytes())
            .map_err(Error::io)?;
        Ok(2)
    }
}

/// Blanket implementation of [`Decode`] for types that implement [`Identifier`]
impl<'a, T> Decode<'a> for T
where
    T: Identifier,
{
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::IncorrectMessageLengthOrInvalidFormat);
        }
        let raw = u16::from_be_bytes([buf[0], buf[1]]);
        match Self::try_from(raw) {
            Ok(identifier) => Ok((identifier, &buf[2..])),
            Err(_) => Err(Error::InvalidDiagnosticIdentifier(raw)),
        }
    }
}

/// Blanket implementation of [`DecodeIter`] for types that implement [`Identifier`]
impl<'a, T> DecodeIter<'a> for T
where
    T: Identifier,
{
    fn decode_next(buf: &'a [u8]) -> Result<Option<(Self, &'a [u8])>, Error> {
        if buf.is_empty() {
            return Ok(None);
        }
        if buf.len() < 2 {
            return Err(Error::IncorrectMessageLengthOrInvalidFormat);
        }
        Decode::decode(buf).map(Some)
    }
}

/// Blanket implementation of [`WireFormat`] for types that implement [`Identifier`]
impl<T> WireFormat for T
where
    T: Identifier,
{
    fn required_size(&self) -> usize {
        2
    }

    fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<usize, Error> {
        writer.write_u16::<BigEndian>((*self).into())?;
        Ok(2)
    }
}

/// Blanket implementation of [`SingleValueWireFormat`] for types that implement [`Identifier`]
impl<T> SingleValueWireFormat for T
where
    T: Identifier,
{
    fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 | 1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        }

        match Self::try_from(u16::from_be_bytes(identifier_data)) {
            Ok(identifier) => Ok(identifier),
            Err(_) => Err(Error::InvalidDiagnosticIdentifier(u16::from_be_bytes(
                identifier_data,
            ))),
        }
    }
}

/// Blanket implementation of [`IterableWireFormat`] for types that implement [`Identifier`]
impl<T> IterableWireFormat for T
where
    T: Identifier,
{
    fn decode_next<R: std::io::Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        }

        match Self::try_from(u16::from_be_bytes(identifier_data)) {
            Ok(identifier) => Ok(Some(identifier)),
            Err(_) => Err(Error::InvalidDiagnosticIdentifier(u16::from_be_bytes(
                identifier_data,
            ))),
        }
    }
}

/// A trait that defines the user-defined diagnostic definitions/specifiers for UDS requests and responses.
///
/// Used to specify the types of the identifiers and payloads used in UDS requests and responses.
/// It allows for flexibility in defining custom data types while adhering to the UDS protocol.
pub trait DiagnosticDefinition: 'static {
    /// UDS Data Identifier
    ///
    /// Requests : [`ReadDataByIdentifierRequest`](crate::ReadDataByIdentifierRequest), [`WriteDataByIdentifierRequest`](crate::WriteDataByIdentifierRequest), and [`ReadDTCInfoRequest`](crate::ReadDTCInfoRequest)
    /// Responses: [`ReadDataByIdentifierResponse`](crate::ReadDataByIdentifierResponse), [`WriteDataByIdentifierResponse`](crate::WriteDataByIdentifierResponse), and [`ReadDTCInfoResponse`](crate::ReadDTCInfoResponse)
    type DID: Identifier
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + PartialEq
        + 'static
        + maybe_serde::Bound
        + maybe_utoipa::Bound;
    /// Response payload for [`ReadDataByIdentifierRequest`](crate::ReadDataByIdentifierRequest)
    type DiagnosticPayload: SingleValueWireFormat
        + IterableWireFormat
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + PartialEq
        + maybe_serde::Bound
        + maybe_utoipa::Bound
        + 'static;

    /// UDS Routine Identifier
    ///
    /// This is used to identify the routine to be controlled in a [`RoutineControlRequest`](crate::RoutineControlRequest)
    type RID: RoutineIdentifier
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + PartialEq
        + 'static
        + maybe_serde::Bound
        + maybe_utoipa::Bound;
    /// Payload for both requests and responses of [`RoutineControlRequest`](crate::RoutineControlRequest) and [`RoutineControlResponse`](crate::RoutineControlResponse)
    type RoutinePayload: SingleValueWireFormat
        + IterableWireFormat
        + Clone
        + std::fmt::Debug
        + Send
        + Sync
        + PartialEq
        + 'static
        + maybe_serde::Bound
        + maybe_utoipa::Bound;
}

/// tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Identifier, UDSIdentifier};
    use byteorder_embedded_io::io::ReadBytesExt;
    use std::io::Cursor;

    #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    #[repr(u16)]
    pub enum MyIdentifier {
        Identifier1 = 0x0101,
        Identifier2 = 0x0202,
        Identifier3 = 0x0303,
        UDSIdentifier(UDSIdentifier),
    }
    impl_identifier!(MyIdentifier);

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
        WireFormat::encode(&identifier, &mut buffer).unwrap();
        buffer.set_position(0);
        let read_identifier = MyIdentifier::parse_from_list(&mut buffer).unwrap();
        assert_eq!(identifier, read_identifier[0]);
    }

    #[test]
    #[allow(clippy::match_same_arms)]
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
