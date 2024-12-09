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
}
