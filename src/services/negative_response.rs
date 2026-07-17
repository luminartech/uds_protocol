//! `NegativeResponse` (0x7F) service implementation
use crate::{Decode, Encode, Error, Incomplete, NegativeResponseCode, UdsServiceType};

/// A negative response from the server indicating a request could not be fulfilled.
///
/// The echoed request-service byte is stored raw so a decoded negative response
/// re-encodes **losslessly**, even when it references a service this library does not
/// model (or a reserved/future SID). Read it as a typed [`UdsServiceType`] via
/// [`request_service`](Self::request_service), or as the raw byte via
/// [`request_service_sid`](Self::request_service_sid).
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct NegativeResponse {
    /// Raw echoed request-service byte from the wire, preserved verbatim.
    request_service_sid: u8,
    /// The negative response code indicating why the request failed.
    nrc: NegativeResponseCode,
}

impl NegativeResponse {
    /// Create a new `NegativeResponse` for a modeled request service.
    #[must_use]
    pub fn new(request_service: UdsServiceType, nrc: NegativeResponseCode) -> Self {
        Self {
            request_service_sid: request_service.request_service_to_byte(),
            nrc,
        }
    }

    /// The service that triggered this negative response, as a typed [`UdsServiceType`].
    ///
    /// An unmodeled/reserved echoed byte maps to
    /// [`UdsServiceType::UnsupportedDiagnosticService`]; the original byte remains available
    /// from [`request_service_sid`](Self::request_service_sid) and is what gets re-encoded.
    #[must_use]
    pub fn request_service(&self) -> UdsServiceType {
        UdsServiceType::service_from_request_byte(self.request_service_sid)
    }

    /// The raw echoed request-service byte, exactly as received on the wire.
    #[must_use]
    pub const fn request_service_sid(&self) -> u8 {
        self.request_service_sid
    }

    /// The negative response code indicating why the request failed.
    #[must_use]
    pub const fn nrc(&self) -> NegativeResponseCode {
        self.nrc
    }
}

impl Encode for NegativeResponse {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[self.request_service_sid, u8::from(self.nrc)])
            .map_err(Error::io)?;
        Ok(2)
    }
}

impl<'a> Decode<'a> for NegativeResponse {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(Incomplete {
                needed: 2,
                available: buf.len(),
            }));
        }
        Ok((
            Self {
                request_service_sid: buf[0],
                nrc: NegativeResponseCode::from(buf[1]),
            },
            &buf[2..],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn negative_response_encode_size_agrees() {
        let value = NegativeResponse::new(
            UdsServiceType::DiagnosticSessionControl,
            NegativeResponseCode::ServiceNotSupported,
        );
        assert_encode_size_agrees(&value);
    }

    #[test]
    fn unknown_echoed_service_round_trips_losslessly() {
        // 0x40 is not a modeled request service. The echoed byte must survive
        // decode -> encode verbatim (it previously normalized to 0x7F).
        let wire = [0x40, 0x12];
        let (nr, rest) = <NegativeResponse as Decode>::decode(&wire).unwrap();
        assert!(rest.is_empty());
        assert_eq!(nr.request_service_sid(), 0x40);
        assert_eq!(
            nr.request_service(),
            UdsServiceType::service_from_request_byte(0x40)
        );
        let mut buf = [0u8; 2];
        let n = Encode::encode(&nr, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..n], &wire);
    }
}
