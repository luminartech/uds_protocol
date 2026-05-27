use crate::{
    Decode, DecodeIter, Encode, Error, UDSIdentifier, UDSRoutineIdentifier, impl_identifier,
};
use core::ops::Deref;

/// Protocol Identifier provides an implementation of Diagnostics Identifiers that only supports Diagnostic Identifiers defined by UDS
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolIdentifier {
    identifier: UDSIdentifier,
}
impl_identifier!(ProtocolIdentifier);

impl ProtocolIdentifier {
    /// Wrap a [`UDSIdentifier`] in a `ProtocolIdentifier`.
    #[must_use]
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

/// Zero-alloc protocol payload. Borrows the raw payload bytes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ProtocolPayloadTx<'d> {
    /// The UDS data identifier this payload belongs to.
    pub identifier: UDSIdentifier,
    /// The raw payload bytes following the identifier.
    pub payload: &'d [u8],
}

impl<'d> ProtocolPayloadTx<'d> {
    /// Creates a new `ProtocolPayloadTx`.
    #[must_use]
    pub const fn new(identifier: UDSIdentifier, payload: &'d [u8]) -> Self {
        Self {
            identifier,
            payload,
        }
    }
}

impl Encode for ProtocolPayloadTx<'_> {
    fn encoded_size(&self) -> usize {
        2 + self.payload.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        Encode::encode(&self.identifier, writer)?;
        writer.write_all(self.payload).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for ProtocolPayloadTx<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::IncorrectMessageLengthOrInvalidFormat);
        }
        let (identifier, rest) = <UDSIdentifier as Decode>::decode(buf)?;
        // Consumes all remaining bytes as payload
        Ok((
            Self {
                identifier,
                payload: rest,
            },
            &[],
        ))
    }
}

impl<'a> DecodeIter<'a> for ProtocolPayloadTx<'a> {
    fn decode_next(buf: &'a [u8]) -> Result<Option<(Self, &'a [u8])>, Error> {
        if buf.is_empty() {
            return Ok(None);
        }
        Decode::decode(buf).map(Some)
    }
}

impl core::fmt::Debug for ProtocolPayloadTx<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} =>", self.identifier)?;
        for b in self.payload {
            write!(f, " {b:02X}")?;
        }
        Ok(())
    }
}

/// Zero-alloc routine payload. Borrows the raw payload bytes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ProtocolRoutinePayloadTx<'d> {
    /// The routine identifier this payload belongs to.
    pub identifier: UDSRoutineIdentifier,
    /// The raw payload bytes following the identifier.
    pub payload: &'d [u8],
}

impl<'d> ProtocolRoutinePayloadTx<'d> {
    /// Creates a new `ProtocolRoutinePayloadTx`.
    #[must_use]
    pub const fn new(identifier: UDSRoutineIdentifier, payload: &'d [u8]) -> Self {
        Self {
            identifier,
            payload,
        }
    }
}

impl Encode for ProtocolRoutinePayloadTx<'_> {
    /// Size of the raw payload only -- the identifier is written by the request.
    fn encoded_size(&self) -> usize {
        self.payload.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(self.payload).map_err(Error::io)?;
        Ok(self.payload.len())
    }
}

impl<'a> Decode<'a> for ProtocolRoutinePayloadTx<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::IncorrectMessageLengthOrInvalidFormat);
        }
        let raw = u16::from_be_bytes([buf[0], buf[1]]);
        let identifier = UDSRoutineIdentifier::from(raw);
        Ok((
            Self {
                identifier,
                payload: &buf[2..],
            },
            &[],
        ))
    }
}

impl<'a> DecodeIter<'a> for ProtocolRoutinePayloadTx<'a> {
    fn decode_next(buf: &'a [u8]) -> Result<Option<(Self, &'a [u8])>, Error> {
        if buf.is_empty() {
            return Ok(None);
        }
        Decode::decode(buf).map(Some)
    }
}

impl core::fmt::Debug for ProtocolRoutinePayloadTx<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?} =>", self.identifier)?;
        for b in self.payload {
            write!(f, " {b:02X}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_construction_and_debug_format() {
        let payload = ProtocolPayloadTx::new(UDSIdentifier::ActiveDiagnosticSession, &[0x01]);
        assert_eq!(format!("{payload:?}"), "0xF186 => 01");
        let mut buf = [0u8; 8];
        let written = Encode::encode(&payload, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 3);
    }

    #[test]
    fn test_encode_and_decode() {
        let payload = ProtocolPayloadTx::new(UDSIdentifier::ActiveDiagnosticSession, &[0x03]);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&payload, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 3);
        let (decoded, _) = ProtocolPayloadTx::decode(&buf[..written]).unwrap();
        assert_eq!(payload, decoded);
    }
}
