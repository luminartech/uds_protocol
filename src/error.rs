use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Insufficient data. Expected {0} bytes.")]
    InsufficientData(usize),
    #[error("Invalid Diagnostic Identifier: {0:#X}")]
    InvalidDiagnosticIdentifier(u16),
    #[error("Invalid diagnostic session type: {0}")]
    InvalidDiagnosticSessionType(u8),
    #[error("Invalid ECU reset type: {0}")]
    InvalidEcuResetType(u8),
    #[error("Invalid Security Access Type: {0}")]
    InvalidSecurityAccessType(u8),
    #[error("Invalid Communication Control Type: {0}")]
    InvalidCommunicationControlType(u8),
    #[error("Invalid Communication Type: {0}")]
    InvalidCommunicationType(u8),
    #[error("Invalid Tester Present Type: {0}")]
    InvalidTesterPresentType(u8),
    #[error("Incorrect Message Length Or Invalid Format")]
    IncorrectMessageLengthOrInvalidFormat,
    #[error("Invalid Memory Address: {0}")]
    InvalidMemoryAddress(u64),
    #[error("Invalid Encryption/Compression Method: {0}")]
    InvalidEncryptionCompressionMethod(u8),
    #[error("Data required but found none")]
    NoDataAvailable,
    #[error("Invalid FileTransfer modeOfOperation (server will send requestOutOfRange): {0}")]
    InvalidFileOperationMode(u8),
    #[error("Invalid file size parameter length (valid values = 1,2,3,4,8,16): {0}")]
    InvalidFileSizeParameterLength(u8),
    #[error("Invalid DTC Subfunction Type: {0}")]
    InvalidDtcSubfunctionType(u8),
    #[error("Invalid DTC Format Identifier: {0}")]
    InvalidDtcFormatIdentifier(u8),
    #[error("Reserved for legislative use: {0} ({1})")]
    ReservedForLegislativeUse(String, u8),
}
