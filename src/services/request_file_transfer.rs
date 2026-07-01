//! `RequestFileTransfer` (0x38) service implementation

use crate::shared::{DataFormatIdentifier, read_be_uint, write_be_uint};
use crate::{Decode, Encode, Error, param_length_u128, NegativeResponseCode};

/// Minimum byte-width (clamped to at least 1) needed to hold the larger of two size
/// values. Used to derive the on-wire `parameterLength` prefix from the data itself,
/// so the declared length can never disagree with the value it describes.
fn size_param_width(a: u128, b: u128) -> usize {
    let a = param_length_u128(a) as usize;
    let b = param_length_u128(b) as usize;
    let w = a.max(b);
    w.max(1)
}

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
/// [Request]: RequestFileTransferRequest
/// [Response]: RequestFileTransferResponse
#[allow(clippy::struct_field_names)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SizePayload {
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

impl SizePayload {
    /// Build a size payload. The on-wire `fileSizeParameterLength` is derived from the
    /// values themselves (the byte-width needed to hold the larger of the two).
    #[must_use]
    pub const fn new(file_size_uncompressed: u128, file_size_compressed: u128) -> Self {
        Self {
            file_size_uncompressed,
            file_size_compressed,
        }
    }
}

/// Payload used for all [`RequestFileTransferRequest`] requests.
///
/// Borrows `file_path_and_name` from the caller.
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
/// [Request]: RequestFileTransferRequest
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamePayload<'a> {
    /// 0x01 - 0x06, the type of operation to be applied to the file or directory specified in `file_path_and_name`
    pub mode_of_operation: FileOperationMode,

    /// The path and name of the file or directory on the server.
    ///
    /// The on-wire length prefix is derived from this field during encoding, so it can
    /// never disagree with the name it describes.
    pub file_path_and_name: &'a str,
}

impl<'a> NamePayload<'a> {
    /// Build a name payload. The on-wire length prefix is computed from
    /// `file_path_and_name` at encode time.
    #[must_use]
    pub const fn new(mode_of_operation: FileOperationMode, file_path_and_name: &'a str) -> Self {
        Self {
            mode_of_operation,
            file_path_and_name,
        }
    }
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RequestFileTransferRequest<'a> {
    /// Add a file to the server
    AddFile(
        #[cfg_attr(feature = "serde", serde(borrow))] NamePayload<'a>,
        DataFormatIdentifier,
        SizePayload,
    ),

    /// Delete the specified file from the server
    DeleteFile(#[cfg_attr(feature = "serde", serde(borrow))] NamePayload<'a>),

    /// Replace the specified file on the server, if it does not exist, add it
    ReplaceFile(
        #[cfg_attr(feature = "serde", serde(borrow))] NamePayload<'a>,
        DataFormatIdentifier,
        SizePayload,
    ),

    /// Read the specified file from the server (upload)
    ReadFile(
        #[cfg_attr(feature = "serde", serde(borrow))] NamePayload<'a>,
        DataFormatIdentifier,
    ),

    /// Read the directory from the server
    /// Implies that the request does not include a `fileName`
    ReadDir(#[cfg_attr(feature = "serde", serde(borrow))] NamePayload<'a>),

    /// Resume a file transfer at the returned `filePosition` indicator
    /// The file must already exist in the ECU's filesystem
    ResumeFile(
        #[cfg_attr(feature = "serde", serde(borrow))] NamePayload<'a>,
        DataFormatIdentifier,
        SizePayload,
    ),
}

const REQUEST_FILE_TRANSFER_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::UploadDownloadNotAccepted,
];

impl RequestFileTransferRequest<'_> {
    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &REQUEST_FILE_TRANSFER_NEGATIVE_RESPONSE_CODES
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
/// [Response]: RequestFileTransferResponse
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SentDataPayload<'a> {
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
    pub max_number_of_block_length: &'a [u8],
}

impl<'a> SentDataPayload<'a> {
    /// Build a sent-data payload. The on-wire `lengthFormatIdentifier` is derived from
    /// the length of `max_number_of_block_length` at encode time.
    #[must_use]
    pub const fn new(max_number_of_block_length: &'a [u8]) -> Self {
        Self {
            max_number_of_block_length,
        }
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
/// [Response]: RequestFileTransferResponse
#[allow(clippy::struct_field_names)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileSizePayload {
    /// Size of the uncompressed file in bytes.
    pub file_size_uncompressed: u128,
    /// Size of the compressed file in bytes.
    pub file_size_compressed: u128,
}

impl FileSizePayload {
    /// Build a file-size payload. The on-wire `fileSizeOrDirInfoParameterLength` is
    /// derived from the values (the byte-width needed to hold the larger of the two).
    #[must_use]
    pub const fn new(file_size_uncompressed: u128, file_size_compressed: u128) -> Self {
        Self {
            file_size_uncompressed,
            file_size_compressed,
        }
    }

    /// Byte-width of each size field on the wire, derived from the values.
    fn width(&self) -> usize {
        size_param_width(self.file_size_uncompressed, self.file_size_compressed)
    }
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
/// [Response]: RequestFileTransferResponse
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DirSizePayload {
    /// Total size of the directory information in bytes.
    pub dir_info_length: u128,
}

impl DirSizePayload {
    /// Build a directory-size payload. The on-wire
    /// `fileSizeOrDirInfoParameterLength` is derived from `dir_info_length`.
    #[must_use]
    pub const fn new(dir_info_length: u128) -> Self {
        Self { dir_info_length }
    }

    /// Byte-width of the `dir_info_length` field on the wire, derived from the value.
    fn width(&self) -> usize {
        size_param_width(self.dir_info_length, 0)
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
/// [Response]: RequestFileTransferResponse
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

impl PositionPayload {
    /// Build a position payload from the resume byte position (fixed 8 bytes on the wire).
    #[must_use]
    pub const fn new(file_position: u64) -> Self {
        Self { file_position }
    }
}

/// Response to a [`RequestFileTransferRequest`] from the server
///
/// The server will respond with a [`RequestFileTransferResponse`] to indicate the status of the request
/// `DataFormatIdentifier` - Echoes the value of the request
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RequestFileTransferResponse<'a> {
    /// Positive response to an [`AddFile`](FileOperationMode::AddFile) request.
    AddFile(
        FileOperationMode,
        #[cfg_attr(feature = "serde", serde(borrow))] SentDataPayload<'a>,
        DataFormatIdentifier,
    ),
    /// Positive response to a [`DeleteFile`](FileOperationMode::DeleteFile) request.
    DeleteFile(FileOperationMode),
    /// Positive response to a [`ReplaceFile`](FileOperationMode::ReplaceFile) request.
    ReplaceFile(
        FileOperationMode,
        #[cfg_attr(feature = "serde", serde(borrow))] SentDataPayload<'a>,
        DataFormatIdentifier,
    ),
    /// Positive response to a [`ReadFile`](FileOperationMode::ReadFile) request, including file size.
    ReadFile(
        FileOperationMode,
        #[cfg_attr(feature = "serde", serde(borrow))] SentDataPayload<'a>,
        DataFormatIdentifier,
        FileSizePayload,
    ),
    /// Positive response to a [`ReadDir`](FileOperationMode::ReadDir) request, including directory size.
    ReadDir(
        FileOperationMode,
        #[cfg_attr(feature = "serde", serde(borrow))] SentDataPayload<'a>,
        DataFormatIdentifier,
        DirSizePayload,
    ),
    /// Positive response to a [`ResumeFile`](FileOperationMode::ResumeFile) request, including file position.
    ResumeFile(
        FileOperationMode,
        #[cfg_attr(feature = "serde", serde(borrow))] SentDataPayload<'a>,
        DataFormatIdentifier,
        PositionPayload,
    ),
}

// ---------------------------------------------------------------------------
// Encode / Decode impls
// ---------------------------------------------------------------------------

impl Encode for NamePayload<'_> {
    fn encoded_size(&self) -> usize {
        1 + 2 + self.file_path_and_name.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let name = self.file_path_and_name.as_bytes();
        let name_len =
            u16::try_from(name.len()).map_err(|_| Error::IncorrectMessageLengthOrInvalidFormat)?;
        writer
            .write_all(&[u8::from(self.mode_of_operation)])
            .map_err(Error::io)?;
        writer
            .write_all(&name_len.to_be_bytes())
            .map_err(Error::io)?;
        writer.write_all(name).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for NamePayload<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 3 {
            return Err(Error::InsufficientData(3));
        }
        let mode_of_operation = FileOperationMode::try_from(buf[0])?;
        let name_len = u16::from_be_bytes([buf[1], buf[2]]) as usize;
        let total = 3 + name_len;
        if buf.len() < total {
            return Err(Error::InsufficientData(total));
        }
        let file_path_and_name = core::str::from_utf8(&buf[3..total])
            .map_err(|_| Error::IncorrectMessageLengthOrInvalidFormat)?;
        Ok((
            Self {
                mode_of_operation,
                file_path_and_name,
            },
            &buf[total..],
        ))
    }
}

impl SizePayload {
    /// Byte-width of each size field on the wire, derived from the values.
    fn width(&self) -> usize {
        size_param_width(self.file_size_uncompressed, self.file_size_compressed)
    }
}

impl Encode for SizePayload {
    fn encoded_size(&self) -> usize {
        1 + 2 * self.width()
    }

    #[allow(clippy::cast_possible_truncation)] // width() <= 16, fits in u8
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let n = self.width();
        writer.write_all(&[n as u8]).map_err(Error::io)?;
        write_be_uint(self.file_size_uncompressed, n, writer)?;
        write_be_uint(self.file_size_compressed, n, writer)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for SizePayload {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let n = buf[0] as usize;
        let total = 1 + 2 * n;
        if buf.len() < total {
            return Err(Error::InsufficientData(total));
        }
        let file_size_uncompressed = read_be_uint(&buf[1..], n)?;
        let file_size_compressed = read_be_uint(&buf[1 + n..], n)?;
        Ok((
            Self {
                file_size_uncompressed,
                file_size_compressed,
            },
            &buf[total..],
        ))
    }
}

impl Encode for SentDataPayload<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.max_number_of_block_length.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let len = u8::try_from(self.max_number_of_block_length.len())
            .map_err(|_| Error::IncorrectMessageLengthOrInvalidFormat)?;
        writer.write_all(&[len]).map_err(Error::io)?;
        writer
            .write_all(self.max_number_of_block_length)
            .map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for SentDataPayload<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let n = buf[0] as usize;
        let total = 1 + n;
        if buf.len() < total {
            return Err(Error::InsufficientData(total));
        }
        Ok((
            Self {
                max_number_of_block_length: &buf[1..total],
            },
            &buf[total..],
        ))
    }
}

impl Encode for FileSizePayload {
    fn encoded_size(&self) -> usize {
        2 + 2 * self.width()
    }

    #[allow(clippy::cast_possible_truncation)] // width() <= 16, fits in u16
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let n = self.width();
        writer
            .write_all(&(n as u16).to_be_bytes())
            .map_err(Error::io)?;
        write_be_uint(self.file_size_uncompressed, n, writer)?;
        write_be_uint(self.file_size_compressed, n, writer)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for FileSizePayload {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(2));
        }
        let n = u16::from_be_bytes([buf[0], buf[1]]) as usize;
        let total = 2 + 2 * n;
        if buf.len() < total {
            return Err(Error::InsufficientData(total));
        }
        let file_size_uncompressed = read_be_uint(&buf[2..], n)?;
        let file_size_compressed = read_be_uint(&buf[2 + n..], n)?;
        Ok((
            Self {
                file_size_uncompressed,
                file_size_compressed,
            },
            &buf[total..],
        ))
    }
}

impl Encode for DirSizePayload {
    fn encoded_size(&self) -> usize {
        2 + self.width()
    }

    #[allow(clippy::cast_possible_truncation)] // width() <= 16, fits in u16
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let n = self.width();
        writer
            .write_all(&(n as u16).to_be_bytes())
            .map_err(Error::io)?;
        write_be_uint(self.dir_info_length, n, writer)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for DirSizePayload {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(2));
        }
        let n = u16::from_be_bytes([buf[0], buf[1]]) as usize;
        let total = 2 + n;
        if buf.len() < total {
            return Err(Error::InsufficientData(total));
        }
        let dir_info_length = read_be_uint(&buf[2..], n)?;
        Ok((Self { dir_info_length }, &buf[total..]))
    }
}

impl Encode for PositionPayload {
    fn encoded_size(&self) -> usize {
        8
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&self.file_position.to_be_bytes())
            .map_err(Error::io)?;
        Ok(8)
    }
}

impl<'a> Decode<'a> for PositionPayload {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 8 {
            return Err(Error::InsufficientData(8));
        }
        let file_position = u64::from_be_bytes([
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ]);
        Ok((Self { file_position }, &buf[8..]))
    }
}

impl Encode for RequestFileTransferRequest<'_> {
    fn encoded_size(&self) -> usize {
        match self {
            Self::AddFile(name, _, size)
            | Self::ReplaceFile(name, _, size)
            | Self::ResumeFile(name, _, size) => name.encoded_size() + 1 + size.encoded_size(),
            Self::ReadFile(name, _) => name.encoded_size() + 1,
            Self::DeleteFile(name) | Self::ReadDir(name) => name.encoded_size(),
        }
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let mut len;
        match self {
            Self::AddFile(name, dfi, size)
            | Self::ReplaceFile(name, dfi, size)
            | Self::ResumeFile(name, dfi, size) => {
                len = name.encode(writer)?;
                writer.write_all(&[u8::from(*dfi)]).map_err(Error::io)?;
                len += 1;
                len += size.encode(writer)?;
            }
            Self::ReadFile(name, dfi) => {
                len = name.encode(writer)?;
                writer.write_all(&[u8::from(*dfi)]).map_err(Error::io)?;
                len += 1;
            }
            Self::DeleteFile(name) | Self::ReadDir(name) => {
                len = name.encode(writer)?;
            }
        }
        Ok(len)
    }
}

impl<'a> Decode<'a> for RequestFileTransferRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        let (name, rest) = NamePayload::decode(buf)?;
        match name.mode_of_operation {
            FileOperationMode::DeleteFile => Ok((Self::DeleteFile(name), rest)),
            FileOperationMode::ReadDir => Ok((Self::ReadDir(name), rest)),
            FileOperationMode::ReadFile => {
                if rest.is_empty() {
                    return Err(Error::InsufficientData(1));
                }
                let dfi = DataFormatIdentifier::from(rest[0]);
                Ok((Self::ReadFile(name, dfi), &rest[1..]))
            }
            mode @ (FileOperationMode::AddFile
            | FileOperationMode::ReplaceFile
            | FileOperationMode::ResumeFile) => {
                if rest.is_empty() {
                    return Err(Error::InsufficientData(1));
                }
                let dfi = DataFormatIdentifier::from(rest[0]);
                let (size, rest) = SizePayload::decode(&rest[1..])?;
                let value = match mode {
                    FileOperationMode::AddFile => Self::AddFile(name, dfi, size),
                    FileOperationMode::ReplaceFile => Self::ReplaceFile(name, dfi, size),
                    FileOperationMode::ResumeFile => Self::ResumeFile(name, dfi, size),
                    _ => unreachable!(),
                };
                Ok((value, rest))
            }
            FileOperationMode::ISOSAEReserved(b) => Err(Error::InvalidFileOperationMode(b)),
        }
    }
}

impl Encode for RequestFileTransferResponse<'_> {
    fn encoded_size(&self) -> usize {
        match self {
            Self::DeleteFile(_) => 1,
            Self::AddFile(_, sent, _) | Self::ReplaceFile(_, sent, _) => {
                1 + sent.encoded_size() + 1
            }
            Self::ReadFile(_, sent, _, fs) => 1 + sent.encoded_size() + 1 + fs.encoded_size(),
            Self::ReadDir(_, sent, _, ds) => 1 + sent.encoded_size() + 1 + ds.encoded_size(),
            Self::ResumeFile(_, sent, _, pos) => 1 + sent.encoded_size() + 1 + pos.encoded_size(),
        }
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let mut len = 1;
        match self {
            Self::DeleteFile(mode) => {
                writer.write_all(&[u8::from(*mode)]).map_err(Error::io)?;
            }
            Self::AddFile(mode, sent, dfi) | Self::ReplaceFile(mode, sent, dfi) => {
                writer.write_all(&[u8::from(*mode)]).map_err(Error::io)?;
                len += sent.encode(writer)?;
                writer.write_all(&[u8::from(*dfi)]).map_err(Error::io)?;
                len += 1;
            }
            Self::ReadFile(mode, sent, dfi, fs) => {
                writer.write_all(&[u8::from(*mode)]).map_err(Error::io)?;
                len += sent.encode(writer)?;
                writer.write_all(&[u8::from(*dfi)]).map_err(Error::io)?;
                len += 1;
                len += fs.encode(writer)?;
            }
            Self::ReadDir(mode, sent, dfi, ds) => {
                writer.write_all(&[u8::from(*mode)]).map_err(Error::io)?;
                len += sent.encode(writer)?;
                writer.write_all(&[u8::from(*dfi)]).map_err(Error::io)?;
                len += 1;
                len += ds.encode(writer)?;
            }
            Self::ResumeFile(mode, sent, dfi, pos) => {
                writer.write_all(&[u8::from(*mode)]).map_err(Error::io)?;
                len += sent.encode(writer)?;
                writer.write_all(&[u8::from(*dfi)]).map_err(Error::io)?;
                len += 1;
                len += pos.encode(writer)?;
            }
        }
        Ok(len)
    }
}

impl<'a> Decode<'a> for RequestFileTransferResponse<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let mode = FileOperationMode::try_from(buf[0])?;
        let rest = &buf[1..];
        match mode {
            FileOperationMode::DeleteFile => Ok((Self::DeleteFile(mode), rest)),
            FileOperationMode::AddFile | FileOperationMode::ReplaceFile => {
                let (sent, rest) = SentDataPayload::decode(rest)?;
                if rest.is_empty() {
                    return Err(Error::InsufficientData(1));
                }
                let dfi = DataFormatIdentifier::from(rest[0]);
                let rest = &rest[1..];
                let value = match mode {
                    FileOperationMode::AddFile => Self::AddFile(mode, sent, dfi),
                    FileOperationMode::ReplaceFile => Self::ReplaceFile(mode, sent, dfi),
                    _ => unreachable!(),
                };
                Ok((value, rest))
            }
            FileOperationMode::ReadFile => {
                let (sent, rest) = SentDataPayload::decode(rest)?;
                if rest.is_empty() {
                    return Err(Error::InsufficientData(1));
                }
                let dfi = DataFormatIdentifier::from(rest[0]);
                let (fs, rest) = FileSizePayload::decode(&rest[1..])?;
                Ok((Self::ReadFile(mode, sent, dfi, fs), rest))
            }
            FileOperationMode::ReadDir => {
                let (sent, rest) = SentDataPayload::decode(rest)?;
                if rest.is_empty() {
                    return Err(Error::InsufficientData(1));
                }
                let dfi = DataFormatIdentifier::from(rest[0]);
                let (ds, rest) = DirSizePayload::decode(&rest[1..])?;
                Ok((Self::ReadDir(mode, sent, dfi, ds), rest))
            }
            FileOperationMode::ResumeFile => {
                let (sent, rest) = SentDataPayload::decode(rest)?;
                if rest.is_empty() {
                    return Err(Error::InsufficientData(1));
                }
                let dfi = DataFormatIdentifier::from(rest[0]);
                let (pos, rest) = PositionPayload::decode(&rest[1..])?;
                Ok((Self::ResumeFile(mode, sent, dfi, pos), rest))
            }
            FileOperationMode::ISOSAEReserved(b) => Err(Error::InvalidFileOperationMode(b)),
        }
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;
    use crate::NegativeResponseCode;

    #[test]
    fn test_allowed_nack_codes() {
        let codes = RequestFileTransferRequest::allowed_nack_codes();
        assert!(codes.contains(&NegativeResponseCode::UploadDownloadNotAccepted));
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

    fn name_payload(mode: FileOperationMode, path: &str) -> NamePayload<'_> {
        NamePayload::new(mode, path)
    }

    #[test]
    fn name_payload_roundtrip() {
        let path = "/tmp/foo.bin";
        let n = name_payload(FileOperationMode::AddFile, path);
        let mut buf = [0u8; 64];
        let written = Encode::encode(&n, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, n.encoded_size());
        let (decoded, rest) = NamePayload::decode(&buf[..written]).unwrap();
        assert!(rest.is_empty());
        assert_eq!(decoded, n);
        assert_encode_size_agrees(&n);
    }

    #[test]
    fn size_payload_roundtrip() {
        let s = SizePayload::new(u128::from(u64::MAX) + 1000, 0x12_3456);
        let mut buf = [0u8; 32];
        let written = Encode::encode(&s, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, s.encoded_size());
        let (decoded, rest) = SizePayload::decode(&buf[..written]).unwrap();
        assert!(rest.is_empty());
        assert_eq!(decoded, s);
        assert_encode_size_agrees(&s);
    }

    #[test]
    fn size_payload_derives_minimal_width_from_data() {
        // Small values must serialize with a 1-byte width derived from the data,
        // not a caller-supplied length that could disagree with the value.
        let s = SizePayload::new(0x12, 0x34);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&s, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x01, 0x12, 0x34]);
    }

    #[test]
    fn name_payload_length_prefix_matches_name() {
        // The 2-byte length prefix is always exactly the UTF-8 length of the name.
        let n = NamePayload::new(FileOperationMode::DeleteFile, "abc");
        let mut buf = [0u8; 16];
        let written = Encode::encode(&n, &mut buf.as_mut_slice()).unwrap();
        // mode=0x02, length=0x0003, "abc"
        assert_eq!(&buf[..written], &[0x02, 0x00, 0x03, b'a', b'b', b'c']);
    }

    #[test]
    fn add_file_request_roundtrip() {
        let path = "test.txt";
        let req = RequestFileTransferRequest::AddFile(
            name_payload(FileOperationMode::AddFile, path),
            DataFormatIdentifier::from(0x00),
            SizePayload::new(0x1234, 0x1234),
        );
        let mut buf = [0u8; 64];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, req.encoded_size());
        let (decoded, rest) = RequestFileTransferRequest::decode(&buf[..written]).unwrap();
        assert!(rest.is_empty());
        assert_eq!(decoded, req);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn delete_file_request_roundtrip() {
        let path = "/var/tmp/delete_file.bin";
        let req = RequestFileTransferRequest::DeleteFile(name_payload(
            FileOperationMode::DeleteFile,
            path,
        ));
        let mut buf = [0u8; 64];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, req.encoded_size());
        let (decoded, rest) = RequestFileTransferRequest::decode(&buf[..written]).unwrap();
        assert!(rest.is_empty());
        assert_eq!(decoded, req);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn read_file_request_roundtrip() {
        let path = "/etc/passwd";
        let req = RequestFileTransferRequest::ReadFile(
            name_payload(FileOperationMode::ReadFile, path),
            DataFormatIdentifier::from(0x11),
        );
        let mut buf = [0u8; 64];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, req.encoded_size());
        let (decoded, rest) = RequestFileTransferRequest::decode(&buf[..written]).unwrap();
        assert!(rest.is_empty());
        assert_eq!(decoded, req);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn read_dir_request_roundtrip() {
        let path = "/var/log";
        let req =
            RequestFileTransferRequest::ReadDir(name_payload(FileOperationMode::ReadDir, path));
        let mut buf = [0u8; 64];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        let (decoded, _) = RequestFileTransferRequest::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, req);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn resume_file_request_roundtrip() {
        let path = "/big/file.bin";
        let req = RequestFileTransferRequest::ResumeFile(
            name_payload(FileOperationMode::ResumeFile, path),
            DataFormatIdentifier::from(0x00),
            SizePayload::new(0xDEAD_BEEF, 0xDEAD_BEEF),
        );
        let mut buf = [0u8; 64];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        let (decoded, _) = RequestFileTransferRequest::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, req);
        assert_encode_size_agrees(&req);
    }
}

#[cfg(test)]
mod response_tests {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    fn sent_data(block: &[u8]) -> SentDataPayload<'_> {
        SentDataPayload::new(block)
    }

    #[test]
    fn add_file_response_roundtrip() {
        let block = [0x10u8, 0x00];
        let resp = RequestFileTransferResponse::AddFile(
            FileOperationMode::AddFile,
            sent_data(&block),
            DataFormatIdentifier::from(0x00),
        );
        let mut buf = [0u8; 32];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, resp.encoded_size());
        let (decoded, remaining) = RequestFileTransferResponse::decode(&buf[..written]).unwrap();
        assert!(remaining.is_empty());
        assert_eq!(decoded, resp);
        assert_encode_size_agrees(&resp);
    }

    #[test]
    fn delete_file_response_roundtrip() {
        let resp = RequestFileTransferResponse::DeleteFile(FileOperationMode::DeleteFile);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        let (decoded, _) = RequestFileTransferResponse::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, resp);
        assert_encode_size_agrees(&resp);
    }

    #[test]
    fn read_file_response_roundtrip() {
        let block = [0x04u8, 0x00];
        let resp = RequestFileTransferResponse::ReadFile(
            FileOperationMode::ReadFile,
            sent_data(&block),
            DataFormatIdentifier::from(0x00),
            FileSizePayload::new(0xAABB_CCDD, 0x1122_3344),
        );
        let mut buf = [0u8; 64];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        let (decoded, _) = RequestFileTransferResponse::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, resp);
        assert_encode_size_agrees(&resp);
    }

    #[test]
    fn read_dir_response_roundtrip() {
        let block = [0x04u8, 0x00];
        let resp = RequestFileTransferResponse::ReadDir(
            FileOperationMode::ReadDir,
            sent_data(&block),
            DataFormatIdentifier::from(0x00),
            DirSizePayload::new(0x1234_5678),
        );
        let mut buf = [0u8; 64];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        let (decoded, _) = RequestFileTransferResponse::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, resp);
        assert_encode_size_agrees(&resp);
    }

    #[test]
    fn resume_file_response_roundtrip() {
        let block = [0x04u8, 0x00];
        let resp = RequestFileTransferResponse::ResumeFile(
            FileOperationMode::ResumeFile,
            sent_data(&block),
            DataFormatIdentifier::from(0x00),
            PositionPayload {
                file_position: 0xDEAD_BEEF_CAFE_BABE,
            },
        );
        let mut buf = [0u8; 64];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        let (decoded, _) = RequestFileTransferResponse::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, resp);
        assert_encode_size_agrees(&resp);
    }

    #[test]
    const fn derive_contract() {
        use crate::test_util::assert_impl_eq;

        // Verify all nine types implement Eq
        assert_impl_eq::<FileOperationMode>();
        assert_impl_eq::<SizePayload>();
        assert_impl_eq::<NamePayload<'static>>();
        assert_impl_eq::<RequestFileTransferRequest<'static>>();
        assert_impl_eq::<SentDataPayload<'static>>();
        assert_impl_eq::<FileSizePayload>();
        assert_impl_eq::<DirSizePayload>();
        assert_impl_eq::<PositionPayload>();
        assert_impl_eq::<RequestFileTransferResponse<'static>>();
    }
}
