use crate::Error;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Format and length of this parameter(s) are vehicle manufacturer specific
pub struct TransferRequestParameter {
    /// Memory address (start) to deliver data to
    pub memory_address: u32,
    /// compression method (high bit) and encrypting method (low 7 bits)
    pub data_format_identifier: u8,
    /// Shall be used by the server to compare to the actual number of bytes transferred 
    /// during execution of [`RequestTransferExit`](crate::Request::RequestTransferExit)
    pub memory_size: u32
}

impl TransferRequestParameter {
    /// Deserialization function to read a [`TransferRequestParameter`] from a `Reader`
    pub fn read<T: Read>(buffer: &mut T) -> Result<Option<Self>, Error> {
        // Read the memory address, using `read` instead of `read_exact` (via read_u24) due to the 
        // possibility of the buffer being empty and the need for the error to be thrown ONLY when 
        // the buffer is partially empty
        let mut memory_address_bytes: [u8; 3] = [0; 3];
        let memory_address = match buffer.read(&mut memory_address_bytes) {
            Ok(0) => return Ok(None),
            Ok(n) => {
                if n != 3 {
                    return Err(Error::IncorrectMessageLengthOrInvalidFormat);
                }
                else {
                    // squish memory_address_bytes into a u32
                    let mut cursor = std::io::Cursor::new(memory_address_bytes);
                    cursor.read_u24::<BigEndian>()?
                }
            }
            Err(e) => return Err(Error::from(e)),
        };
        let data_format_identifier = buffer.read_u8()?;
        let memory_size = buffer.read_u24::<BigEndian>()?;
        Ok(Some(Self {
            memory_address,
            data_format_identifier,
            memory_size
        }))
    }

    /// Serialization function to write a [`TransferRequestParameter`] to a `Writer`
    pub fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u24::<BigEndian>(self.memory_address)?;
        buffer.write_u8(self.data_format_identifier)?;
        buffer.write_u24::<BigEndian>(self.memory_size)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_request_parameter_valid() {
        let bytes = [
            0x00, 0x00, 0x01, 
            0x02, 
            0x00, 0x00, 0x01
        ];
        let mut cursor = std::io::Cursor::new(&bytes);
        let transfer_request_parameter = TransferRequestParameter::read(&mut cursor).unwrap().unwrap();
        assert_eq!(transfer_request_parameter.memory_address, 1);
        assert_eq!(transfer_request_parameter.data_format_identifier, 2);
        assert_eq!(transfer_request_parameter.memory_size, 1);
        let mut buffer = Vec::new();
        transfer_request_parameter.write(&mut buffer).unwrap();
        assert_eq!(buffer, bytes);
    }

    #[test]
    fn partial_transfer_request_parameter() {
        let bytes = [
            0x00, 0x00
        ];
        let mut cursor = std::io::Cursor::new(&bytes);
        let transfer_request_parameter = 
            TransferRequestParameter::read(&mut cursor);

        assert!(matches!(transfer_request_parameter, Err(Error::IncorrectMessageLengthOrInvalidFormat)));
    }

    #[test]
    fn empty_transfer_request_parameter() {
        let bytes = [];
        let mut cursor = std::io::Cursor::new(&bytes);
        let transfer_request_parameter = 
            TransferRequestParameter::read(&mut cursor);

        assert!(matches!(transfer_request_parameter, Ok(None)));
    }

    fn parse_transfer_request_parameters(bytes: &[u8]) -> Result<Vec<TransferRequestParameter>, Error> {
        let mut cursor = std::io::Cursor::new(bytes);
        let mut transfer_request_parameters = Vec::new();
        while let Some(transfer_request_parameter) = TransferRequestParameter::read(&mut cursor)? {
            transfer_request_parameters.push(transfer_request_parameter);
        }
        Ok(transfer_request_parameters)
    }

    #[test]
    fn multiple_valid_requests() {
        let bytes = [
            0x00, 0x00, 0x01, 
            0x02, 
            0x00, 0x00, 0x01,
            0x00, 0x00, 0x02, 
            0x03, 
            0x00, 0x00, 0x03
        ];

        let transfer_request_parameters = parse_transfer_request_parameters(&bytes).unwrap();
        assert_eq!(transfer_request_parameters.len(), 2);
        assert_eq!(transfer_request_parameters[0].memory_address, 1);
        assert_eq!(transfer_request_parameters[0].data_format_identifier, 2);
        assert_eq!(transfer_request_parameters[0].memory_size, 1);
        assert_eq!(transfer_request_parameters[1].memory_address, 2);
        assert_eq!(transfer_request_parameters[1].data_format_identifier, 3);
        assert_eq!(transfer_request_parameters[1].memory_size, 3);
    }

    #[test]
    fn multiple_requests_partial() {
        use std::io::ErrorKind;
        let bytes = [
            0x00, 0x00, 0x01, 
            0x02, 
            0x00, 0x00, 0x01,
            0x00, 0x00, 0x02, 
            0x03, 
            0x00, 0x00
        ];

        let transfer_request_parameters = parse_transfer_request_parameters(&bytes);

        let my_error = transfer_request_parameters.unwrap_err();
        let is_unexpected_eof = match my_error {
            Error::IoError(e) => match e.kind() {
                ErrorKind::UnexpectedEof => true,
                _ => false
            },
            _ => false
        };
        assert!(is_unexpected_eof, "Error was not UnexpectedEof");
    }
}
