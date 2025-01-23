use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::Read;
use serde::{Deserialize, Serialize};

use crate::{DataFormatIdentifier, Error, SingleValueWireFormat, WireFormat};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum FileOperationMode {
    // 0x00, 0x07-0xFF Reserved for future definition by ISO
    ISOSAEReserved(u8),
    /// Add a file to the server
    AddFile = 0x01,
    /// Delete the specified file from the server
    DeleteFile = 0x02,
    /// Replace the specified file on the server, if it does not exist, add it
    ReplaceFile = 0x03,
    /// Read the specified file from the server (upload)
    ReadFile = 0x04,
    /// Read the directory from the server
    /// Implies that the request does not include a `fileName`
    ReadDir = 0x05,
    /// Resume a file transfer at the returned `filePosition` indicator
    /// The file must already exist in the ECU's filesystem
    ResumeFile = 0x06,
}

impl From<FileOperationMode> for u8 {
    fn from(value: FileOperationMode) -> Self {
        match value {
            FileOperationMode::ISOSAEReserved(value) => value,
            FileOperationMode::AddFile => 0x01,
            FileOperationMode::DeleteFile => 0x02,
            FileOperationMode::ReplaceFile => 0x03,
            FileOperationMode::ReadFile => 0x04,
            FileOperationMode::ReadDir => 0x05,
            FileOperationMode::ResumeFile => 0x06,
        }
    }
}

impl TryFrom<u8> for FileOperationMode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::AddFile),
            0x02 => Ok(Self::DeleteFile),
            0x03 => Ok(Self::ReplaceFile),
            0x04 => Ok(Self::ReadFile),
            0x05 => Ok(Self::ReadDir),
            0x06 => Ok(Self::ResumeFile),
            0x00 | 0x07..=0xFF => Ok(Self::ISOSAEReserved(value)),
        }
    }
}

/// A request to the server to transfer a file, either upload or download.
/// 
/// Capabilities:
///   * Receive information about the file system on the server
///   * Send/Receive files to/from the server
/// 
/// Available as an alternative to [`crate::RequestDownloadRequest`] and [`crate::RequestUploadRequest`]
/// if the server implements a file system for data storage
/// 
/// Use [`crate::UdsServiceType::TransferData`] to send the file data to the server and [`crate::UdsServiceType::RequestTransferExit`] to end the transfer
/// 
/// If this service is used to delete files or directories on the server, 
/// there is no need to use the TransferData or [`crate::UdsServiceType::RequestTransferExit`] services.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[non_exhaustive]
pub struct RequestFileTransferRequest {
    /// 0x01 - 0x06, the type of operation to be applied to the file or directory specified in `file_path_and_name`
    pub mode_of_operation: FileOperationMode,

    /// Length in bytes of the `file_path_and_name` field
    pub file_path_and_name_length: u16,

    /// The path and name of the file or directory on the server
    pub file_path_and_name: String,

    /// compression method and encrypting method. 0x00 is no compression or encryption
    /// Not included in the request message if `mode_of_operation` is `DeleteFile` (0x02) or `ReadDir` (0x05)
    data_format_identifier: DataFormatIdentifier,

    // Length in bytes for both `file_size_uncompressed` and `file_size_compressed`
    /// Not included in the request message if `mode_of_operation` is one of:
    ///     * `DeleteFile` (0x02) 
    ///     * `ReadFile` (0x04) 
    ///     * `ReadDir` (0x05)
    pub file_size_parameter_length: u8,

    /// Specifies the size of the uncompressed file in bytes.
    /// Not included in the request message if `mode_of_operation` is one of:
    ///     * `DeleteFile` (0x02) 
    ///     * `ReadFile` (0x04) 
    ///     * `ReadDir` (0x05)
    pub file_size_uncompressed: u128,

    /// Specifies the size of the compressed file in bytes
    /// Not included in the request message if `mode_of_operation` is one of:
    ///     * `DeleteFile` (0x02) 
    ///     * `ReadFile` (0x04) 
    ///     * `ReadDir` (0x05)
    pub file_size_compressed: u128,
}

impl RequestFileTransferRequest {
    fn write_file_sizes<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let mut len: usize= 0;
        // file_size_parameter_length should be a power of 2

        // Ensure file_size_parameter_length is a power of 2
        match self.file_size_parameter_length {
            1 => {
                // Dependent size: `file_size_parameter_length` bytes
                writer.write_u8(1)?;
                writer.write_u8(self.file_size_uncompressed as u8)?;
                writer.write_u8(self.file_size_compressed as u8)?;
                len += 3;
            }
            2 => {
                writer.write_u8(2)?;
                writer.write_u16::<byteorder::BigEndian>(self.file_size_uncompressed as u16)?;
                writer.write_u16::<byteorder::BigEndian>(self.file_size_compressed as u16)?;
                len += 5;
            }
            3..=4 => {
                writer.write_u8(4)?;
                writer.write_u32::<byteorder::BigEndian>(self.file_size_uncompressed as u32)?;
                writer.write_u32::<byteorder::BigEndian>(self.file_size_compressed as u32)?;
                len += 9;
            }
            5..=8 => {
                writer.write_u8(8)?;
                writer.write_u64::<byteorder::BigEndian>(self.file_size_uncompressed as u64)?;
                writer.write_u64::<byteorder::BigEndian>(self.file_size_compressed as u64)?;
                len += 17;
            }
            9..=16 => {
                writer.write_u8(16)?;
                writer.write_u128::<byteorder::BigEndian>(self.file_size_uncompressed)?;
                writer.write_u128::<byteorder::BigEndian>(self.file_size_compressed)?;
                len += 33;
            }
            _ => return Err(Error::InvalidFileSizeParameterLength(self.file_size_parameter_length)),
        };

        Ok(len)
    }
}

impl WireFormat for RequestFileTransferRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mode_of_operation = FileOperationMode::try_from(reader.read_u8()?)?;
        let file_path_and_name_length = reader.read_u16::<byteorder::BigEndian>()?;

        // Read # of bytes specified by `file_path_and_name_length`
        let mut file_path_and_name = String::new();
        reader.take(file_path_and_name_length as u64)
            .read_to_string(&mut file_path_and_name)?;

        // If the mode of operation is DeleteFile or ReadDir, the data format identifier is not included
        // zero it out and don't use read
        let data_format_identifier = {
            if mode_of_operation == FileOperationMode::DeleteFile || mode_of_operation == FileOperationMode::ReadDir {
                DataFormatIdentifier::new(0, 0).unwrap()
            } else {
            DataFormatIdentifier::from(reader.read_u8()?)
            }
        };

        let mut file_size_parameter_length = 0;
        let mut file_size_uncompressed = Vec::new();
        let mut file_size_compressed = Vec::new();

        // If the mode of operation is DeleteFile, ReadFile, or ReadDir, the file size parameters are not included
        if mode_of_operation != FileOperationMode::DeleteFile
            && mode_of_operation != FileOperationMode::ReadFile
            && mode_of_operation != FileOperationMode::ReadDir
        {
            file_size_parameter_length = reader.read_u8()?;

            file_size_uncompressed = vec![0; file_size_parameter_length as usize];
            file_size_compressed = vec![0; file_size_parameter_length as usize];
            reader.read_exact(&mut file_size_uncompressed)?;
            reader.read_exact(&mut file_size_compressed)?;
        }


        Ok(Some(Self {
            mode_of_operation,
            file_path_and_name_length,
            file_path_and_name,
            data_format_identifier,
            file_size_parameter_length,
            file_size_uncompressed: u128::from_be_bytes({
                let mut bytes = [0; 16];
                bytes[16 - file_size_parameter_length as usize..].copy_from_slice(&file_size_uncompressed);
                bytes
            }),
            file_size_compressed: u128::from_be_bytes({
                let mut bytes = [0; 16];
                bytes[16 - file_size_parameter_length as usize..].copy_from_slice(&file_size_compressed);
                bytes
            }),
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let mut len = 0;

        // Fixed size: 1 byte
        writer.write_u8(self.mode_of_operation.into())?;
        len += 1;

        // Fixed size: 2 bytes
        writer.write_u16::<byteorder::BigEndian>(self.file_path_and_name_length)?;
        len += 2;

        // Dependent size: `file_path_and_name_length` bytes
        writer.write_all(self.file_path_and_name.as_bytes())?;
        len += self.file_path_and_name_length as usize;

        // If the mode of operation is DeleteFile or ReadDir, the data format identifier is not included
        // Fixed size: 1 byte
        if self.mode_of_operation != FileOperationMode::DeleteFile && self.mode_of_operation != FileOperationMode::ReadDir {
            writer.write_u8(self.data_format_identifier.into())?;
            len += 1;
        }


        // If the mode of operation is DeleteFile, ReadFile, or ReadDir, the file size parameters are not included
        if self.mode_of_operation != FileOperationMode::DeleteFile
            && self.mode_of_operation != FileOperationMode::ReadFile
            && self.mode_of_operation != FileOperationMode::ReadDir
        {
            len += self.write_file_sizes(writer)?;
        }

        Ok(len)
    }
}

impl SingleValueWireFormat for RequestFileTransferRequest {}

#[cfg(test)]
mod tests {
    use super::*;

    // helper function to get some bytes to read from
    fn get_bytes(mode: FileOperationMode, file_name: &str, file_size: u128) -> Vec<u8> {

        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(mode.into()); // AddFile (u8)
        // write file_name len as 2 bytes
        bytes.write_u16::<byteorder::BigEndian>(file_name.len() as u16).unwrap();
        bytes.extend_from_slice(file_name.as_bytes());

        if mode != FileOperationMode::DeleteFile && mode != FileOperationMode::ReadDir {
            bytes.push(0x00); // No compression or encryption (u8)
        }
        // only add file size if not DeleteFile, ReadDir, or ReadFile
        if mode != FileOperationMode::DeleteFile 
            && mode != FileOperationMode::ReadDir 
            && mode != FileOperationMode::ReadFile {
            // count the number of bytes occupied by the file size
            let num = ((u128::BITS - file_size.leading_zeros() + 15 )/ 8) as u8;
            match num {
                1 => {
                    bytes.push(1);
                    bytes.write_u8(file_size as u8).unwrap();
                    bytes.write_u8(file_size as u8).unwrap();
                }
                2 => {
                    bytes.push(2);
                    bytes.write_u16::<byteorder::BigEndian>(file_size as u16).unwrap();
                    bytes.write_u16::<byteorder::BigEndian>(file_size as u16).unwrap();
                }
                3..=4 => {
                    bytes.push(4);
                    bytes.write_u32::<byteorder::BigEndian>(file_size as u32).unwrap();
                    bytes.write_u32::<byteorder::BigEndian>(file_size as u32).unwrap();
                }
                5..=8 => {
                    bytes.push(8);
                    bytes.write_u64::<byteorder::BigEndian>(file_size as u64).unwrap();
                    bytes.write_u64::<byteorder::BigEndian>(file_size as u64).unwrap();
                }
                _ => {
                    bytes.push(16);
                    bytes.write_u128::<byteorder::BigEndian>(file_size).unwrap();
                    bytes.write_u128::<byteorder::BigEndian>(file_size).unwrap();
                }
            }
        }
        bytes
    }

    #[test]
    fn add_file() {
        let compare_string = "test.txt";
        let file_size: u128 = (u64::MAX as u128) + 1000u128;
        let bytes = get_bytes(FileOperationMode::AddFile, compare_string, file_size);
        let req = RequestFileTransferRequest::option_from_reader(&mut &bytes[..])
            .unwrap()
            .unwrap();
        assert_eq!(req.mode_of_operation, FileOperationMode::AddFile);
        assert_eq!(req.file_path_and_name_length, 8);
        assert_eq!(req.file_path_and_name, compare_string);
        assert_eq!(req.data_format_identifier, 0x00);
        assert_eq!(req.file_size_parameter_length, 16);
        assert_eq!(req.file_size_uncompressed, file_size);
        assert_eq!(req.file_size_compressed, file_size);
    }

    #[test]
    fn delete_file() {
        let compare_string = "/var/tmp/delete_file.bin";
        let bytes = get_bytes(FileOperationMode::DeleteFile, compare_string, 0);
        let req = RequestFileTransferRequest::option_from_reader(&mut &bytes[..])
            .unwrap()
            .unwrap();
        assert_eq!(req.mode_of_operation, FileOperationMode::DeleteFile);
        assert_eq!(req.file_path_and_name_length, compare_string.len() as u16);
        assert_eq!(req.file_path_and_name, compare_string);
        assert_eq!(req.data_format_identifier, 0x00);
        assert_eq!(req.file_size_parameter_length, 0);
    }

    #[test]
    fn write_add_file() {
        let compare_string = "test.txt";
        let file_size: u128 = (u64::MAX as u128) + 1000u128;
        let req = RequestFileTransferRequest {
            mode_of_operation: FileOperationMode::AddFile,
            file_path_and_name_length: compare_string.len() as u16,
            file_path_and_name: compare_string.to_string(),
            data_format_identifier: DataFormatIdentifier::new(0, 0).unwrap(),
            file_size_parameter_length: 16,
            file_size_uncompressed: file_size,
            file_size_compressed: file_size,
        };
        let mut bytes = Vec::new();
        req.to_writer(&mut bytes).unwrap();
        // Should be equivalent to our helper function
        let expected_bytes = get_bytes(FileOperationMode::AddFile, compare_string, file_size);
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn write_delete_file() {
        let compare_string = "/var/tmp/delete_file.bin";
        let req = RequestFileTransferRequest {
            mode_of_operation: FileOperationMode::DeleteFile,
            file_path_and_name_length: compare_string.len() as u16,
            file_path_and_name: compare_string.to_string(),
            data_format_identifier: DataFormatIdentifier::new(0, 0).unwrap(),
            // these shouldn't be used
            file_size_parameter_length: 0,
            file_size_uncompressed: 0,
            file_size_compressed: 0,
        };
        let mut bytes = Vec::new();
        req.to_writer(&mut bytes).unwrap();
        // Should be equivalent to our helper function
        let expected_bytes = get_bytes(FileOperationMode::DeleteFile, compare_string, 0);
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_file_operation_mode() {
        assert_eq!(FileOperationMode::AddFile, FileOperationMode::try_from(0x01).unwrap());
        assert_eq!(FileOperationMode::DeleteFile, FileOperationMode::try_from(0x02).unwrap());
        assert_eq!(FileOperationMode::ReplaceFile, FileOperationMode::try_from(0x03).unwrap());
        assert_eq!(FileOperationMode::ReadFile, FileOperationMode::try_from(0x04).unwrap());
        assert_eq!(FileOperationMode::ReadDir, FileOperationMode::try_from(0x05).unwrap());
        assert_eq!(FileOperationMode::ResumeFile, FileOperationMode::try_from(0x06).unwrap());

        // assert error
        assert!(matches!(FileOperationMode::try_from(0x00), Err(Error::InvalidFileOperationMode(0x00))));
    }
}
