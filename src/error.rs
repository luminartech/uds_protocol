use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
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
}
