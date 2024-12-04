use crate::{Error, NegativeResponseCode, SecurityAccessType, SuppressablePositiveResponse};
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const SECURITY_ACCESS_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::SecurityAccessDenied,
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Request for the server to reset the ECU
pub struct SecurityAccessRequest {
    access_type: SuppressablePositiveResponse<SecurityAccessType>,
    data_record: Vec<u8>,
}

impl SecurityAccessRequest {
    /// Create a new 'SecurityAccessRequest'
    pub(crate) fn new(
        suppress_positive_response: bool,
        access_type: SecurityAccessType,
        data_record: Vec<u8>,
    ) -> Self {
        Self {
            access_type: SuppressablePositiveResponse::new(suppress_positive_response, access_type),
            data_record,
        }
    }

    /// Getter for whether a positive response should be suppressed
    pub fn suppress_positive_response(&self) -> bool {
        self.access_type.suppress_positive_response()
    }

    /// Getter for the requested [`SecurityAccessType`]
    pub fn access_type(&self) -> SecurityAccessType {
        self.access_type.value()
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &SECURITY_ACCESS_NEGATIVE_RESPONSE_CODES
    }

    /// Deserialization function to read a [`SecurityAccessRequest`] from a `Reader`
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let access_type = SuppressablePositiveResponse::try_from(buffer.read_u8()?)?;
        let mut data_record: Vec<u8> = Vec::new();
        _ = buffer.read_to_end(&mut data_record)?;
        Ok(Self {
            access_type,
            data_record,
        })
    }

    /// Serialization function to write a [`SecurityAccessRequest`] to a `Writer`
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.access_type))?;
        buffer.write_all(&self.data_record)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct SecurityAccessResponse {
    pub access_type: SecurityAccessType,
    pub security_seed: Vec<u8>,
}

impl SecurityAccessResponse {
    /// Create a new 'SecurityAccessResponse'
    pub(crate) fn new(access_type: SecurityAccessType, security_seed: Vec<u8>) -> Self {
        Self {
            access_type,
            security_seed,
        }
    }

    /// Deserialization function to read a [`SecurityAccessResponse`] from a [`Reader`](std::io::Read)
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let access_type = SecurityAccessType::try_from(buffer.read_u8()?)?;
        let mut security_seed = Vec::new();
        let _ = buffer.read_to_end(&mut security_seed)?;
        Ok(Self {
            access_type,
            security_seed,
        })
    }

    /// Serialization function to write a [`SecurityAccessResponse`] to a [`Writer`](std::io::Write)
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.access_type))?;
        buffer.write_all(&self.security_seed)?;
        Ok(())
    }
}
