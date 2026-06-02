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
}

/// RX-side trait: zero-copy decode from a byte slice.
///
/// Implementations borrow directly from the input buffer where possible. The decoded
/// value points into `buf` and is valid only as long as `buf` lives — for C developers
/// new to Rust, think of it like a `struct` overlaid on a `char buf[]`. Copy out any
/// fields you need to retain beyond the buffer's lifetime.
///
/// [`decode`](Self::decode) returns the value together with the unconsumed remainder of
/// the buffer, so leaf and sequence decoders can be composed. Frame-level decoders
/// (`Request`, `Response`) consume the whole buffer and return an empty remainder; use
/// [`decode_exact`](Self::decode_exact) when a buffer must contain exactly one value.
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
