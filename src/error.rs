use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Invalid diagnostic session type: {0}")]
    InvalidDiagnosticSessionType(u8),
}
