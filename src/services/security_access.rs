//! `SecurityAccess` (0x27) service implementation
use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, NegativeResponseCode};

/// Security Access Type allows for multiple different security challenges within an ECU.
///
/// The Security Access Type is used to determine both the sub function,
/// as well as ECU specific access type being requested
///
/// *Note*:
///
/// Conversions from `u8` to `SecurityAccessType` are fallible and will return an [`Error`] if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SecurityAccessType {
    /// This value is reserved for future definition
    ISOSAEReserved(u8),
    /// `RequestSeed` with the level of security defined by the vehicle manufacturer
    RequestSeed(u8),
    /// `SendKey` with the level of security defined by the vehicle manufacturer
    SendKey(u8),
    /// `RequestSeed` with different levels of security defined for end of life
    /// activation of on-board pyrotechnic devices
    ISO26021_2Values,
    /// `SendKey` with different levels of security defined for end of life activation
    ISO26021_2SendKeyValues,
    /// This range of values is reserved for system supplier specific use
    SystemSupplierSpecific(u8),
}

impl From<SecurityAccessType> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: SecurityAccessType) -> Self {
        match value {
            SecurityAccessType::ISOSAEReserved(val) => val,
            SecurityAccessType::RequestSeed(val) => val,
            SecurityAccessType::SendKey(val) => val,
            SecurityAccessType::ISO26021_2Values => 0x5F,
            SecurityAccessType::ISO26021_2SendKeyValues => 0x60,
            SecurityAccessType::SystemSupplierSpecific(val) => val,
        }
    }
}

impl TryFrom<u8> for SecurityAccessType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 | 0x43..=0x5E | 0x7F => Ok(Self::ISOSAEReserved(value)),
            // Security requests alternate, with odd numbers being seed requests,
            // and even numbers being send key requests
            0x01..=0x42 => {
                if value % 2 == 1 {
                    Ok(Self::RequestSeed(value))
                } else {
                    Ok(Self::SendKey(value))
                }
            }
            0x5F => Ok(Self::ISO26021_2Values),
            0x60 => Ok(Self::ISO26021_2SendKeyValues),
            0x61..=0x7E => Ok(Self::SystemSupplierSpecific(value)),
            _ => Err(Error::InvalidSecurityAccessType(value)),
        }
    }
}

#[cfg(test)]
mod security_access_type_tests {
    use super::*;

    const REQUEST_SEED_VALUES: [u8; 33] = [
        0x01, 0x03, 0x05, 0x07, 0x09, 0x0B, 0x0D, 0x0F, 0x11, 0x13, 0x15, 0x17, 0x19, 0x1B, 0x1D,
        0x1F, 0x21, 0x23, 0x25, 0x27, 0x29, 0x2B, 0x2D, 0x2F, 0x31, 0x33, 0x35, 0x37, 0x39, 0x3B,
        0x3D, 0x3F, 0x41,
    ];
    const SEND_KEY_VALUES: [u8; 33] = [
        0x02, 0x04, 0x06, 0x08, 0x0A, 0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A, 0x1C, 0x1E,
        0x20, 0x22, 0x24, 0x26, 0x28, 0x2A, 0x2C, 0x2E, 0x30, 0x32, 0x34, 0x36, 0x38, 0x3A, 0x3C,
        0x3E, 0x40, 0x42,
    ];
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn security_access_type_from_all_u8_values() {
        assert_eq!(
            SecurityAccessType::try_from(0).unwrap(),
            SecurityAccessType::ISOSAEReserved(0)
        );
        for value in &REQUEST_SEED_VALUES {
            assert_eq!(
                SecurityAccessType::try_from(*value).unwrap(),
                SecurityAccessType::RequestSeed(*value)
            );
        }
        for value in &SEND_KEY_VALUES {
            assert_eq!(
                SecurityAccessType::try_from(*value).unwrap(),
                SecurityAccessType::SendKey(*value)
            );
        }
        for i in 0x43..=0x5E {
            assert_eq!(
                SecurityAccessType::try_from(i).unwrap(),
                SecurityAccessType::ISOSAEReserved(i)
            );
        }
        assert_eq!(
            SecurityAccessType::try_from(0x5F).unwrap(),
            SecurityAccessType::ISO26021_2Values
        );
        assert_eq!(
            SecurityAccessType::try_from(0x60).unwrap(),
            SecurityAccessType::ISO26021_2SendKeyValues
        );
        for i in 0x61..=0x7E {
            assert_eq!(
                SecurityAccessType::try_from(i).unwrap(),
                SecurityAccessType::SystemSupplierSpecific(i)
            );
        }
        assert_eq!(
            SecurityAccessType::try_from(0x7F).unwrap(),
            SecurityAccessType::ISOSAEReserved(0x7F)
        );
        for i in 0x80..=0xFF {
            match SecurityAccessType::try_from(i).unwrap_err() {
                Error::InvalidSecurityAccessType(value) => assert_eq!(value, i),
                _ => panic!("Invalid error type"),
            }
        }
    }

    #[test]
    fn security_access_type_round_trip_all_values() {
        for i in 0..=u8::MAX {
            let value = SecurityAccessType::try_from(i);
            match value {
                Ok(value) => assert_eq!(u8::from(value), i),
                Err(Error::InvalidSecurityAccessType(value)) => assert_eq!(value, i),
                _ => panic!("Invalid error type"),
            }
        }
    }
}

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
/// Zero-alloc request for security access. Borrows from the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityAccessRequest<'d> {
    access_type: SuppressablePositiveResponse<SecurityAccessType>,
    request_data: &'d [u8],
}

impl<'d> SecurityAccessRequest<'d> {
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

impl Encode for SecurityAccessRequest<'_> {
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

impl<'a> Decode<'a> for SecurityAccessRequest<'a> {
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

/// Zero-alloc response for security access. Borrows from the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityAccessResponse<'d> {
    /// The security access type echoed from the request.
    pub access_type: SecurityAccessType,
    /// The security seed bytes (empty for a `SendKey` positive response).
    pub security_seed: &'d [u8],
}

impl<'d> SecurityAccessResponse<'d> {
    /// Create a new security access response.
    #[must_use]
    pub const fn new(access_type: SecurityAccessType, security_seed: &'d [u8]) -> Self {
        Self {
            access_type,
            security_seed,
        }
    }
}

impl Encode for SecurityAccessResponse<'_> {
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

impl<'a> Decode<'a> for SecurityAccessResponse<'a> {
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
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn request_seed() {
        let bytes: [u8; 6] = [
            0x01, // aka SecurityAccessType::RequestSeed(0x01)
            0x00, 0x01, 0x02, 0x03, 0x04, // fake data
        ];
        let (req, _) = <SecurityAccessRequest as Decode>::decode(&bytes).unwrap();

        assert_eq!(req.access_type(), SecurityAccessType::RequestSeed(0x01));
        assert_eq!(req.request_data(), &[0x00, 0x01, 0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = Encode::encode(&req, &mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, req.encoded_size());
        assert_encode_size_agrees(&req);
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn response_send() {
        let bytes: [u8; 6] = [
            0x02, // aka SecurityAccessType::SendKey(0x02)
            0x00, 0x01, 0x02, 0x03, 0x04, // fake data
        ];
        let (resp, _) = <SecurityAccessResponse as Decode>::decode(&bytes).unwrap();

        assert_eq!(resp.access_type, SecurityAccessType::SendKey(0x02));
        assert_eq!(resp.security_seed, &[0x00, 0x01, 0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = Encode::encode(&resp, &mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, resp.encoded_size());
        assert_encode_size_agrees(&resp);
    }
}
