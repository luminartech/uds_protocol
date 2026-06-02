//! `SecurityAccess` (0x27) service implementation
use crate::{
    Decode, Encode, Error, NegativeResponseCode, SecurityAccessType, SuppressablePositiveResponse,
};

/// List of allowed [`NegativeResponseCode`] variants for the `SecurityAccess` service
const SECURITY_ACCESS_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 8] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestSequenceError,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::InvalidKey,
    NegativeResponseCode::ExceedNumberOfAttempts,
    NegativeResponseCode::RequiredTimeDelayNotExpired,
];

/// Client request to access a security level
///
/// This service supports two primary types of request:
///
/// ## Request Seed
///
/// When requesting a seed, the request data represents implementation defined
/// `SecurityAccessDataRecord` values.
/// This data is optional, and its use is implementation defined.
/// Suppressing a positive response to this request is not supported.
///
/// ## Send Key
///
/// When sending a key, the request data represents the key to be sent.
/// After receiving a seed,
/// the client must calculate the corresponding key and send it to the server.
/// The server will then validate the key and respond with a positive or negative response.
/// Successful verification of the key will result in the server unlocking the requested security level.
/// Suppressing a positive response to this request is allowed.
///
/// Zero-alloc TX request for security access. Borrows from the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityAccessRequestTx<'d> {
    access_type: SuppressablePositiveResponse<SecurityAccessType>,
    request_data: &'d [u8],
}

impl<'d> SecurityAccessRequestTx<'d> {
    /// Create a new security access request.
    #[must_use]
    pub const fn new(
        suppress_positive_response: bool,
        access_type: SecurityAccessType,
        request_data: &'d [u8],
    ) -> Self {
        Self {
            access_type: SuppressablePositiveResponse::new(suppress_positive_response, access_type),
            request_data,
        }
    }

    /// Getter for whether a positive response should be suppressed
    #[must_use]
    pub fn suppress_positive_response(&self) -> bool {
        self.access_type.suppress_positive_response()
    }

    /// Getter for the requested [`SecurityAccessType`]
    #[must_use]
    pub fn access_type(&self) -> SecurityAccessType {
        self.access_type.value()
    }

    /// Getter for the request data
    #[must_use]
    pub const fn request_data(&self) -> &[u8] {
        self.request_data
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &SECURITY_ACCESS_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for SecurityAccessRequestTx<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.request_data.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.access_type)])
            .map_err(Error::io)?;
        writer.write_all(self.request_data).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for SecurityAccessRequestTx<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let access_type = SuppressablePositiveResponse::try_from(buf[0])?;
        Ok((
            Self {
                access_type,
                request_data: &buf[1..],
            },
            &[],
        ))
    }
}

/// Zero-alloc TX response for security access. Borrows from the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityAccessResponseTx<'d> {
    /// The security access type echoed from the request.
    pub access_type: SecurityAccessType,
    /// The security seed bytes (empty for a `SendKey` positive response).
    pub security_seed: &'d [u8],
}

impl<'d> SecurityAccessResponseTx<'d> {
    /// Create a new security access response.
    #[must_use]
    pub const fn new(access_type: SecurityAccessType, security_seed: &'d [u8]) -> Self {
        Self {
            access_type,
            security_seed,
        }
    }
}

impl Encode for SecurityAccessResponseTx<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.security_seed.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.access_type)])
            .map_err(Error::io)?;
        writer.write_all(self.security_seed).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for SecurityAccessResponseTx<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let access_type = SecurityAccessType::try_from(buf[0])?;
        Ok((
            Self {
                access_type,
                security_seed: &buf[1..],
            },
            &[],
        ))
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, Encode};

    #[test]
    fn request_seed() {
        let bytes: [u8; 6] = [
            0x01, // aka SecurityAccessType::RequestSeed(0x01)
            0x00, 0x01, 0x02, 0x03, 0x04, // fake data
        ];
        let (req, _) = <SecurityAccessRequestTx as Decode>::decode(&bytes).unwrap();

        assert_eq!(req.access_type(), SecurityAccessType::RequestSeed(0x01));
        assert_eq!(req.request_data(), &[0x00, 0x01, 0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = Encode::encode(&req, &mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, req.encoded_size());
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode};

    #[test]
    fn response_send() {
        let bytes: [u8; 6] = [
            0x02, // aka SecurityAccessType::SendKey(0x02)
            0x00, 0x01, 0x02, 0x03, 0x04, // fake data
        ];
        let (resp, _) = <SecurityAccessResponseTx as Decode>::decode(&bytes).unwrap();

        assert_eq!(resp.access_type, SecurityAccessType::SendKey(0x02));
        assert_eq!(resp.security_seed, &[0x00, 0x01, 0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = Encode::encode(&resp, &mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, resp.encoded_size());
    }
}
