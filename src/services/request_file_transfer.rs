use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Read;

use crate::{
    DataFormatIdentifier, Error, LengthFormatIdentifier, SingleValueWireFormat, WireFormat,
};

///////////////////////////////////////// - Request - ///////////////////////////////////////////////////
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

/// Holds the sizes of the file to be transferred (if applicable)
/// Used for both [RequestFileTransferRequest] and [ResponseFileTransferResponse]
///
/// |              | [AddFile] | [DeleteFile] | [ReplaceFile] | [ReadFile] | [ReadDir] | [ResumeFile] |
/// |--------------|-----------|--------------|---------------|------------|-----------|--------------|
/// |**[Request]** | Yes       |              | Yes           |            |           | Yes          |
/// |**[Response]**|           |              |               | Yes        |           |              |
///
/// [AddFile]: FileOperationMode::AddFile
/// [DeleteFile]: FileOperationMode::DeleteFile
/// [ReplaceFile]: FileOperationMode::ReplaceFile
/// [ReadFile]: FileOperationMode::ReadFile
/// [ReadDir]: FileOperationMode::ReadDir
/// [ResumeFile]: FileOperationMode::ResumeFile
/// [Request]: RequestFileTransferRequest (RequestFileTransferRequest)
/// [Response]: ResponseFileTransferResponse (ResponseFileTransferResponse)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SizePayload {
    /// Length in bytes for both `file_size_uncompressed` and `file_size_compressed`
    ///
    /// Not included in the *Request* message if `mode_of_operation` is one of:
    ///  * `DeleteFile` (0x02)
    ///  * `ReadFile` (0x04)
    ///  * `ReadDir` (0x05)
    ///
    /// Not included in the *Response* message if `mode_of_operation` is one of:
    ///    * `DeleteFile` (0x02)
    pub file_size_parameter_length: u8,

    /// Specifies the size of the uncompressed file in bytes.
    ///
    /// Not included in the request message if `mode_of_operation` is one of:
    ///     * `DeleteFile` (0x02)
    ///     * `ReadFile` (0x04)
    ///     * `ReadDir` (0x05)
    pub file_size_uncompressed: u128,

    /// Specifies the size of the compressed file in bytes
    ///
    /// Not included in the request message if `mode_of_operation` is one of:
    ///     * `DeleteFile` (0x02)
    ///     * `ReadFile` (0x04)
    ///     * `ReadDir` (0x05)
    pub file_size_compressed: u128,
}

impl WireFormat for SizePayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let file_size_parameter_length = reader.read_u8()?;
        let mut file_size_uncompressed = vec![0; file_size_parameter_length as usize];
        let mut file_size_compressed = vec![0; file_size_parameter_length as usize];

        reader.read_exact(&mut file_size_uncompressed)?;
        reader.read_exact(&mut file_size_compressed)?;

        Ok(Some(Self {
            file_size_parameter_length,
            file_size_uncompressed: u128::from_be_bytes({
                let mut bytes = [0; 16];
                bytes[16 - file_size_parameter_length as usize..]
                    .copy_from_slice(&file_size_uncompressed);
                bytes
            }),
            file_size_compressed: u128::from_be_bytes({
                let mut bytes = [0; 16];
                bytes[16 - file_size_parameter_length as usize..]
                    .copy_from_slice(&file_size_compressed);
                bytes
            }),
        }))
    }
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let mut len: usize = 0;

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
            _ => {
                return Err(Error::InvalidFileSizeParameterLength(
                    self.file_size_parameter_length,
                ))
            }
        };

        Ok(len)
    }
}
impl SingleValueWireFormat for SizePayload {}

/// Payload used for all [RequestFileTransfer requests][RequestFileTransferRequest]
///
/// #### ***Request*** Message
/// |               | [AddFile] | [DeleteFile] | [ReplaceFile] | [ReadFile] | [ReadDir] | [ResumeFile] |
/// |---------------|-----------|--------------|---------------|------------|-----------|--------------|
/// |**[Request]**  | Yes       | Yes          | Yes           | Yes        | Yes       | Yes          |
///
/// [AddFile]: FileOperationMode::AddFile
/// [DeleteFile]: FileOperationMode::DeleteFile
/// [ReplaceFile]: FileOperationMode::ReplaceFile
/// [ReadFile]: FileOperationMode::ReadFile
/// [ReadDir]: FileOperationMode::ReadDir
/// [ResumeFile]: FileOperationMode::ResumeFile
/// [Request]: RequestFileTransferRequest (RequestFileTransferRequest)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NamePayload {
    /// 0x01 - 0x06, the type of operation to be applied to the file or directory specified in `file_path_and_name`
    ///
    /// Duplicated as we need to read and store it somewhere
    mode_of_operation: FileOperationMode,

    /// Length in bytes of the `file_path_and_name` field
    file_path_and_name_length: u16,

    /// The path and name of the file or directory on the server
    file_path_and_name: String,
}

impl WireFormat for NamePayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mode_of_operation = FileOperationMode::try_from(reader.read_u8()?)?;
        let file_path_and_name_length = reader.read_u16::<byteorder::BigEndian>()?;

        // Read # of bytes specified by `file_path_and_name_length`
        let mut file_path_and_name = String::new();
        reader
            .take(file_path_and_name_length as u64)
            .read_to_string(&mut file_path_and_name)?;

        Ok(Some(Self {
            mode_of_operation,
            file_path_and_name_length,
            file_path_and_name,
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let mut len = 0;
        // Write the mode of operation
        writer.write_u8((self.mode_of_operation).into())?;
        len += 1;
        // Write the file path and name length
        writer.write_u16::<byteorder::BigEndian>(self.file_path_and_name_length)?;
        len += 2;
        // Write the file path and name
        writer.write_all(self.file_path_and_name.as_bytes())?;
        len += self.file_path_and_name.len();
        Ok(len)
    }
}
impl SingleValueWireFormat for NamePayload {}
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
pub enum RequestFileTransferRequest {
    /// Add a file to the server
    AddFile(NamePayload, DataFormatIdentifier, SizePayload),

    /// Delete the specified file from the server
    DeleteFile(NamePayload),

    /// Replace the specified file on the server, if it does not exist, add it
    ReplaceFile(NamePayload, DataFormatIdentifier, SizePayload),

    /// Read the specified file from the server (upload)
    ReadFile(NamePayload, DataFormatIdentifier),

    /// Read the directory from the server
    /// Implies that the request does not include a `fileName`
    ReadDir(NamePayload),

    /// Resume a file transfer at the returned `filePosition` indicator
    /// The file must already exist in the ECU's filesystem
    ResumeFile(NamePayload, DataFormatIdentifier, SizePayload),
}

impl SingleValueWireFormat for RequestFileTransferRequest {}

impl WireFormat for RequestFileTransferRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let name_payload = NamePayload::from_reader(reader)?;

        // read the filename
        Ok(Some(match name_payload.mode_of_operation {
            // Complicated
            FileOperationMode::AddFile => Self::AddFile(
                name_payload,
                DataFormatIdentifier::from_reader(reader)?,
                SizePayload::from_reader(reader)?,
            ),
            FileOperationMode::ReplaceFile => Self::ReplaceFile(
                name_payload,
                DataFormatIdentifier::from_reader(reader)?,
                SizePayload::from_reader(reader)?,
            ),
            FileOperationMode::ResumeFile => Self::ResumeFile(
                name_payload,
                DataFormatIdentifier::from_reader(reader)?,
                SizePayload::from_reader(reader)?,
            ),
            FileOperationMode::ReadFile => {
                Self::ReadFile(name_payload, DataFormatIdentifier::from_reader(reader)?)
            }
            FileOperationMode::ReadDir => Self::ReadDir(name_payload),
            FileOperationMode::DeleteFile => Self::DeleteFile(name_payload),
            FileOperationMode::ISOSAEReserved(_) => {
                return Err(Error::InvalidFileOperationMode(
                    name_payload.mode_of_operation.into(),
                ))
            }
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let mut len = 0;
        Ok(match self {
            Self::AddFile(name_payload, data_format_identifier, file_size_payload)
            | Self::ReplaceFile(name_payload, data_format_identifier, file_size_payload)
            | Self::ResumeFile(name_payload, data_format_identifier, file_size_payload) => {
                len += name_payload.to_writer(writer)?;
                len += data_format_identifier.to_writer(writer)?;
                len += file_size_payload.to_writer(writer)?;
                len
            }
            Self::ReadFile(name_payload, data_format_identifier) => {
                len += name_payload.to_writer(writer)?;
                len += data_format_identifier.to_writer(writer)?;
                len
            }
            Self::DeleteFile(name_payload) | Self::ReadDir(name_payload) => {
                len += name_payload.to_writer(writer)?;
                len
            }
        })
    }
}

///////////////////////////////////////// - Response - ///////////////////////////////////////////////////

/// Sent by the server to inform the client of the maximum number of bytes to include in each TransferData request message
///
/// |               | [AddFile] | [DeleteFile] | [ReplaceFile] | [ReadFile] | [ReadDir] | [ResumeFile] |
/// |---------------|-----------|--------------|---------------|------------|-----------|--------------|
/// |**[Response]** | Yes       |              | Yes           | Yes        | Yes       | Yes          |
///
/// [AddFile]: FileOperationMode::AddFile
/// [DeleteFile]: FileOperationMode::DeleteFile
/// [ReplaceFile]: FileOperationMode::ReplaceFile
/// [ReadFile]: FileOperationMode::ReadFile
/// [ReadDir]: FileOperationMode::ReadDir
/// [ResumeFile]: FileOperationMode::ResumeFile
/// [Response]: RequestFileTransferRequest (RequestFileTransferResponse)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SentDataPayload {
    length_format_identifier: LengthFormatIdentifier,
    /// This parameter is used by the requestFileTransfer positive response message to inform the client how many
    /// data bytes (maxNumberOfBlockLength) to include in each TransferData request message from the client or how
    /// many data bytes the server will include in a TransferData positive response when uploading data. This length
    /// reflects the complete message length, including the service identifier and the data parameters present in the
    /// TransferData request message or positive response message. This parameter allows either the client to adapt to
    /// the receive buffer size of the server before it starts transferring data to the server or to indicate how many data
    /// bytes will be included in each TransferData positive response in the event that data is uploaded. A server is
    /// required to accept transferData requests that are equal in length to its reported maxNumberOfBlockLength. It is
    /// server specific what transferData request lengths less than maxNumberOfBlockLength are accepted (if any).
    ///
    /// NOTE The last transferData request within a given block can be required to be less than
    /// maxNumberOfBlockLength. It is not allowed for a server to write additional data bytes (i.e. pad bytes) not
    /// contained within the transferData message (either in a compressed or uncompressed format), as this would
    /// affect the memory address of where the subsequent transferData request data would be written.
    /// If the modeOfOperation parameter equals to 0x02 (DeleteFile) this parameter shall be not be included in the
    /// response message.
    pub max_number_of_block_length: Vec<u8>,
}

impl SingleValueWireFormat for SentDataPayload {}
impl WireFormat for SentDataPayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let length_format_identifier = LengthFormatIdentifier::from(reader.read_u8()?);

        let mut max_number_of_block_length: Vec<u8> =
            vec![0; length_format_identifier.max_number_of_block_length as usize];
        reader.read_exact(&mut max_number_of_block_length)?;
        Ok(Some(Self {
            length_format_identifier,
            max_number_of_block_length,
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.length_format_identifier.into())?;
        writer.write_all(&self.max_number_of_block_length)?;
        Ok(1 + self.max_number_of_block_length.len())
    }
}

/// Used to inform the client of the size of the directory to be transferred
/// This data functionally overlaps in the ISO 14229-1:2020 standard with the
/// SizePayload fields for file size (fileSizeOrDirInfoParameterLength, fileSizeUncompressedOrDirInfoLength, fileSizeCompressed)
///
/// |               | [AddFile] | [DeleteFile] | [ReplaceFile] | [ReadFile] | [ReadDir] | [ResumeFile] |
/// |---------------|-----------|--------------|---------------|------------|-----------|--------------|
/// |**[Response]** |           |              |               |            | Yes       |              |
///
/// [AddFile]: FileOperationMode::AddFile
/// [DeleteFile]: FileOperationMode::DeleteFile
/// [ReplaceFile]: FileOperationMode::ReplaceFile
/// [ReadFile]: FileOperationMode::ReadFile
/// [ReadDir]: FileOperationMode::ReadDir
/// [ResumeFile]: FileOperationMode::ResumeFile
/// [Response]: RequestFileTransferRequest (RequestFileTransferResponse)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DirSizePayload {
    pub dir_info_parameter_length: u16,
    pub dir_info_length: u128,
}

impl SingleValueWireFormat for DirSizePayload {}
impl WireFormat for DirSizePayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let dir_info_parameter_length = reader.read_u16::<byteorder::BigEndian>()?;
        let mut dir_info_length = vec![0; dir_info_parameter_length as usize];
        reader.read_exact(&mut dir_info_length)?;

        Ok(Some(Self {
            dir_info_parameter_length,
            dir_info_length: u128::from_be_bytes({
                let mut bytes = [0; 16];
                bytes[16 - dir_info_parameter_length as usize..].copy_from_slice(&dir_info_length);
                bytes
            }),
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        let mut len = 0;
        writer.write_u16::<byteorder::BigEndian>(self.dir_info_parameter_length)?;
        len += 2;
        // write the file size only as many bytes as needed
        // Slice off only the number of bytes we need from the end of the file_size bytes
        let dir_info_length = self.dir_info_length.to_be_bytes();
        let mut bytes: Vec<u8> = Vec::new();

        bytes.extend_from_slice(&dir_info_length[16 - self.dir_info_parameter_length as usize..]);
        writer.write_all(&bytes)?;

        len += bytes.len();

        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // helper function to get some bytes to read from
    fn get_bytes(mode: FileOperationMode, file_name: &str, file_size: u128) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(mode.into()); // AddFile (u8)
                                 // write file_name len as 2 bytes
        bytes
            .write_u16::<byteorder::BigEndian>(file_name.len() as u16)
            .unwrap();
        bytes.extend_from_slice(file_name.as_bytes());

        if mode != FileOperationMode::DeleteFile && mode != FileOperationMode::ReadDir {
            bytes.push(0x00); // No compression or encryption (u8)
        }
        // only add file size if not DeleteFile, ReadDir, or ReadFile
        if mode != FileOperationMode::DeleteFile
            && mode != FileOperationMode::ReadDir
            && mode != FileOperationMode::ReadFile
        {
            // count the number of bytes occupied by the file size
            let num = ((u128::BITS - file_size.leading_zeros() + 15) / 8) as u8;
            match num {
                1 => {
                    bytes.push(1);
                    bytes.write_u8(file_size as u8).unwrap();
                    bytes.write_u8(file_size as u8).unwrap();
                }
                2 => {
                    bytes.push(2);
                    bytes
                        .write_u16::<byteorder::BigEndian>(file_size as u16)
                        .unwrap();
                    bytes
                        .write_u16::<byteorder::BigEndian>(file_size as u16)
                        .unwrap();
                }
                3..=4 => {
                    bytes.push(4);
                    bytes
                        .write_u32::<byteorder::BigEndian>(file_size as u32)
                        .unwrap();
                    bytes
                        .write_u32::<byteorder::BigEndian>(file_size as u32)
                        .unwrap();
                }
                5..=8 => {
                    bytes.push(8);
                    bytes
                        .write_u64::<byteorder::BigEndian>(file_size as u64)
                        .unwrap();
                    bytes
                        .write_u64::<byteorder::BigEndian>(file_size as u64)
                        .unwrap();
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
        let req: crate::RequestFileTransferRequest =
            RequestFileTransferRequest::option_from_reader(&mut &bytes[..])
                .unwrap()
                .unwrap();

        match req {
            RequestFileTransferRequest::AddFile(pl, data_format_pl, file_size_pl) => {
                assert_eq!(pl.mode_of_operation, FileOperationMode::AddFile);
                assert_eq!(pl.file_path_and_name_length, compare_string.len() as u16);
                assert_eq!(pl.file_path_and_name, compare_string);
                assert_eq!(data_format_pl, DataFormatIdentifier::new(0, 0).unwrap());
                assert_eq!(file_size_pl.file_size_parameter_length, 16);
                assert_eq!(file_size_pl.file_size_uncompressed, file_size);
                assert_eq!(file_size_pl.file_size_compressed, file_size);
            }
            _ => panic!("Expected AddFile"),
        }
    }

    #[test]
    fn delete_file() {
        let compare_string = "/var/tmp/delete_file.bin";
        let bytes = get_bytes(FileOperationMode::DeleteFile, compare_string, 0);
        let req = RequestFileTransferRequest::option_from_reader(&mut &bytes[..])
            .unwrap()
            .unwrap();
        match req {
            RequestFileTransferRequest::DeleteFile(pl) => {
                assert_eq!(pl.mode_of_operation, FileOperationMode::DeleteFile);
                assert_eq!(pl.file_path_and_name_length, compare_string.len() as u16);
                assert_eq!(pl.file_path_and_name, compare_string);
            }
            _ => panic!("Expected DeleteFile"),
        }
    }

    #[test]
    fn write_add_file() {
        let compare_string = "test.txt";
        let file_size: u128 = 0x1234;
        let bytes = get_bytes(FileOperationMode::AddFile, compare_string, file_size);
        let req = RequestFileTransferRequest::option_from_reader(&mut &bytes[..])
            .unwrap()
            .unwrap();
        let mut bytes = Vec::new();
        req.to_writer(&mut bytes).unwrap();
        // Should be equivalent to our helper function
        let expected_bytes = get_bytes(FileOperationMode::AddFile, compare_string, file_size);
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn write_delete_file() {
        let compare_string = "/var/tmp/delete_file.bin";
        let req = RequestFileTransferRequest::DeleteFile(NamePayload {
            mode_of_operation: FileOperationMode::DeleteFile,
            file_path_and_name_length: compare_string.len() as u16,
            file_path_and_name: compare_string.to_string(),
        });
        let mut bytes = Vec::new();
        req.to_writer(&mut bytes).unwrap();
        // Should be equivalent to our helper function
        let expected_bytes = get_bytes(FileOperationMode::DeleteFile, compare_string, 0);
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_file_operation_mode() {
        use FileOperationMode::*;
        assert_eq!(AddFile, FileOperationMode::try_from(0x01).unwrap());
        assert_eq!(DeleteFile, FileOperationMode::try_from(0x02).unwrap());
        assert_eq!(ReplaceFile, FileOperationMode::try_from(0x03).unwrap());
        assert_eq!(ReadFile, FileOperationMode::try_from(0x04).unwrap());
        assert_eq!(ReadDir, FileOperationMode::try_from(0x05).unwrap());
        assert_eq!(ResumeFile, FileOperationMode::try_from(0x06).unwrap());
        assert_eq!(
            ISOSAEReserved(0x07),
            FileOperationMode::try_from(0x07).unwrap()
        );
    }
}
