use thiserror::Error;

/// Errors that can occur during UDS message encoding, decoding, or validation.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An underlying I/O error occurred while reading or writing.
    #[error("I/O error: {0:?}")]
    IoError(embedded_io::ErrorKind),
    /// The byte stream contained fewer bytes than expected.
    #[error("Insufficient data. Expected {0} bytes.")]
    InsufficientData(usize),
    /// The u16 value does not map to a known diagnostic identifier.
    #[error("Invalid Diagnostic Identifier: {0:X}")]
    InvalidDiagnosticIdentifier(u16),
    /// The u16 identifier is unrecognised and carried an unexpected payload.
    #[error("Invalid Diagnostic Identifier with payload: {0:X}")]
    InvalidDiagnosticIdentifierPayload(u16),
    /// The session-type byte is not a valid [`DiagnosticSessionType`](crate::DiagnosticSessionType).
    #[error("Invalid diagnostic session type: {0}")]
    InvalidDiagnosticSessionType(u8),
    /// The reset-type byte is not a valid [`ResetType`](crate::ResetType).
    #[error("Invalid ECU reset type: {0}")]
    InvalidEcuResetType(u8),
    /// The security-access–type byte is not a valid [`SecurityAccessType`](crate::SecurityAccessType).
    #[error("Invalid Security Access Type: {0}")]
    InvalidSecurityAccessType(u8),
    /// The communication-control–type byte is not a valid [`CommunicationControlType`](crate::CommunicationControlType).
    #[error("Invalid Communication Control Type: {0}")]
    InvalidCommunicationControlType(u8),
    /// The communication-type byte is not a valid [`CommunicationType`](crate::CommunicationType).
    #[error("Invalid Communication Type: {0}")]
    InvalidCommunicationType(u8),
    /// The tester-present–type byte is not valid.
    #[error("Invalid Tester Present Type: {0}")]
    InvalidTesterPresentType(u8),
    /// The message length did not match the expected format.
    #[error("Incorrect Message Length Or Invalid Format")]
    IncorrectMessageLengthOrInvalidFormat,
    /// The memory address value is out of the valid range.
    #[error("Invalid Memory Address: {0}")]
    InvalidMemoryAddress(u64),
    /// The encryption or compression method byte is not recognised.
    #[error("Invalid Encryption/Compression Method: {0}")]
    InvalidEncryptionCompressionMethod(u8),
    /// A payload was expected but the stream contained no data.
    #[error("Data required but found none")]
    NoDataAvailable,
    /// The `RequestFileTransfer` `modeOfOperation` byte is not valid.
    #[error("Invalid FileTransfer modeOfOperation (server will send requestOutOfRange): {0}")]
    InvalidFileOperationMode(u8),
    /// The `fileSizeParameterLength` / `fileSizeOrDirInfoParameterLength` value is outside 1–16.
    #[error("Invalid fileSizeParameterLength (valid range: 1..=16): {0}")]
    InvalidFileSizeParameterLength(u16),
    /// The `ReadDTCInformation` sub-function byte is not valid.
    #[error("Invalid DTC Subfunction Type: {0}")]
    InvalidDtcSubfunctionType(u8),
    /// The DTC format identifier byte is not recognised.
    #[error("Invalid DTC Format Identifier: {0}")]
    InvalidDtcFormatIdentifier(u8),
    /// The routine-control sub-function byte is not a valid [`RoutineControlSubFunction`](crate::RoutineControlSubFunction).
    #[error("Invalid Routine Control Sub-Function: {0}")]
    InvalidRoutineControlSubFunction(u8),
    /// The DTC-setting byte is not a valid [`DtcSettings`](crate::DtcSettings) value.
    #[error("Invalid DTC Setting: {0}")]
    InvalidDtcSetting(u8),
    /// The value is reserved for legislative use and must not be used.
    #[error("Reserved for legislative use: {0}")]
    ReservedForLegislativeUse(u8),
    /// The service type is not yet implemented in this crate.
    #[error("UDS service not implemented: {0:?}")]
    ServiceNotImplemented(crate::UdsServiceType),
}

impl Error {
    /// Convert any `embedded_io::Error` into [`Error::IoError`].
    #[inline]
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn io<E: embedded_io::Error>(e: E) -> Self {
        Self::IoError(e.kind())
    }
}

impl From<embedded_io::ErrorKind> for Error {
    fn from(kind: embedded_io::ErrorKind) -> Self {
        Self::IoError(kind)
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(_err: std::io::Error) -> Self {
        Self::IoError(embedded_io::ErrorKind::Other)
    }
}
