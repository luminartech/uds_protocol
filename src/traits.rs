use crate::Error;

// ---------------------------------------------------------------------------
// New no_std-compatible traits (TX: Encode, RX: Decode / DecodeIter)
// ---------------------------------------------------------------------------

/// TX-side trait: encode a value into an [`embedded_io::Write`] implementor.
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

/// Trait for types that can be used as identifiers (ie Data Identifiers and Routine Identifiers)
///
/// Use the [`impl_identifier!`](crate::impl_identifier) macro to implement this trait for your types.
pub trait Identifier: TryFrom<u16> + Into<u16> + Clone + Copy {}

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
            .write_all(&Into::<u16>::into(*self).to_be_bytes())
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

/// Trait for diagnostic definitions that specifies the identifier and payload
/// types used when constructing and parsing UDS requests and responses.
pub trait DiagnosticDefinition<'a> {
    /// UDS Data Identifier type.
    type DID: Identifier + Clone + core::fmt::Debug + PartialEq + 'static;
    /// Payload type for read/write data by identifier etc.
    type DiagnosticPayload: Encode + Clone + core::fmt::Debug + PartialEq + 'a;
    /// UDS Routine Identifier type.
    type RID: RoutineIdentifier + Clone + core::fmt::Debug + PartialEq + 'static;
    /// Payload type for routine control requests/responses.
    type RoutinePayload: Encode + Clone + core::fmt::Debug + PartialEq + 'a;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UDSIdentifier;

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

    #[test]
    fn test_identifier_encode_decode() {
        let identifier = MyIdentifier::Identifier1;
        let mut buf = [0u8; 2];
        Encode::encode(&identifier, &mut buf.as_mut_slice()).unwrap();
        let (decoded, rest) = <MyIdentifier as Decode>::decode(&buf).unwrap();
        assert_eq!(identifier, decoded);
        assert!(rest.is_empty());
    }

    #[test]
    #[allow(clippy::match_same_arms)]
    fn test_identifier_decode_iter() {
        let data = [0x01u8, 0x01, 0x02, 0x02, 0x03, 0x03];
        let mut remaining = &data[..];
        let mut count = 0;
        while let Some((id, rest)) = MyIdentifier::decode_next(remaining).unwrap() {
            remaining = rest;
            count += 1;
            match id {
                MyIdentifier::Identifier1 => assert_eq!(count, 1),
                MyIdentifier::Identifier2 => assert_eq!(count, 2),
                MyIdentifier::Identifier3 => assert_eq!(count, 3),
                MyIdentifier::UDSIdentifier(_) => panic!("Unexpected"),
            }
        }
        assert_eq!(count, 3);
    }
}
