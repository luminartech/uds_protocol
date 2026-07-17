use automotive_wire_codec::{
    Incomplete, InvalidWidth, ReadUintError, TrailingBytes, WriteUintError,
};
use thiserror::Error;

/// Errors that can occur during UDS message encoding, decoding, or validation.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An underlying I/O error occurred while reading or writing.
    #[error("I/O error: {0:?}")]
    IoError(embedded_io::ErrorKind),
    /// The byte stream contained fewer bytes than expected.
    ///
    /// Corresponds to NRC 0x13 (`incorrectMessageLengthOrInvalidFormat`).
    #[error("Insufficient data: {0}")]
    InsufficientData(Incomplete),
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
    /// Bytes remained after a decode that should have consumed the whole buffer.
    ///
    /// Corresponds to NRC 0x13 (`incorrectMessageLengthOrInvalidFormat`), like
    /// [`Error::IncorrectMessageLengthOrInvalidFormat`].
    #[error("{0}")]
    TrailingBytes(TrailingBytes),
    /// A wire-declared variable-width field requested a byte width the target
    /// type cannot hold.
    ///
    /// Corresponds to NRC 0x13 (`incorrectMessageLengthOrInvalidFormat`).
    #[error("{0}")]
    InvalidWidth(InvalidWidth),
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

impl From<Incomplete> for Error {
    fn from(frag: Incomplete) -> Self {
        Self::InsufficientData(frag)
    }
}

impl From<TrailingBytes> for Error {
    fn from(frag: TrailingBytes) -> Self {
        Self::TrailingBytes(frag)
    }
}

impl From<InvalidWidth> for Error {
    fn from(frag: InvalidWidth) -> Self {
        Self::InvalidWidth(frag)
    }
}

impl From<ReadUintError> for Error {
    fn from(err: ReadUintError) -> Self {
        match err {
            ReadUintError::Incomplete(i) => Self::InsufficientData(i),
            ReadUintError::InvalidWidth(w) => Self::InvalidWidth(w),
        }
    }
}

impl From<WriteUintError> for Error {
    fn from(err: WriteUintError) -> Self {
        match err {
            WriteUintError::Io(kind) => Self::IoError(kind),
            WriteUintError::InvalidWidth(w) => Self::InvalidWidth(w),
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.kind().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use automotive_wire_codec::{Incomplete, TrailingBytes};

    #[test]
    fn incomplete_lifts_into_error_losslessly() {
        let frag = Incomplete {
            needed: 4,
            available: 1,
        };
        let err = Error::from(frag);
        assert!(matches!(err, Error::InsufficientData(i) if i == frag));
    }

    #[test]
    fn trailing_bytes_lifts_into_error() {
        let err = Error::from(TrailingBytes(3));
        assert!(matches!(err, Error::TrailingBytes(TrailingBytes(3))));
    }

    #[test]
    fn read_uint_error_arms_map_losslessly() {
        use automotive_wire_codec::{Incomplete, InvalidWidth, ReadUintError};
        let inc = ReadUintError::Incomplete(Incomplete {
            needed: 4,
            available: 1,
        });
        assert!(
            matches!(Error::from(inc), Error::InsufficientData(i) if i.needed == 4 && i.available == 1)
        );
        let iw = ReadUintError::InvalidWidth(InvalidWidth { max: 4, got: 5 });
        assert!(matches!(Error::from(iw), Error::InvalidWidth(w) if w.max == 4 && w.got == 5));
    }

    #[test]
    fn write_uint_error_arms_map_losslessly() {
        use automotive_wire_codec::{InvalidWidth, WriteUintError};
        let io = WriteUintError::Io(embedded_io::ErrorKind::WriteZero);
        assert!(matches!(
            Error::from(io),
            Error::IoError(embedded_io::ErrorKind::WriteZero)
        ));
        let iw = WriteUintError::InvalidWidth(InvalidWidth { max: 16, got: 17 });
        assert!(matches!(Error::from(iw), Error::InvalidWidth(w) if w.got == 17));
    }
}
