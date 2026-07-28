//! `NegativeResponse` (0x7F) service implementation
use crate::{Decode, Encode, Error, Incomplete, NegativeResponseCode, UdsServiceType};
use automotive_wire_codec::write_all;

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
    ///
    /// Private, unlike the public data-bag fields on most response types: it is a raw byte
    /// whose typed meaning is derived ([`request_service`](Self::request_service)), and the two
    /// constructors deliberately offer different guarantees — [`new`](Self::new) takes a typed
    /// service, [`new_with_sid`](Self::new_with_sid) takes the byte.
    request_service_sid: u8,
    /// The negative response code indicating why the request failed.
    nrc: NegativeResponseCode,
}

impl NegativeResponse {
    /// Create a new `NegativeResponse` for a modeled request service.
    ///
    /// Note that [`UdsServiceType::UnsupportedDiagnosticService`] and
    /// [`UdsServiceType::NegativeResponse`] have no request SID and both echo `0x7F`. To NACK a
    /// service byte the crate does not model — the `sid` of a
    /// [`Request::Other`](crate::Request::Other) — use [`new_with_sid`](Self::new_with_sid) so
    /// the original byte is echoed.
    #[must_use]
    pub fn new(request_service: UdsServiceType, nrc: NegativeResponseCode) -> Self {
        Self {
            request_service_sid: request_service.to_request_sid(),
            nrc,
        }
    }

    /// Create a new `NegativeResponse` echoing a raw request-service byte.
    ///
    /// For the pass-through case: a server that decoded a
    /// [`Request::Other`](crate::Request::Other) can answer
    /// [`ServiceNotSupported`](NegativeResponseCode::ServiceNotSupported) while echoing the
    /// byte it actually received, which [`new`](Self::new) cannot express.
    ///
    /// ```
    /// use uds_protocol::{NegativeResponse, NegativeResponseCode};
    ///
    /// let nack = NegativeResponse::new_with_sid(0x40, NegativeResponseCode::ServiceNotSupported);
    /// assert_eq!(nack.request_service_sid(), 0x40);
    /// ```
    #[must_use]
    pub const fn new_with_sid(request_service_sid: u8, nrc: NegativeResponseCode) -> Self {
        Self {
            request_service_sid,
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
        UdsServiceType::from_request_sid(self.request_service_sid)
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
        write_all(writer, &[self.request_service_sid, u8::from(self.nrc)]).map_err(Error::io)
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
    fn a_server_can_nack_an_unmodeled_service_byte() {
        // The pass-through case this type advertises, from the *construction* side. A server
        // that decodes `Request::Other { sid: 0x40 }` must be able to answer
        // serviceNotSupported echoing 0x40. `new()` cannot express that: it routes through
        // `to_request_sid()`, which collapses every unmodeled service to 0x7F.
        let (req, _) = <crate::Request as Decode>::decode(&[0x40, 0xAA]).unwrap();
        let sid = match req {
            crate::Request::Other { sid, .. } => sid,
            other => panic!("expected Other, got {other:?}"),
        };
        let nack = NegativeResponse::new_with_sid(sid, NegativeResponseCode::ServiceNotSupported);
        assert_eq!(nack.request_service_sid(), 0x40);

        let mut buf = [0u8; 2];
        let n = Encode::encode(&nack, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..n], &[0x40, 0x11]);

        // The typed constructor still collapses unmodeled services, which is why the raw
        // constructor has to exist.
        let via_typed = NegativeResponse::new(
            UdsServiceType::UnsupportedDiagnosticService,
            NegativeResponseCode::ServiceNotSupported,
        );
        assert_eq!(via_typed.request_service_sid(), 0x7F);
    }

    #[test]
    fn unknown_echoed_service_round_trips_losslessly() {
        // 0x40 is not a modeled request service. The echoed byte must survive
        // decode -> encode verbatim (it previously normalized to 0x7F).
        let wire = [0x40, 0x12];
        let (nr, rest) = <NegativeResponse as Decode>::decode(&wire).unwrap();
        assert!(rest.is_empty());
        assert_eq!(nr.request_service_sid(), 0x40);
        assert_eq!(nr.request_service(), UdsServiceType::from_request_sid(0x40));
        let mut buf = [0u8; 2];
        let n = Encode::encode(&nr, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..n], &wire);
    }
}
