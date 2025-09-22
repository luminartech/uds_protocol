use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Read;
use utoipa::ToSchema;

use crate::{DataFormatIdentifier, Error, SingleValueWireFormat, WireFormat};

///////////////////////////////////////// - Request - ///////////////////////////////////////////////////
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
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
/// Used for both [`RequestFileTransferRequest`] and [`RequestFileTransferResponse`]
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
/// [Response]: RequestFileTransferResponse (RequestFileTransferResponse)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[allow(clippy::struct_field_names)]
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

    fn required_size(&self) -> usize {
        1 + (2 * self.file_size_parameter_length as usize)
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Always write the file size as 1 byte
        writer.write_u8(self.file_size_parameter_length)?;
        // write the file size only as many bytes as needed
        // Slice off only the number of bytes we need from the end of the file_size bytes
        let uncompressed = self.file_size_uncompressed.to_be_bytes();
        let compressed = self.file_size_compressed.to_be_bytes();
        // file_size_uncompressed
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&uncompressed[16 - self.file_size_parameter_length as usize..]);
        // file_size_compressed
        bytes.extend_from_slice(&compressed[16 - self.file_size_parameter_length as usize..]);

        writer.write_all(&bytes)?;

        Ok(self.required_size())
    }
}
impl SingleValueWireFormat for SizePayload {}

/// Payload used for all [`RequestFileTransfer` requests][RequestFileTransferRequest]
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
            .take(u64::from(file_path_and_name_length))
            .read_to_string(&mut file_path_and_name)?;

        Ok(Some(Self {
            mode_of_operation,
            file_path_and_name_length,
            file_path_and_name,
        }))
    }

    fn required_size(&self) -> usize {
        1 + 2 + self.file_path_and_name.len()
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Write the mode of operation
        writer.write_u8((self.mode_of_operation).into())?;
        // Write the file path and name length
        writer.write_u16::<byteorder::BigEndian>(self.file_path_and_name_length)?;
        // Write the file path and name
        writer.write_all(self.file_path_and_name.as_bytes())?;
        Ok(self.required_size())
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
/// there is no need to use the `TransferData` or [`crate::UdsServiceType::RequestTransferExit`] services.
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
                ));
            }
        }))
    }

    fn required_size(&self) -> usize {
        match self {
            Self::AddFile(name_payload, data_format_identifier, file_size_payload)
            | Self::ReplaceFile(name_payload, data_format_identifier, file_size_payload)
            | Self::ResumeFile(name_payload, data_format_identifier, file_size_payload) => {
                name_payload.required_size()
                    + data_format_identifier.required_size()
                    + file_size_payload.required_size()
            }
            Self::ReadFile(name_payload, data_format_identifier) => {
                name_payload.required_size() + data_format_identifier.required_size()
            }
            Self::DeleteFile(name_payload) | Self::ReadDir(name_payload) => {
                name_payload.required_size()
            }
        }
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

/// Sent by the server to inform the client of the maximum number of bytes to include in each `TransferData` request message
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
    /// Not related to `RequestDownload`
    length_format_identifier: u8,
    /// This parameter is used by the requestFileTransfer positive response message to inform the client how many
    /// data bytes (maxNumberOfBlockLength) to include in each `TransferData` request message from the client or how
    /// many data bytes the server will include in a `TransferData` positive response when uploading data. This length
    /// reflects the complete message length, including the service identifier and the data parameters present in the
    /// `TransferData` request message or positive response message. This parameter allows either the client to adapt to
    /// the receive buffer size of the server before it starts transferring data to the server or to indicate how many data
    /// bytes will be included in each `TransferData` positive response in the event that data is uploaded. A server is
    /// required to accept transferData requests that are equal in length to its reported maxNumberOfBlockLength. It is
    /// server specific what transferData request lengths less than maxNumberOfBlockLength are accepted (if any).
    ///
    /// NOTE The last transferData request within a given block can be required to be less than
    /// maxNumberOfBlockLength. It is not allowed for a server to write additional data bytes (i.e. pad bytes) not
    /// contained within the transferData message (either in a compressed or uncompressed format), as this would
    /// affect the memory address of where the subsequent transferData request data would be written.
    /// If the modeOfOperation parameter equals to 0x02 (`DeleteFile`) this parameter shall be not be included in the
    /// response message.
    pub max_number_of_block_length: Vec<u8>,
}

impl SingleValueWireFormat for SentDataPayload {}
impl WireFormat for SentDataPayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let length_format_identifier = reader.read_u8()?;

        let mut max_number_of_block_length: Vec<u8> = vec![0; length_format_identifier as usize];
        reader.read_exact(&mut max_number_of_block_length)?;
        Ok(Some(Self {
            length_format_identifier,
            max_number_of_block_length,
        }))
    }

    fn required_size(&self) -> usize {
        1 + self.max_number_of_block_length.len()
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.length_format_identifier)?;
        writer.write_all(&self.max_number_of_block_length)?;
        Ok(self.required_size())
    }
}

/// Used to inform the client of the size of the file to be transferred
///
/// |               | [AddFile] | [DeleteFile] | [ReplaceFile] | [ReadFile] | [ReadDir] | [ResumeFile] |
/// |---------------|-----------|--------------|---------------|------------|-----------|--------------|
/// |**[Response]** |           |              |               | Yes        |           |              |
///
/// [AddFile]: FileOperationMode::AddFile
/// [DeleteFile]: FileOperationMode::DeleteFile
/// [ReplaceFile]: FileOperationMode::ReplaceFile
/// [ReadFile]: FileOperationMode::ReadFile
/// [ReadDir]: FileOperationMode::ReadDir
/// [ResumeFile]: FileOperationMode::ResumeFile
/// [Response]: RequestFileTransferRequest (RequestFileTransferResponse)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[allow(clippy::struct_field_names)]
pub struct FileSizePayload {
    pub file_size_parameter_length: u16,
    pub file_size_uncompressed: u128,
    pub file_size_compressed: u128,
}

impl WireFormat for FileSizePayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let file_size_parameter_length = reader.read_u16::<byteorder::BE>()?;
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

    fn required_size(&self) -> usize {
        2 + (2 * self.file_size_parameter_length as usize)
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Always write the file size as 1 byte

        writer.write_u16::<byteorder::BE>(self.file_size_parameter_length)?;
        // write the file size only as many bytes as needed
        // Slice off only the number of bytes we need from the end of the file_size bytes
        let uncompressed = self.file_size_uncompressed.to_be_bytes();
        let compressed = self.file_size_compressed.to_be_bytes();
        // file_size_uncompressed
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend_from_slice(&uncompressed[16 - self.file_size_parameter_length as usize..]);
        // file_size_compressed
        bytes.extend_from_slice(&compressed[16 - self.file_size_parameter_length as usize..]);

        writer.write_all(&bytes)?;

        Ok(self.required_size())
    }
}
impl SingleValueWireFormat for FileSizePayload {}

/// Used to inform the client of the size of the directory to be transferred
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

    fn required_size(&self) -> usize {
        2 + self.dir_info_parameter_length as usize
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

/// Used to inform the client of the byte position within the file at which the Tester will resume downloading after an initial download is suspended
///
/// |               | [AddFile] | [DeleteFile] | [ReplaceFile] | [ReadFile] | [ReadDir] | [ResumeFile] |
/// |---------------|-----------|--------------|---------------|------------|-----------|--------------|
/// |**[Response]** |           |              |               |            |           | Yes          |
///
/// [AddFile]: FileOperationMode::AddFile
/// [DeleteFile]: FileOperationMode::DeleteFile
/// [ReplaceFile]: FileOperationMode::ReplaceFile
/// [ReadFile]: FileOperationMode::ReadFile
/// [ReadDir]: FileOperationMode::ReadDir
/// [ResumeFile]: FileOperationMode::ResumeFile
/// [Response]: RequestFileTransferRequest (RequestFileTransferResponse)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PositionPayload {
    /// Specifies the byte position within the file at which the Tester will resume downloading after an initial download is suspended
    /// A download is suspended when the ECU stops receiving [`crate::TransferDataRequest`] requests and does not receive the
    /// [`crate::RequestTransferExitRequest`] request to end the transfer before returning to the default session
    ///
    /// Fixed size: 8 bytes
    ///
    /// Not included for [`AddFile`][FileOperationMode::AddFile], [`DeleteFile`][FileOperationMode::DeleteFile], [`ReplaceFile`][FileOperationMode::ReplaceFile], [`ReadFile`][FileOperationMode::ReadFile], or [`ReadDir`][FileOperationMode::ReadDir]
    /// Only present if `mode_of_operation` is [`ResumeFile`][FileOperationMode::ResumeFile] (for ISO 14229-1:2020)
    pub file_position: u64,
}

impl SingleValueWireFormat for PositionPayload {}
impl WireFormat for PositionPayload {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        Ok(Some(Self {
            file_position: reader.read_u64::<byteorder::BigEndian>()?,
        }))
    }
    // For PositionPayload
    fn required_size(&self) -> usize {
        8
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u64::<byteorder::BigEndian>(self.file_position)?;
        Ok(8)
    }
}

/// Response to a [`RequestFileTransferRequest`] from the server
///
/// The server will respond with a [`RequestFileTransferResponse`] to indicate the status of the request
/// [`DataFormatIdentifier`] - Echoes the value of the request
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, ToSchema)]
#[non_exhaustive]
pub enum RequestFileTransferResponse {
    AddFile(FileOperationMode, SentDataPayload, DataFormatIdentifier),
    DeleteFile(FileOperationMode),
    ReplaceFile(FileOperationMode, SentDataPayload, DataFormatIdentifier),
    ReadFile(
        FileOperationMode,
        SentDataPayload,
        DataFormatIdentifier,
        FileSizePayload,
    ),
    ReadDir(
        FileOperationMode,
        SentDataPayload,
        DataFormatIdentifier,
        DirSizePayload,
    ),
    ResumeFile(
        FileOperationMode,
        SentDataPayload,
        DataFormatIdentifier,
        PositionPayload,
    ),
}

impl SingleValueWireFormat for RequestFileTransferResponse {}
impl WireFormat for RequestFileTransferResponse {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        // Read the mode of operation
        let mode_of_operation = FileOperationMode::try_from(reader.read_u8()?)?;
        Ok(Some(match mode_of_operation {
            FileOperationMode::AddFile => Self::AddFile(
                mode_of_operation,
                SentDataPayload::from_reader(reader)?,
                DataFormatIdentifier::from_reader(reader)?,
            ),
            FileOperationMode::DeleteFile => Self::DeleteFile(mode_of_operation),
            FileOperationMode::ReplaceFile => Self::ReplaceFile(
                mode_of_operation,
                SentDataPayload::from_reader(reader)?,
                DataFormatIdentifier::from_reader(reader)?,
            ),
            FileOperationMode::ReadFile => Self::ReadFile(
                mode_of_operation,
                SentDataPayload::from_reader(reader)?,
                DataFormatIdentifier::from_reader(reader)?,
                FileSizePayload::from_reader(reader)?,
            ),
            FileOperationMode::ReadDir => Self::ReadDir(
                mode_of_operation,
                SentDataPayload::from_reader(reader)?,
                DataFormatIdentifier::from_reader(reader)?,
                DirSizePayload::from_reader(reader)?,
            ),
            FileOperationMode::ResumeFile => Self::ResumeFile(
                mode_of_operation,
                SentDataPayload::from_reader(reader)?,
                DataFormatIdentifier::from_reader(reader)?,
                PositionPayload::from_reader(reader)?,
            ),
            FileOperationMode::ISOSAEReserved(_) => {
                return Err(Error::InvalidFileOperationMode(mode_of_operation.into()));
            }
        }))
    }

    // For RequestFileTransferResponse
    fn required_size(&self) -> usize {
        match self {
            Self::AddFile(_, sent_data, data_format)
            | Self::ReplaceFile(_, sent_data, data_format) => {
                1 + sent_data.required_size() + data_format.required_size()
            }
            Self::DeleteFile(_) => 1,
            Self::ReadFile(_, sent_data, data_format, file_size) => {
                1 + sent_data.required_size()
                    + data_format.required_size()
                    + file_size.required_size()
            }
            Self::ReadDir(_, sent_data, data_format, dir_size) => {
                1 + sent_data.required_size()
                    + data_format.required_size()
                    + dir_size.required_size()
            }
            Self::ResumeFile(_, sent_data, data_format, position) => {
                1 + sent_data.required_size()
                    + data_format.required_size()
                    + position.required_size()
            }
        }
    }
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Every variant has a mode of operation
        let mut len = 1;

        match self {
            Self::AddFile(mode_of_operation, sent_data_payload, data_format_identifier)
            | Self::ReplaceFile(mode_of_operation, sent_data_payload, data_format_identifier) => {
                writer.write_u8((*mode_of_operation).into())?;
                len += sent_data_payload.to_writer(writer)?;
                len += data_format_identifier.to_writer(writer)?;
            }
            Self::DeleteFile(mode_of_operation) => {
                writer.write_u8((*mode_of_operation).into())?;
            }
            Self::ReadFile(
                mode_of_operation,
                sent_data_payload,
                data_format_identifier,
                size_payload,
            ) => {
                writer.write_u8((*mode_of_operation).into())?;
                len += sent_data_payload.to_writer(writer)?;
                len += data_format_identifier.to_writer(writer)?;
                len += size_payload.to_writer(writer)?;
            }
            Self::ReadDir(
                mode_of_operation,
                sent_data_payload,
                data_format_identifier,
                dir_size_payload,
            ) => {
                writer.write_u8((*mode_of_operation).into())?;
                len += sent_data_payload.to_writer(writer)?;
                len += data_format_identifier.to_writer(writer)?;
                len += dir_size_payload.to_writer(writer)?;
            }
            Self::ResumeFile(
                mode_of_operation,
                sent_data_payload,
                data_format_identifier,
                position_payload,
            ) => {
                writer.write_u8((*mode_of_operation).into())?;
                len += sent_data_payload.to_writer(writer)?;
                len += data_format_identifier.to_writer(writer)?;
                len += position_payload.to_writer(writer)?;
            }
        }
        Ok(len)
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;
    use crate::param_length_u128;

    // helper function to get some bytes to read from
    #[allow(clippy::cast_possible_truncation)]
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
            let num = param_length_u128(file_size) as u8;
            // use write exact
            bytes.write_u8(num).unwrap();
            // write the file size only as many bytes as needed
            // Slice off only the number of bytes we need from the end of the file_size bytes
            let source = file_size.to_be_bytes();
            // file_size_uncompressed
            bytes.extend_from_slice(&source[16 - num as usize..]);
            // file_size_compressed
            bytes.extend_from_slice(&source[16 - num as usize..]);
        }
        bytes
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_lossless)]
    fn add_file() {
        let compare_string = "test.txt";
        let file_size: u128 = (u64::MAX as u128) + 1000u128;
        let bytes = get_bytes(FileOperationMode::AddFile, compare_string, file_size);
        let req: crate::RequestFileTransferRequest =
            RequestFileTransferRequest::option_from_reader(&mut bytes.as_slice())
                .unwrap()
                .unwrap();

        let mut written_bytes = Vec::new();
        let written = req.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, written_bytes.len());
        assert_eq!(written, req.required_size());

        match req {
            RequestFileTransferRequest::AddFile(pl, data_format_pl, file_size_pl) => {
                assert_eq!(pl.mode_of_operation, FileOperationMode::AddFile);
                assert_eq!(pl.file_path_and_name_length, compare_string.len() as u16);
                assert_eq!(pl.file_path_and_name, compare_string);
                assert_eq!(data_format_pl, DataFormatIdentifier::new(0, 0).unwrap());
                assert_eq!(file_size_pl.file_size_parameter_length, 9);
                assert_eq!(file_size_pl.file_size_uncompressed, file_size);
                assert_eq!(file_size_pl.file_size_compressed, file_size);
            }
            _ => panic!("Expected AddFile"),
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn delete_file() {
        let compare_string = "/var/tmp/delete_file.bin";
        let bytes = get_bytes(FileOperationMode::DeleteFile, compare_string, 0);
        let req = RequestFileTransferRequest::option_from_reader(&mut bytes.as_slice())
            .unwrap()
            .unwrap();

        let mut written_bytes = Vec::new();
        let written = req.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, written_bytes.len());
        assert_eq!(written, req.required_size());

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
    #[allow(clippy::cast_possible_truncation)]
    fn write_add_file() {
        let compare_string = "test.txt";
        let file_size: u128 = 0x1234;
        let bytes = get_bytes(FileOperationMode::AddFile, compare_string, file_size);
        let req = RequestFileTransferRequest::option_from_reader(&mut bytes.as_slice())
            .unwrap()
            .unwrap();

        let mut bytes = Vec::new();
        let written = req.to_writer(&mut bytes).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(written, req.required_size());

        // Should be equivalent to our helper function
        let expected_bytes = get_bytes(FileOperationMode::AddFile, compare_string, file_size);
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn write_delete_file() {
        let compare_string = "/var/tmp/delete_file.bin";
        let req = RequestFileTransferRequest::DeleteFile(NamePayload {
            mode_of_operation: FileOperationMode::DeleteFile,
            file_path_and_name_length: compare_string.len() as u16,
            file_path_and_name: compare_string.to_string(),
        });
        let mut bytes = Vec::new();
        let written = req.to_writer(&mut bytes).unwrap();
        // Should be equivalent to our helper function
        let expected_bytes = get_bytes(FileOperationMode::DeleteFile, compare_string, 0);
        assert_eq!(bytes, expected_bytes);
        assert_eq!(bytes.len(), written);
        assert_eq!(req.required_size(), written);
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn replace_file() {
        let compare_string = "/opt/testing/replace_file.bin";
        let file_size: u128 = 0x1234;
        let bytes = get_bytes(FileOperationMode::ReplaceFile, compare_string, file_size);
        let req = RequestFileTransferRequest::from_reader(&mut bytes.as_slice()).unwrap();

        let mut written_bytes = Vec::new();
        let written = req.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, written_bytes.len());
        assert_eq!(written, req.required_size());

        match req {
            RequestFileTransferRequest::ReplaceFile(pl, data_format_pl, file_size_pl) => {
                assert_eq!(pl.mode_of_operation, FileOperationMode::ReplaceFile);
                assert_eq!(pl.file_path_and_name_length, compare_string.len() as u16);
                assert_eq!(pl.file_path_and_name, compare_string);
                assert_eq!(data_format_pl, DataFormatIdentifier::new(0, 0).unwrap());
                assert_eq!(file_size_pl.file_size_parameter_length, 2);
                assert_eq!(file_size_pl.file_size_uncompressed, file_size);
                assert_eq!(file_size_pl.file_size_compressed, file_size);
            }
            _ => panic!("Expected ReplaceFile"),
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn read_file() {
        let compare_string = "/opt/testing/just_reading_stuff.txt";
        let file_size: u128 = 0x0;
        let bytes = get_bytes(FileOperationMode::ReadFile, compare_string, file_size);
        let req = RequestFileTransferRequest::from_reader(&mut bytes.as_slice()).unwrap();

        let mut written_bytes = Vec::new();
        let written = req.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, written_bytes.len());
        assert_eq!(written, req.required_size());

        match req {
            RequestFileTransferRequest::ReadFile(pl, data_format_pl) => {
                assert_eq!(pl.mode_of_operation, FileOperationMode::ReadFile);
                assert_eq!(pl.file_path_and_name_length, compare_string.len() as u16);
                assert_eq!(pl.file_path_and_name, compare_string);
                assert_eq!(data_format_pl, DataFormatIdentifier::new(0, 0).unwrap());
            }
            _ => panic!("Expected ReadFile"),
        }
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn resume_file() {
        let compare_string = "/var/tmp/resume_file.bin";
        let file_size = 0x1234;
        let bytes = get_bytes(FileOperationMode::ResumeFile, compare_string, file_size);
        let req = RequestFileTransferRequest::from_reader(&mut bytes.as_slice()).unwrap();
        let mut written_bytes = Vec::new();
        let written = req.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, written_bytes.len());
        assert_eq!(written, req.required_size());

        match req {
            RequestFileTransferRequest::ResumeFile(pl, data_format_pl, file_size_pl) => {
                assert_eq!(pl.mode_of_operation, FileOperationMode::ResumeFile);
                assert_eq!(pl.file_path_and_name_length, compare_string.len() as u16);
                assert_eq!(pl.file_path_and_name, compare_string);
                assert_eq!(data_format_pl, DataFormatIdentifier::new(0, 0).unwrap());
                assert_eq!(file_size_pl.file_size_parameter_length, 2);
                assert_eq!(file_size_pl.file_size_uncompressed, file_size);
                assert_eq!(file_size_pl.file_size_compressed, file_size);
            }
            _ => panic!("Expected ResumeFile"),
        }
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

#[cfg(test)]
mod response_tests {

    use crate::{param_length_u32, param_length_u128};

    use super::*;

    fn get_bytes(
        mode: FileOperationMode,
        max_block_len: u32,
        data_format: u8,
        file_size: u128,
        file_position: u64,
    ) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(mode.into());

        // SentDataPayload
        if mode != FileOperationMode::DeleteFile {
            let len_max_block_len = param_length_u32(max_block_len);
            bytes.write_u8(len_max_block_len).unwrap();
            let source = max_block_len.to_be_bytes();
            bytes.extend_from_slice(&source[4 - len_max_block_len as usize..]);

            let mut data_format = data_format;
            if mode == FileOperationMode::ReadDir {
                data_format = 0x00;
            }
            // DataFormatIdentifier
            bytes.write_u8(data_format).unwrap();
        }

        // File or dir size
        let num = param_length_u128(file_size);
        if mode == FileOperationMode::ReadFile {
            print!("{mode:?}");

            bytes.write_u16::<byteorder::BE>(num).unwrap();
            let source = file_size.to_be_bytes();
            // Compressed
            bytes.extend_from_slice(&source[16 - num as usize..]);
            // Uncompressed
            bytes.extend_from_slice(&source[16 - num as usize..]);
        } else if mode == FileOperationMode::ReadDir {
            bytes.write_u16::<byteorder::BE>(num).unwrap();
            let source = file_size.to_be_bytes();
            // Compressed
            bytes.extend_from_slice(&source[16 - num as usize..]);
        }

        if mode == FileOperationMode::ResumeFile {
            bytes.write_u64::<byteorder::BE>(file_position).unwrap();
        }
        bytes
    }

    #[test]
    fn response_add() {
        let bytes = get_bytes(FileOperationMode::AddFile, 0x1234, 0x00, 0x1234, 0);
        let reader = &mut &bytes[..];
        let resp = RequestFileTransferResponse::from_reader(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(resp.required_size(), bytes.len());

        match resp {
            RequestFileTransferResponse::AddFile(mode, sent_data, data_format) => {
                assert_eq!(mode, FileOperationMode::AddFile);
                assert_eq!(sent_data.length_format_identifier, 2);
                assert_eq!(sent_data.max_number_of_block_length, vec![0x12, 0x34]);
                assert_eq!(data_format, DataFormatIdentifier::new(0, 0).unwrap());
            }
            _ => panic!("Expected AddFile"),
        }
    }

    #[test]
    fn delete_file() {
        let bytes = get_bytes(FileOperationMode::DeleteFile, 0, 0, 0, 0);
        let reader = &mut &bytes[..];
        let resp = RequestFileTransferResponse::from_reader(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(resp.required_size(), bytes.len());

        match resp {
            RequestFileTransferResponse::DeleteFile(mode) => {
                assert_eq!(mode, FileOperationMode::DeleteFile);
            }
            _ => panic!("Expected DeleteFile"),
        }
    }

    #[test]
    fn replace_file() {
        let bytes = get_bytes(FileOperationMode::ReplaceFile, 0x1_1234, 0, 0, 0);
        let reader = &mut &bytes[..];
        let resp = RequestFileTransferResponse::from_reader(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(resp.required_size(), bytes.len());

        match resp {
            RequestFileTransferResponse::ReplaceFile(mode, sent_data, data_format) => {
                assert_eq!(mode, FileOperationMode::ReplaceFile);
                assert_eq!(sent_data.length_format_identifier, 3);
                assert_eq!(sent_data.max_number_of_block_length, vec![0x01, 0x12, 0x34]);
                assert_eq!(data_format, DataFormatIdentifier::new(0, 0).unwrap());
            }
            _ => panic!("Expected ReplaceFile"),
        }
    }

    #[test]
    fn read_file() {
        let bytes = get_bytes(FileOperationMode::ReadFile, 0x1, 0x11, 0x11_1111_1111, 0);
        let reader = &mut &bytes[..];
        let resp = RequestFileTransferResponse::from_reader(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(resp.required_size(), bytes.len());

        match resp {
            RequestFileTransferResponse::ReadFile(mode, sent_data, df, size) => {
                assert_eq!(mode, FileOperationMode::ReadFile);
                assert_eq!(sent_data.length_format_identifier, 1);
                assert_eq!(sent_data.max_number_of_block_length, vec![0x01]);
                assert_eq!(df, DataFormatIdentifier::new(0x01, 0x01).unwrap());
                assert_eq!(size.file_size_parameter_length, 5);
                assert_eq!(size.file_size_uncompressed, 0x11_1111_1111);
                assert_eq!(size.file_size_compressed, 0x11_1111_1111);
            }
            _ => panic!("Expected ReadFile"),
        }
    }

    #[test]
    fn read_dir() {
        let bytes = get_bytes(FileOperationMode::ReadDir, 0x1_1234, 0, 0x11_1111_1111, 0);
        let reader = &mut &bytes[..];
        let resp = RequestFileTransferResponse::from_reader(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(resp.required_size(), bytes.len());

        match resp {
            RequestFileTransferResponse::ReadDir(mode, sent_data, df, size) => {
                assert_eq!(mode, FileOperationMode::ReadDir);
                assert_eq!(sent_data.length_format_identifier, 3);
                assert_eq!(sent_data.max_number_of_block_length, vec![0x01, 0x12, 0x34]);
                assert_eq!(df, DataFormatIdentifier::new(0, 0).unwrap());
                assert_eq!(size.dir_info_parameter_length, 5);
                assert_eq!(size.dir_info_length, 0x11_1111_1111);
            }
            _ => panic!("Expected ReadDir"),
        }
    }

    #[test]
    fn resume_file() {
        let bytes = get_bytes(
            FileOperationMode::ResumeFile,
            0x1_1234,
            0x11,
            0x11_1111_1111,
            0x1234_5678_9ABC_DEF0,
        );
        let reader = &mut &bytes[..];
        let resp = RequestFileTransferResponse::from_reader(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.to_writer(&mut written_bytes).unwrap();
        assert_eq!(written, bytes.len());
        assert_eq!(resp.required_size(), bytes.len());

        match resp {
            RequestFileTransferResponse::ResumeFile(mode, sent_data, df, pos) => {
                assert_eq!(mode, FileOperationMode::ResumeFile);
                assert_eq!(sent_data.length_format_identifier, 3);
                assert_eq!(sent_data.max_number_of_block_length, vec![0x01, 0x12, 0x34]);
                assert_eq!(df, DataFormatIdentifier::new(1, 1).unwrap());
                assert_eq!(pos.file_position, 0x1234_5678_9ABC_DEF0);
            }
            _ => panic!("Expected ResumeFile"),
        }
    }
}
