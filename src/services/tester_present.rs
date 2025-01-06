use crate::{Error, NegativeResponseCode, SuppressablePositiveResponse};

use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const TESTER_PRESENT_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 2] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
];

const NO_SUBFUNCTION_VALUE: u8 = 0x00;

// Subfunction parameter values for the Test Present service.
// The range of values is only 7 of the 8 bits, with bit 7 being used as the Suppress Positive Response (SPR) Message Indication Bit.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum ZeroSubFunction {
    // Request and response. Indicates that no value beside the (SPR) Message Indication Bit is supported by this service.
    NoSubFunctionSupported(u8),

    // Request only.
    ISOSAEReserved(u8),
}

impl ZeroSubFunction {
    fn new(value: u8) -> Result<Self, Error> {
        match value {
            NO_SUBFUNCTION_VALUE => Ok(ZeroSubFunction::NoSubFunctionSupported(value)),
            _ => Err(Error::InvalidTesterPresentType(value)),
        }
    }

    fn default() -> Self {
        ZeroSubFunction::NoSubFunctionSupported(NO_SUBFUNCTION_VALUE)
    }
}

impl From<ZeroSubFunction> for u8 {
    fn from(sub_function: ZeroSubFunction) -> Self {
        match sub_function {
            ZeroSubFunction::NoSubFunctionSupported(value) => value,
            ZeroSubFunction::ISOSAEReserved(value) => value,
        }
    }
}

impl TryFrom<u8> for ZeroSubFunction {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        ZeroSubFunction::new(value)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TesterPresentRequest {
    zero_sub_function: SuppressablePositiveResponse<ZeroSubFunction>,
}

impl TesterPresentRequest {
    /// Create a new TesterPresentRequest
    pub(crate) fn new(suppress_positive_response: bool) -> Self {
        Self::with_subfunction(suppress_positive_response, ZeroSubFunction::default())
    }

    fn with_subfunction(
        suppress_positive_response: bool,
        zero_sub_function: ZeroSubFunction,
    ) -> Self {
        Self {
            zero_sub_function: SuppressablePositiveResponse::new(
                suppress_positive_response,
                zero_sub_function,
            ),
        }
    }

    /// Getter for whether a positive response should be suppressed
    pub fn suppress_positive_response(&self) -> bool {
        self.zero_sub_function.suppress_positive_response()
    }

    /// Get the allowed Nack codes for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &TESTER_PRESENT_NEGATIVE_RESPONSE_CODES
    }

    /// Deserialization function to read a TesterPresentRequest from a `Reader`
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let zero_sub_function = SuppressablePositiveResponse::try_from(buffer.read_u8()?)?;
        Ok(Self { zero_sub_function })
    }

    /// Serialization function to write a TesterPresentRequest to a `Writer`
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.zero_sub_function))?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TesterPresentResponse {
    zero_sub_function: ZeroSubFunction,
}

impl TesterPresentResponse {
    /// Create a new TesterPresentResponse
    pub(crate) fn new() -> Self {
        Self {
            zero_sub_function: ZeroSubFunction::default(),
        }
    }

    /// Create a TesterPresentResponse from a sequence of bytes
    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let zero_sub_function = ZeroSubFunction::try_from(buffer.read_u8()?)?;
        Ok(Self { zero_sub_function })
    }

    /// Write the response as a sequence of bytes to a buffer
    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.zero_sub_function))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn try_from_all_zero_subfunction() {
        for i in 0..u8::MAX {
            let try_result: Result<ZeroSubFunction, Error> = ZeroSubFunction::try_from(i);
            match i {
                0x00 => {
                    assert_eq!(
                        try_result.unwrap(),
                        ZeroSubFunction::NoSubFunctionSupported(i)
                    )
                }
                0x01..=0xFF => {
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
        for i in 0..u8::MAX {
            match i {
                0x00 => {
                    assert_eq!(u8::from(ZeroSubFunction::default()), i);
                }
                0x01..=0x7F => {
                    let result = ZeroSubFunction::ISOSAEReserved(i);
                    assert_eq!(u8::from(result), i);
                }
                0x80 => {
                    let result = ZeroSubFunction::NoSubFunctionSupported(i);
                    assert_eq!(u8::from(result), i);
                }
                0x81..=0xFF => {
                    let result = ZeroSubFunction::ISOSAEReserved(i);
                    assert_eq!(u8::from(result), i);
                }
            }
        }
    }

    fn make_request(byte: u8) -> Result<TesterPresentRequest, Error> {
        let bytes = vec![byte];
        let mut byte_access = Cursor::new(bytes);
        TesterPresentRequest::read(&mut byte_access)
    }

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
                    assert!(matches!(result, Err(Error::InvalidTesterPresentType(_))));
                }
                0x80 => {
                    let expected = TesterPresentRequest::new(true);
                    assert_eq!(result.unwrap(), expected);
                }
                0x81..=0xFF => {
                    assert!(matches!(result, Err(Error::InvalidTesterPresentType(_))));
                }
            }
        }
    }

    #[test]
    fn write_request_type() {
        let test_type = TesterPresentRequest::new(false);
        let mut buffer = Vec::new();
        test_type.write(&mut buffer).unwrap();

        let expected_bytes = vec![0];
        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn read_response_type() {
        let bytes = vec![0u8];
        let mut byte_access = Cursor::new(bytes);
        let test_type = TesterPresentResponse::read(&mut byte_access).unwrap();
        assert_eq!(test_type, TesterPresentResponse::new());
    }

    #[test]
    fn write_response_type() {
        let test_type = TesterPresentResponse::new();
        let mut buffer = Vec::new();
        test_type.write(&mut buffer).unwrap();

        let expected_bytes = vec![0];
        assert_eq!(buffer, expected_bytes);
    }
}
