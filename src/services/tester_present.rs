use crate::{
    Error, NegativeResponseCode, SingleValueWireFormat, SuppressablePositiveResponse, WireFormat,
};

use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

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
    NoSubFunctionSupported,
    // Request only.
    ISOSAEReserved(u8),
}

impl ZeroSubFunction {
    #[inline]
    fn new() -> Self {
        Self::default()
    }
}

impl Default for ZeroSubFunction {
    #[inline]
    fn default() -> Self {
        ZeroSubFunction::NoSubFunctionSupported
    }
}

impl From<ZeroSubFunction> for u8 {
    fn from(sub_function: ZeroSubFunction) -> Self {
        match sub_function {
            ZeroSubFunction::NoSubFunctionSupported => NO_SUBFUNCTION_VALUE,
            ZeroSubFunction::ISOSAEReserved(value) => value,
        }
    }
}

impl TryFrom<u8> for ZeroSubFunction {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            NO_SUBFUNCTION_VALUE => Ok(ZeroSubFunction::NoSubFunctionSupported),
            0x01..=0x7F => Ok(ZeroSubFunction::ISOSAEReserved(value)),
            _ => Err(Error::InvalidTesterPresentType(value)),
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
        Self::with_subfunction(suppress_positive_response, ZeroSubFunction::new())
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
}

impl WireFormat<Error> for TesterPresentRequest {
    /// Deserialization function to read a TesterPresentRequest from a `Reader`
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let zero_sub_function = SuppressablePositiveResponse::try_from(reader.read_u8()?)?;
        Ok(Some(Self { zero_sub_function }))
    }

    /// Serialization function to write a TesterPresentRequest to a `Writer`
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.zero_sub_function))?;
        Ok(1)
    }
}

impl SingleValueWireFormat<Error> for TesterPresentRequest {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TesterPresentResponse {
    zero_sub_function: ZeroSubFunction,
}

impl TesterPresentResponse {
    /// Create a new TesterPresentResponse
    pub(crate) fn new() -> Self {
        Self {
            zero_sub_function: ZeroSubFunction::new(),
        }
    }
}

impl WireFormat<Error> for TesterPresentResponse {
    /// Create a TesterPresentResponse from a sequence of bytes
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let zero_sub_function = ZeroSubFunction::try_from(reader.read_u8()?)?;
        Ok(Some(Self { zero_sub_function }))
    }

    /// Write the response as a sequence of bytes to a buffer
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(u8::from(self.zero_sub_function))?;
        Ok(1)
    }
}

impl SingleValueWireFormat<Error> for TesterPresentResponse {}

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
                    assert_eq!(try_result.unwrap(), ZeroSubFunction::NoSubFunctionSupported)
                }
                0x01..=0x7F => {
                    assert!(matches!(try_result, Ok(ZeroSubFunction::ISOSAEReserved(_))));
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
            let result = ZeroSubFunction::ISOSAEReserved(i);
            assert_eq!(u8::from(result), i);
        }
    }

    fn make_request(byte: u8) -> Result<Option<TesterPresentRequest>, Error> {
        let bytes = vec![byte];
        let mut byte_access = Cursor::new(bytes);
        TesterPresentRequest::option_from_reader(&mut byte_access)
    }

    #[test]
    fn read_request_type() {
        for i in 0..u8::MAX {
            let result = make_request(i);
            match i {
                0x00 => {
                    let expected = TesterPresentRequest::new(false);
                    assert_eq!(result.unwrap().unwrap(), expected);
                }
                0x01..=0x7F => {
                    let result = result.unwrap().unwrap();
                    assert!(!result.suppress_positive_response());
                    assert!(matches!(
                        result.zero_sub_function.value(),
                        ZeroSubFunction::ISOSAEReserved(_)
                    ));
                }
                0x80 => {
                    let expected = TesterPresentRequest::new(true);
                    assert_eq!(result.unwrap().unwrap(), expected);
                }
                0x81..=0xFF => {
                    let result = result.unwrap().unwrap();
                    assert!(result.suppress_positive_response());
                    assert!(matches!(
                        result.zero_sub_function.value(),
                        ZeroSubFunction::ISOSAEReserved(_)
                    ));
                }
            }
        }
    }

    #[test]
    fn write_request_type() {
        let test_type = TesterPresentRequest::new(false);
        let mut buffer = Vec::new();
        test_type.to_writer(&mut buffer).unwrap();

        let expected_bytes = vec![0];
        assert_eq!(buffer, expected_bytes);
    }

    #[test]
    fn read_response_type() {
        let bytes = vec![0u8];
        let mut byte_access = Cursor::new(bytes);
        let test_type = TesterPresentResponse::option_from_reader(&mut byte_access)
            .unwrap()
            .unwrap();
        assert_eq!(test_type, TesterPresentResponse::new());
    }

    #[test]
    fn write_response_type() {
        let test_type = TesterPresentResponse::new();
        let mut buffer = Vec::new();
        test_type.to_writer(&mut buffer).unwrap();

        let expected_bytes = vec![0];
        assert_eq!(buffer, expected_bytes);
    }
}
