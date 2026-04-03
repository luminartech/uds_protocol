//! `RequestFileTransfer` (0x38) service implementation

use crate::{DataFormatIdentifier, Error};

///////////////////////////////////////// - Request - ///////////////////////////////////////////////////
/// Mode of operation for file transfer requests
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FileOperationMode {
    /// ISO/SAE reserved (`0x00`, `0x07–0xFF`).
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
#[allow(clippy::struct_field_names)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
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

/// A request to the server to transfer a file, either upload or download.
///
/// Capabilities:
///   * Receive information about the file system on the server
///   * Send/Receive files to/from the server
///
/// Available as an alternative to [`RequestDownloadRequest`](crate::RequestDownloadRequest) and `RequestUploadRequest`
/// if the server implements a file system for data storage
///
/// Use [`crate::UdsServiceType::TransferData`] to send the file data to the server and [`crate::UdsServiceType::RequestTransferExit`] to end the transfer
///
/// If this service is used to delete files or directories on the server,
/// there is no need to use the `TransferData` or [`crate::UdsServiceType::RequestTransferExit`] services.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
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
#[allow(clippy::struct_field_names)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
pub struct FileSizePayload {
    /// Length in bytes of both `file_size_uncompressed` and `file_size_compressed`.
    pub file_size_parameter_length: u16,
    /// Size of the uncompressed file in bytes.
    pub file_size_uncompressed: u128,
    /// Size of the compressed file in bytes.
    pub file_size_compressed: u128,
}

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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
pub struct DirSizePayload {
    /// Length in bytes of the `dir_info_length` field.
    pub dir_info_parameter_length: u16,
    /// Total size of the directory information in bytes.
    pub dir_info_length: u128,
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionPayload {
    /// Specifies the byte position within the file at which the Tester will resume downloading after an initial download is suspended
    /// A download is suspended when the ECU stops receiving [`crate::TransferDataRequest`] requests and does not receive the
    /// `RequestTransferExit` request to end the transfer before returning to the default session
    ///
    /// Fixed size: 8 bytes
    ///
    /// Not included for [`AddFile`][FileOperationMode::AddFile], [`DeleteFile`][FileOperationMode::DeleteFile], [`ReplaceFile`][FileOperationMode::ReplaceFile], [`ReadFile`][FileOperationMode::ReadFile], or [`ReadDir`][FileOperationMode::ReadDir]
    /// Only present if `mode_of_operation` is [`ResumeFile`][FileOperationMode::ResumeFile] (for ISO 14229-1:2020)
    pub file_position: u64,
}

/// Response to a [`RequestFileTransferRequest`] from the server
///
/// The server will respond with a [`RequestFileTransferResponse`] to indicate the status of the request
/// `DataFormatIdentifier` - Echoes the value of the request
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum RequestFileTransferResponse {
    /// Positive response to an [`AddFile`](FileOperationMode::AddFile) request.
    AddFile(FileOperationMode, SentDataPayload, DataFormatIdentifier),
    /// Positive response to a [`DeleteFile`](FileOperationMode::DeleteFile) request.
    DeleteFile(FileOperationMode),
    /// Positive response to a [`ReplaceFile`](FileOperationMode::ReplaceFile) request.
    ReplaceFile(FileOperationMode, SentDataPayload, DataFormatIdentifier),
    /// Positive response to a [`ReadFile`](FileOperationMode::ReadFile) request, including file size.
    ReadFile(
        FileOperationMode,
        SentDataPayload,
        DataFormatIdentifier,
        FileSizePayload,
    ),
    /// Positive response to a [`ReadDir`](FileOperationMode::ReadDir) request, including directory size.
    ReadDir(
        FileOperationMode,
        SentDataPayload,
        DataFormatIdentifier,
        DirSizePayload,
    ),
    /// Positive response to a [`ResumeFile`](FileOperationMode::ResumeFile) request, including file position.
    ResumeFile(
        FileOperationMode,
        SentDataPayload,
        DataFormatIdentifier,
        PositionPayload,
    ),
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
            .write_u16::<byteorder_embedded_io::BigEndian>(file_name.len() as u16)
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
            RequestFileTransferRequest::decode(&mut bytes.as_slice()).unwrap();

        let mut written_bytes = Vec::new();
        let written = req.encode(&mut written_bytes).unwrap();
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
        let req = RequestFileTransferRequest::decode(&mut bytes.as_slice()).unwrap();

        let mut written_bytes = Vec::new();
        let written = req.encode(&mut written_bytes).unwrap();
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
        let req = RequestFileTransferRequest::decode(&mut bytes.as_slice()).unwrap();

        let mut bytes = Vec::new();
        let written = req.encode(&mut bytes).unwrap();
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
        let written = req.encode(&mut bytes).unwrap();
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
        let req = RequestFileTransferRequest::decode(&mut bytes.as_slice()).unwrap();

        let mut written_bytes = Vec::new();
        let written = req.encode(&mut written_bytes).unwrap();
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
        let req = RequestFileTransferRequest::decode(&mut bytes.as_slice()).unwrap();

        let mut written_bytes = Vec::new();
        let written = req.encode(&mut written_bytes).unwrap();
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
        let req = RequestFileTransferRequest::decode(&mut bytes.as_slice()).unwrap();
        let mut written_bytes = Vec::new();
        let written = req.encode(&mut written_bytes).unwrap();
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

            bytes
                .write_u16::<byteorder_embedded_io::BigEndian>(num)
                .unwrap();
            let source = file_size.to_be_bytes();
            // Compressed
            bytes.extend_from_slice(&source[16 - num as usize..]);
            // Uncompressed
            bytes.extend_from_slice(&source[16 - num as usize..]);
        } else if mode == FileOperationMode::ReadDir {
            bytes
                .write_u16::<byteorder_embedded_io::BigEndian>(num)
                .unwrap();
            let source = file_size.to_be_bytes();
            // Compressed
            bytes.extend_from_slice(&source[16 - num as usize..]);
        }

        if mode == FileOperationMode::ResumeFile {
            bytes
                .write_u64::<byteorder_embedded_io::BigEndian>(file_position)
                .unwrap();
        }
        bytes
    }

    #[test]
    fn response_add() {
        let bytes = get_bytes(FileOperationMode::AddFile, 0x1234, 0x00, 0x1234, 0);
        let reader = &mut &bytes[..];
        let resp = RequestFileTransferResponse::decode(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.encode(&mut written_bytes).unwrap();
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
        let resp = RequestFileTransferResponse::decode(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.encode(&mut written_bytes).unwrap();
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
        let resp = RequestFileTransferResponse::decode(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.encode(&mut written_bytes).unwrap();
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
        let resp = RequestFileTransferResponse::decode(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.encode(&mut written_bytes).unwrap();
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
        let resp = RequestFileTransferResponse::decode(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.encode(&mut written_bytes).unwrap();
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
        let resp = RequestFileTransferResponse::decode(reader).unwrap();
        assert!(reader.is_empty());

        let mut written_bytes = Vec::new();
        let written = resp.encode(&mut written_bytes).unwrap();
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
