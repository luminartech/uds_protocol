use crate::{
    Error, NegativeResponseCode, SecurityAccessType, SingleValueWireFormat,
    SuppressablePositiveResponse, WireFormat,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityAccessRequest {
    access_type: SuppressablePositiveResponse<SecurityAccessType>,
    request_data: Vec<u8>,
}

impl SecurityAccessRequest {
    /// Create a new '`SecurityAccessRequest`'
    pub(crate) fn new(
        suppress_positive_response: bool,
        access_type: SecurityAccessType,
        request_data: Vec<u8>,
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
    pub fn request_data(&self) -> &[u8] {
        &self.request_data
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &SECURITY_ACCESS_NEGATIVE_RESPONSE_CODES
    }
}

impl WireFormat for SecurityAccessRequest {
    /// Deserialization function to read a [`SecurityAccessRequest`] from a `Reader`
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let access_type = SuppressablePositiveResponse::try_from(reader.read_u8()?)?;
        let mut request_data: Vec<u8> = Vec::new();
        _ = reader.read_to_end(&mut request_data)?;
        Ok(Some(Self {
            access_type,
            request_data,
        }))
    }

    fn required_size(&self) -> usize {
        1 + self.request_data().len()
    }

    /// Serialization function to write a [`SecurityAccessRequest`] to a `Writer`
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.access_type))?;
        writer.write_all(&self.request_data)?;
        Ok(self.required_size())
    }

    fn is_positive_response_suppressed(&self) -> bool {
        self.suppress_positive_response()
    }
}

impl SingleValueWireFormat for SecurityAccessRequest {}

/// Response to `SecurityAccessRequest`
///
/// ## Request Seed
///
/// When responding to a seed request, the `security_seed` field shall contain the seed value.
///
/// ## Send Key
///
/// The positive response to a `SendKey` request shall not have any data in the security seed field.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct SecurityAccessResponse {
    pub access_type: SecurityAccessType,
    pub security_seed: Vec<u8>,
}

impl SecurityAccessResponse {
    /// Create a new '`SecurityAccessResponse`'
    pub(crate) fn new(access_type: SecurityAccessType, security_seed: Vec<u8>) -> Self {
        Self {
            access_type,
            security_seed,
        }
    }
}

impl WireFormat for SecurityAccessResponse {
    /// Deserialization function to read a `SecurityAccessResponse` from a [`Reader`](std::io::Read)
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let access_type = SecurityAccessType::try_from(reader.read_u8()?)?;
        let mut security_seed = Vec::new();
        let _ = reader.read_to_end(&mut security_seed)?;
        Ok(Some(Self {
            access_type,
            security_seed,
        }))
    }

    fn required_size(&self) -> usize {
        1 + self.security_seed.len()
    }

    /// Serialization function to write a `SecurityAccessResponse` to a [`Writer`](std::io::Write)
    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.access_type))?;
        writer.write_all(&self.security_seed)?;
        Ok(self.required_size())
    }
}

impl SingleValueWireFormat for SecurityAccessResponse {}

#[cfg(test)]
mod request {
    use super::*;

    #[test]
    fn request_seed() {
        let bytes: [u8; 6] = [
            0x01, // aka SecurityAccessType::RequestSeed(0x01)
            0x00, 0x01, 0x02, 0x03, 0x04, // fake data
        ];
        let req = SecurityAccessRequest::from_reader(&mut bytes.as_slice()).unwrap();

        assert_eq!(
            req.access_type,
            SuppressablePositiveResponse::new(false, SecurityAccessType::RequestSeed(0x01))
        );

        let mut buf = Vec::new();
        let written = req.to_writer(&mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, req.required_size());
    }
}

#[cfg(test)]
mod response {
    use super::*;

    #[test]
    fn response_send() {
        let bytes: [u8; 6] = [
            0x02, // aka SecurityAccessType::SendKey(0x02)
            0x00, 0x01, 0x02, 0x03, 0x04, // fake data
        ];
        let resp = SecurityAccessResponse::from_reader(&mut bytes.as_slice()).unwrap();

        assert_eq!(resp.access_type, SecurityAccessType::SendKey(0x02));
        assert_eq!(resp.security_seed, vec![0x00, 0x01, 0x02, 0x03, 0x04]);

        let mut buf = Vec::new();
        let written = resp.to_writer(&mut buf).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, resp.required_size());
    }
}
