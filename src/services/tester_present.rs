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
// The range of values is only 6 of the 8 bits, with bit 7 being used as the Suppress Positive Response (SPR) Message Indication Bit.
// Setting the SPR bit is done at service request creation and NOT by specifying ZeroSubFunction::NoSubFunctionSupported(0x80).
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum ZeroSubFunction {
    // Request and response. Indicates that no value beside the (SPR) Message Indication Bit is supported by this service.
    NoSubFunctionSupported(u8),

    // Request only.
    ISOSAEReserved(u8),
}

impl ZeroSubFunction {
    fn default() -> Self {
        ZeroSubFunction::NoSubFunctionSupported(NO_SUBFUNCTION_VALUE)
    }

    fn iso_sae_reserved(value: u8) -> Result<Self, Error> {
        match value {
            0x01..=0x7F => Ok(ZeroSubFunction::ISOSAEReserved(value)),
            _ => Err(Error::InvalidTestPresetType(value)),
        }
    }
}

impl From<ZeroSubFunction> for u8 {
    fn from(sub_function: ZeroSubFunction) -> Self {
        match sub_function {
            ZeroSubFunction::NoSubFunctionSupported(_) => NO_SUBFUNCTION_VALUE,
            ZeroSubFunction::ISOSAEReserved(value) => value,
        }
    }
}

impl TryFrom<u8> for ZeroSubFunction {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(ZeroSubFunction::default()),
            _ => ZeroSubFunction::iso_sae_reserved(value),
        }
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
    pub(crate) fn new() -> Self {
        Self {
            zero_sub_function: ZeroSubFunction::default(),
        }
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let zero_sub_function = ZeroSubFunction::try_from(buffer.read_u8()?)?;
        Ok(Self { zero_sub_function })
    }

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
            let result: Result<ZeroSubFunction, Error> = ZeroSubFunction::try_from(i);
            match i {
                0 => assert_eq!(ZeroSubFunction::default(), result.unwrap()),
                1..=0x7F => {
                    assert_eq!(
                        ZeroSubFunction::iso_sae_reserved(i).unwrap(),
                        result.unwrap()
                    );
                }
                // This should never happen as ZeroSubFunction is defined as 6 bytes
                0x80..=0xFF => {
                    let error = ZeroSubFunction::iso_sae_reserved(i).unwrap_err();
                    match error {
                        Error::InvalidTestPresetType(value) => assert_eq!(value, i),
                        _ => assert!(false, "Invalid error, expected InvalidTestPresetType."),
                    }
                }
            }
        }
    }

    #[test]
    fn from_all_zero_subfunction() {
        for i in 1..u8::MAX {
            match i {
                0 => {
                    assert_eq!(u8::from(ZeroSubFunction::NoSubFunctionSupported(i)), i);
                }
                1..=0x7F => {
                    let result = ZeroSubFunction::iso_sae_reserved(i);
                    assert_eq!(u8::from(result.unwrap()), i);
                }
                // This should never happen as ZeroSubFunction is defined as 6 bytes
                0x80..=0xFF => {
                    let result = ZeroSubFunction::iso_sae_reserved(i);
                    let error = result.unwrap_err();
                    match error {
                        Error::InvalidTestPresetType(value) => assert_eq!(value, i),
                        _ => assert!(false, "Invalid error, expected InvalidTestPresetType."),
                    }
                }
            }
        }
    }

    #[test]
    fn read_request_type() {
        let bytes = vec![0 as u8];
        let mut byte_access = Cursor::new(bytes);
        let test_type = TesterPresentRequest::read(&mut byte_access).unwrap();
        assert_eq!(test_type, TesterPresentRequest::new(false));
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
        let bytes = vec![0 as u8];
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
