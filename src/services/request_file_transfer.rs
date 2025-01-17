use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{DataFormatIdentifier, Error, SingleValueWireFormat, WireFormat};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum FileOperationMode {
    // 0x00, 0x07-0xFF Reserved for future definition by ISO
    ISOSAEReserved,
    /// Add a file to the server
    AddFile,
    /// Delete the specified file from the server
    DeleteFile,
    /// Replace the specified file on the server, if it does not exist, add it
    ReplaceFile,
    /// Read the specified file from the server (upload)
    ReadFile,
    /// Read the directory from the server
    /// Implies that the request does not include a `fileName`
    ReadDir,
    /// Resume a file transfer at the returned `filePosition` indicator
    /// The file must already exist in the ECU's filesystem
    ResumeFile,
}

impl From<FileOperationMode> for u8 {
    fn from(value: FileOperationMode) -> Self {
        match value {
            FileOperationMode::ISOSAEReserved => 0x00,
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
            _ => Err(Error::InvalidFileOperationMode(value)),
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
    pub mode_of_operation: u8,

    pub file_path_and_name_length: [u8; 2],

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
}

impl WireFormat for RequestFileTransferRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        todo!();
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        todo!();
    }
}

impl SingleValueWireFormat for RequestFileTransferRequest {}

#[cfg(test)]
mod tests {
    use super::*;

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
