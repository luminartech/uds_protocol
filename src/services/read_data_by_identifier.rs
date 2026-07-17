//! `ReadDataByIdentifier` (0x22) service implementation
use crate::{Decode, Encode, Error, NegativeResponseCode};

/// Positive response to `ReadDataByIdentifier`: raw `[DID][data record]…` bytes.
///
/// Left opaque **by design**: each data record's length is defined by the ECU's
/// configuration for that DID and is not present on the wire, so the library cannot
/// split it into `(DID, value)` pairs. Read the 2-byte big-endian DID, take the
/// application-defined number of data bytes via [`records`](Self::records), then repeat.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ReadDataByIdentifierResponse<'a> {
    #[cfg_attr(feature = "serde", serde(borrow))]
    records: &'a [u8],
}

impl<'a> ReadDataByIdentifierResponse<'a> {
    /// Wrap the raw record bytes of a positive `ReadDataByIdentifier` response.
    #[must_use]
    pub const fn new(records: &'a [u8]) -> Self {
        Self { records }
    }

    /// The raw `[DID][data record]…` bytes, to be parsed caller-side.
    #[must_use]
    pub const fn records(&self) -> &'a [u8] {
        self.records
    }
}

impl Encode for ReadDataByIdentifierResponse<'_> {
    fn encoded_size(&self) -> usize {
        self.records.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(self.records).map_err(Error::io)?;
        Ok(self.records.len())
    }
}

impl<'a> Decode<'a> for ReadDataByIdentifierResponse<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        Ok((Self { records: buf }, &[]))
    }
}

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

/// Read-DID request: a list of 16-bit Data Identifiers. Built from native `&[u16]`
/// or borrowed from the wire as big-endian bytes; `dids()` yields `u16` either way.
///
/// # serde / utoipa carve-out
///
/// serde derives are omitted: the zero-copy `Native(&[u16])` backing has no borrowed
/// `Deserialize` impl in serde (zero-copy borrowing via `#[serde(borrow)]` only works
/// for `&[u8]` and `&str`, not `&[u16]`), so `Dids::Native(&[u16])` cannot be
/// deserialized without an owned allocation.
///
/// utoipa derives are omitted: `utoipa::ToSchema` cannot be derived on
/// `ReadDataByIdentifierRequest` without also deriving it on the private `Dids` enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ReadDataByIdentifierRequest<'d> {
    dids: Dids<'d>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Dids<'d> {
    Native(&'d [u16]),
    Wire(&'d [u8]),
}

impl<'d> ReadDataByIdentifierRequest<'d> {
    /// Build a request from a list of Data Identifiers.
    #[must_use]
    pub const fn new(dids: &'d [u16]) -> Self {
        Self {
            dids: Dids::Native(dids),
        }
    }

    /// Iterate the requested DIDs as `u16` (big-endian swap hidden for wire-backed values).
    pub fn dids(&self) -> impl Iterator<Item = u16> + '_ {
        DidIter {
            dids: self.dids,
            pos: 0,
        }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DID_NEGATIVE_RESPONSE_CODES
    }
}

struct DidIter<'d> {
    dids: Dids<'d>,
    pos: usize,
}

impl Iterator for DidIter<'_> {
    type Item = u16;
    fn next(&mut self) -> Option<u16> {
        match self.dids {
            Dids::Native(s) => {
                let v = *s.get(self.pos)?;
                self.pos += 1;
                Some(v)
            }
            Dids::Wire(b) => {
                let hi = *b.get(self.pos)?;
                let lo = *b.get(self.pos + 1)?;
                self.pos += 2;
                Some(u16::from_be_bytes([hi, lo]))
            }
        }
    }
}

impl Encode for ReadDataByIdentifierRequest<'_> {
    fn encoded_size(&self) -> usize {
        match self.dids {
            Dids::Native(s) => s.len() * 2,
            Dids::Wire(b) => b.len(),
        }
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        match self.dids {
            Dids::Native(s) => {
                for did in s {
                    writer.write_all(&did.to_be_bytes()).map_err(Error::io)?;
                }
            }
            Dids::Wire(b) => writer.write_all(b).map_err(Error::io)?,
        }
        Ok(match self.dids {
            Dids::Native(s) => s.len() * 2,
            Dids::Wire(b) => b.len(),
        })
    }
}

impl<'a> Decode<'a> for ReadDataByIdentifierRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() || buf.len() % 2 != 0 {
            return Err(Error::IncorrectMessageLengthOrInvalidFormat);
        }
        Ok((
            Self {
                dids: Dids::Wire(buf),
            },
            &[],
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::{assert_encode_size_agrees, assert_impl_eq};

    #[test]
    fn derive_contract() {
        assert_impl_eq::<ReadDataByIdentifierRequest<'static>>();
        // serde: omitted — see struct-level doc comment for rationale
    }

    #[test]
    fn rdbi_native_encodes_be() {
        let req = ReadDataByIdentifierRequest::new(&[0xF190, 0xF186]);
        let mut buf = [0u8; 8];
        let n = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..n], &[0xF1, 0x90, 0xF1, 0x86]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn rdbi_wire_decodes_and_dids_iterates() {
        let (req, rest) =
            <ReadDataByIdentifierRequest as Decode>::decode(&[0xF1, 0x90, 0xF1, 0x86]).unwrap();
        assert!(rest.is_empty());
        // Iterate without alloc (no_std-friendly): pull items directly.
        let mut it = req.dids();
        assert_eq!(it.next(), Some(0xF190));
        assert_eq!(it.next(), Some(0xF186));
        assert_eq!(it.next(), None);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn rdbi_cross_backing_encodes_identically() {
        let native = ReadDataByIdentifierRequest::new(&[0xF190]);
        let mut a = [0u8; 4];
        let na = Encode::encode(&native, &mut a.as_mut_slice()).unwrap();
        let (wire, _) = <ReadDataByIdentifierRequest as Decode>::decode(&a[..na]).unwrap();
        let mut b = [0u8; 4];
        let nb = Encode::encode(&wire, &mut b.as_mut_slice()).unwrap();
        assert_eq!(a[..na], b[..nb]);
    }

    #[test]
    fn rdbi_rejects_empty_and_odd() {
        assert!(<ReadDataByIdentifierRequest as Decode>::decode(&[]).is_err());
        assert!(<ReadDataByIdentifierRequest as Decode>::decode(&[0xF1]).is_err());
        assert!(<ReadDataByIdentifierRequest as Decode>::decode(&[0xF1, 0x90, 0xF1]).is_err());
    }

    #[test]
    fn rdbi_response_wraps_and_roundtrips() {
        use crate::{Decode, Encode};
        let raw = [0xF1, 0x90, 0x01, 0x02];
        let (resp, remaining) = <ReadDataByIdentifierResponse as Decode>::decode(&raw).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(resp.records(), &raw);
        let mut buf = [0u8; 8];
        let n = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..n], &raw);
    }

    #[test]
    fn encode_read_did_request_tx() {
        let ids = [0xF180u16, 0xF186u16];
        let req = ReadDataByIdentifierRequest::new(&ids);
        let mut buf = [0u8; 16];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 4); // 2 DIDs * 2 bytes each
        assert_eq!(&buf[..4], &[0xF1, 0x80, 0xF1, 0x86]);
        assert_encode_size_agrees(&req);
    }
}
