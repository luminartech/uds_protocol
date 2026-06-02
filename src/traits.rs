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

    /// Decode from `buf`, requiring the entire buffer to be consumed.
    ///
    /// Use this when `buf` is expected to contain exactly one value and any
    /// trailing bytes indicate a malformed frame.
    ///
    /// # Errors
    /// Returns [`Error::IncorrectMessageLengthOrInvalidFormat`] if any bytes
    /// remain after decoding, or whatever error [`decode`](Self::decode)
    /// produces.
    fn decode_exact(buf: &'a [u8]) -> Result<Self, Error> {
        let (value, rest) = Self::decode(buf)?;
        if rest.is_empty() {
            Ok(value)
        } else {
            Err(Error::IncorrectMessageLengthOrInvalidFormat)
        }
    }
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
