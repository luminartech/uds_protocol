//! `TesterPresent` (0x3E) service implementation
use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, Incomplete, NegativeResponseCode};
use automotive_wire_codec::write_u8;

const TESTER_PRESENT_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 2] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
];

const NO_SUBFUNCTION_VALUE: u8 = 0x00;

/// Subfunction parameter values for the `TesterPresent` service.
///
/// The range of values is only 7 of the 8 bits, with bit 7 being used as the
/// Suppress Positive Response (SPR) Message Indication Bit.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ZeroSubFunction {
    /// Request and response. Indicates that no value beside the SPR Message Indication Bit is supported by this service.
    NoSubFunctionSupported,
    /// Request only.
    IsoSaeReserved(u8),
}

impl Default for ZeroSubFunction {
    #[inline]
    fn default() -> Self {
        ZeroSubFunction::NoSubFunctionSupported
    }
}

impl ZeroSubFunction {
    /// The raw sub-function byte. `const` so callers' accessors can be `const` too.
    const fn value(self) -> u8 {
        match self {
            ZeroSubFunction::NoSubFunctionSupported => NO_SUBFUNCTION_VALUE,
            ZeroSubFunction::IsoSaeReserved(value) => value,
        }
    }
}

impl From<ZeroSubFunction> for u8 {
    fn from(sub_function: ZeroSubFunction) -> Self {
        sub_function.value()
    }
}

impl TryFrom<u8> for ZeroSubFunction {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            NO_SUBFUNCTION_VALUE => Ok(ZeroSubFunction::NoSubFunctionSupported),
            0x01..=0x7F => Ok(ZeroSubFunction::IsoSaeReserved(value)),
            _ => Err(Error::InvalidTesterPresentType(value)),
        }
    }
}

/// Request to indicate the client is still connected
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct TesterPresentRequest {
    /// Whether the server should suppress a positive response (SPRMIB).
    pub suppress_positive_response: bool,
    /// The sub-function byte with SPRMIB stripped. `TesterPresent` defines only the zero
    /// sub-function, so conformant traffic always carries `0x00`; this is kept private so a
    /// caller cannot mint a reserved value, but is retained on decode so that a reserved byte
    /// re-encodes unchanged. Read it back with [`TesterPresentRequest::sub_function`].
    zero_sub_function: ZeroSubFunction,
}

impl TesterPresentRequest {
    /// Create a new `TesterPresentRequest` carrying the zero sub-function.
    #[must_use]
    pub const fn new(suppress_positive_response: bool) -> Self {
        Self {
            suppress_positive_response,
            zero_sub_function: ZeroSubFunction::NoSubFunctionSupported,
        }
    }

    /// The sub-function byte, with the SPRMIB bit stripped.
    ///
    /// `0x00` for conformant traffic. `0x01..=0x7F` is reserved by ISO/SAE: the value is
    /// retained rather than normalized so it re-encodes unchanged, and so a server can report
    /// [`NegativeResponseCode::SubFunctionNotSupported`] against the byte it actually received.
    #[must_use]
    pub const fn sub_function(&self) -> u8 {
        self.zero_sub_function.value()
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &TESTER_PRESENT_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for TesterPresentRequest {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        // Fuse the SPRMIB bit back onto the sub-function at the wire boundary. The retained
        // sub-function is written verbatim so a reserved value round-trips unchanged.
        let sub_function = SuppressablePositiveResponse::new(
            self.suppress_positive_response,
            self.zero_sub_function,
        );
        write_u8(writer, u8::from(sub_function)).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for TesterPresentRequest {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        // Split out the SPRMIB flag. Once SPRMIB is stripped the low 7 bits are always a
        // valid zero sub-function, so this never rejects; the sub-function value is retained
        // so that a reserved byte re-encodes unchanged.
        let sub_function = SuppressablePositiveResponse::<ZeroSubFunction>::try_from(buf[0])?;
        Ok((
            Self {
                suppress_positive_response: sub_function.suppress_positive_response(),
                zero_sub_function: sub_function.value(),
            },
            &buf[1..],
        ))
    }
}

/// Positive response to a `TesterPresentRequest`
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct TesterPresentResponse {
    zero_sub_function: ZeroSubFunction,
}

impl TesterPresentResponse {
    /// Create a new `TesterPresentResponse`
    #[must_use]
    pub const fn new() -> Self {
        Self {
            zero_sub_function: ZeroSubFunction::NoSubFunctionSupported,
        }
    }

    /// The sub-function byte echoed by the server. `0x00` for conformant traffic; a reserved
    /// `0x01..=0x7F` value is retained verbatim, mirroring
    /// [`TesterPresentRequest::sub_function`].
    #[must_use]
    pub const fn sub_function(&self) -> u8 {
        self.zero_sub_function.value()
    }
}

impl Default for TesterPresentResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl Encode for TesterPresentResponse {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        write_u8(writer, u8::from(self.zero_sub_function)).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for TesterPresentResponse {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        let zero_sub_function = ZeroSubFunction::try_from(buf[0])?;
        Ok((Self { zero_sub_function }, &buf[1..]))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::{vec, vec::Vec};

    #[test]
    fn try_from_all_zero_subfunction() {
        for i in 0..u8::MAX {
            let try_result: Result<ZeroSubFunction, Error> = ZeroSubFunction::try_from(i);
            match i {
                0x00 => {
                    assert_eq!(try_result.unwrap(), ZeroSubFunction::NoSubFunctionSupported);
                }
                0x01..=0x7F => {
                    assert!(matches!(try_result, Ok(ZeroSubFunction::IsoSaeReserved(_))));
                }
                _ => {
                    assert!(matches!(
                        try_result,
                        Err(Error::InvalidTesterPresentType(_))
                    ));
                }
            }
        }
    }

    #[test]
    fn from_all_zero_subfunction() {
        assert_eq!(u8::from(ZeroSubFunction::default()), NO_SUBFUNCTION_VALUE);

        for i in 0x01..=0x7F {
            let result = ZeroSubFunction::IsoSaeReserved(i);
            assert_eq!(u8::from(result), i);
        }
    }

    #[cfg(feature = "alloc")]
    fn make_request(byte: u8) -> Result<TesterPresentRequest, Error> {
        let bytes = vec![byte];
        let (val, _) = <TesterPresentRequest as Decode>::decode(&bytes)?;
        Ok(val)
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn read_request_type() {
        for i in 0..u8::MAX {
            let result = make_request(i);
            match i {
                0x00 => {
                    let expected = TesterPresentRequest::new(false);
                    assert_eq!(result.unwrap(), expected);
                }
                0x01..=0x7F => {
                    // Reserved sub-function bytes decode with SPRMIB clear, and the reserved
                    // value is retained verbatim (see
                    // `reserved_sub_function_survives_a_round_trip`).
                    let req = result.unwrap();
                    assert!(!req.suppress_positive_response);
                    assert_eq!(req.sub_function(), i);
                }
                0x80 => {
                    let expected = TesterPresentRequest::new(true);
                    assert_eq!(result.unwrap(), expected);
                }
                0x81..=0xFF => {
                    // SPRMIB set over a reserved value: both the flag and the value are kept.
                    let req = result.unwrap();
                    assert!(req.suppress_positive_response);
                    assert_eq!(req.sub_function(), i & 0x7F);
                }
            }
        }
    }

    #[test]
    fn reserved_sub_function_survives_a_round_trip() {
        // Reserved sub-function bytes must re-encode byte-for-byte. Previously the value was
        // discarded and normalized to 0x00, so `[0x3E, 0x01]` came back out as `[0x3E, 0x00]` —
        // the request silently rewrote the tester's frame, while `TesterPresentResponse`
        // preserved the same values. A server needs the original byte to report
        // subFunctionNotSupported against it.
        for raw in [0x00u8, 0x01, 0x42, 0x7F] {
            for suppress in [false, true] {
                let wire = [raw | if suppress { 0x80 } else { 0x00 }];
                let (req, rest) = <TesterPresentRequest as Decode>::decode(&wire).unwrap();
                assert!(rest.is_empty());
                assert_eq!(req.suppress_positive_response, suppress);
                assert_eq!(req.sub_function(), raw, "sub-function byte not retained");

                let mut buf = [0u8; 4];
                let n = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
                assert_eq!(&buf[..n], &wire, "lossy re-encode for {wire:02X?}");
                assert_encode_size_agrees(&req);
            }
        }
    }

    #[test]
    fn reserved_sub_function_round_trips_through_the_request_frame() {
        use crate::Request;
        let wire = [0x3E, 0x01];
        let (req, _) = Request::decode(&wire).unwrap();
        let mut buf = [0u8; 4];
        let n = req.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..n], &wire);
    }

    #[test]
    fn new_still_produces_the_zero_sub_function() {
        // The common case keeps its one-argument constructor and its 0x00 encoding.
        assert_eq!(TesterPresentRequest::new(false).sub_function(), 0x00);
        let mut buf = [0u8; 4];
        let n = Encode::encode(&TesterPresentRequest::new(true), &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..n], &[0x80]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn write_request_type() {
        let test_type = TesterPresentRequest::new(false);
        let mut buffer = Vec::new();
        Encode::encode(&test_type, &mut buffer).unwrap();

        let expected_bytes = vec![0];
        assert_eq!(buffer, expected_bytes);
        assert_encode_size_agrees(&test_type);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn read_response_type() {
        let bytes = vec![0u8];
        let (test_type, _) = <TesterPresentResponse as Decode>::decode(&bytes).unwrap();
        assert_eq!(test_type, TesterPresentResponse::new());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn write_response_type() {
        let test_type = TesterPresentResponse::new();
        let mut buffer = Vec::new();
        Encode::encode(&test_type, &mut buffer).unwrap();

        let expected_bytes = vec![0];
        assert_eq!(buffer, expected_bytes);
        assert_encode_size_agrees(&test_type);
    }
}
