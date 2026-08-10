use automotive_wire_codec::{
    Incomplete, InvalidWidth, ReadUintError, TrailingBytes, WriteUintError,
};
use thiserror::Error;

use crate::NegativeResponseCode;

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
    /// The memory size does not fit the width the `addressAndLengthFormatIdentifier` declares
    /// for it, so encoding it would truncate the value on the wire.
    ///
    /// Tables 444 and 449 put a `memorySize` that "is not valid" on NRC 0x31, like
    /// [`Error::InvalidMemoryAddress`].
    #[error("Invalid Memory Size: {0}")]
    InvalidMemorySize(u32),
    /// The `addressAndLengthFormatIdentifier` byte declares a width ISO does not permit.
    ///
    /// ISO 14229-1:2020 Annex H Table H.1 marks the high nibble (`memorySize`) applicable for
    /// 1 to 4 bytes and the low nibble (`memoryAddress`) for 1 to 5; a zero nibble or anything
    /// wider is "not applicable". Tables 444 and 449 both put this on NRC 0x31, not 0x13.
    #[error("Invalid addressAndLengthFormatIdentifier: {0:#04X}")]
    InvalidAddressAndLengthFormatIdentifier(u8),
    /// A `u32` did not fit the three bytes of a DTC (i.e. exceeded `0x00FF_FFFF`).
    #[error("Invalid DTC record: {0:#010X} does not fit three bytes")]
    InvalidDtcRecord(u32),
    /// The encryption or compression method byte is not recognised.
    #[error("Invalid Encryption/Compression Method: {0}")]
    InvalidEncryptionCompressionMethod(u8),
    /// The `RequestFileTransfer` `modeOfOperation` byte is not valid.
    #[error("Invalid FileTransfer modeOfOperation (server will send requestOutOfRange): {0}")]
    InvalidFileOperationMode(u8),
    /// The `ReadDTCInformation` sub-function byte is not valid.
    #[error("Invalid DTC Subfunction Type: {0}")]
    InvalidDtcSubfunctionType(u8),
    /// The routine-control sub-function byte is not a valid [`RoutineControlSubFunction`](crate::RoutineControlSubFunction).
    #[error("Invalid Routine Control Sub-Function: {0}")]
    InvalidRoutineControlSubFunction(u8),
    /// The DTC-setting byte is not a valid [`DtcSettingType`](crate::DtcSettingType) value.
    #[error("Invalid DTC Setting: {0}")]
    InvalidDtcSetting(u8),
}

impl Error {
    /// Convert any `embedded_io::Error` into [`Error::IoError`].
    #[inline]
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn io<E: embedded_io::Error>(e: E) -> Self {
        Self::IoError(e.kind())
    }

    /// The [`NegativeResponseCode`] a server should return for this error.
    ///
    /// A server decodes an inbound request, and on failure has to answer with a negative
    /// response. This is that mapping, so callers do not have to re-derive it against a
    /// `#[non_exhaustive]` enum they cannot match exhaustively:
    ///
    /// ```
    /// use uds_protocol::{Decode, NegativeResponse, Request, UdsServiceType};
    ///
    /// let frame = [0x11, 0x01, 0xAA]; // EcuReset with a trailing junk byte
    /// let err = Request::decode(&frame).expect_err("the trailing byte must be rejected");
    /// let nrc = err.negative_response_code().expect("a malformed frame maps to an NRC");
    /// let nack = NegativeResponse::new(UdsServiceType::EcuReset, nrc);
    /// assert_eq!(u8::from(nack.nrc()), 0x13);
    /// ```
    ///
    /// Returns `None` for [`Error::IoError`]: a transport failure is not a protocol error, so
    /// there is no NRC to send. Every other error maps to a code.
    ///
    /// # Classification
    ///
    /// This is a **default**, covering the lane ISO 14229-1 actually mandates — clause 8.7.5's
    /// pseudo-code and the "shall" rows of Annex A.1. Clause 8.7.2 notes that "a specific NRC
    /// is not guaranteed for all possible test pattern sequences", and several outcomes in
    /// Tables 4 to 7 are specified only as "NRC = XX", so a server is free to answer
    /// differently where its own tables allow:
    ///
    /// - **`0x13` `incorrectMessageLengthOrInvalidFormat`** — the frame itself is malformed:
    ///   too short, too long, or a declared width that does not fit.
    /// - **`0x12` `subFunctionNotSupported`** — the *sub-function* byte is not a value this
    ///   service defines.
    ///
    ///   Only two services reject a reserved sub-function at decode and so produce this code
    ///   themselves: `0x31` `RoutineControl` (Table 426 defines no manufacturer range, so
    ///   anything outside `0x01`-`0x03` is invalid) and `0x85` `ControlDTCSetting`.
    ///
    ///   The others — `0x10`, `0x11`, `0x19`, `0x27`, `0x28`, `0x3E` — model the whole
    ///   `0x00..=0x7F` space, so a reserved byte decodes into an `IsoSaeReserved`-style variant
    ///   and round-trips unchanged rather than failing. That is deliberate: it lets a server see
    ///   the byte it was actually sent. **Answering `0x12` for those is the server's job**, not
    ///   this mapping's: match on the sub-function and use
    ///   [`NegativeResponseCode::SubFunctionNotSupported`] for a value you do not implement.
    /// - **`0x31` `requestOutOfRange`** — a *parameter* (not a sub-function) carries a value
    ///   outside its permitted range. Note that `communicationType` is a parameter of
    ///   `CommunicationControl`, not its sub-function, so it lands here rather than on `0x12`.
    ///
    /// Two codes this mapping deliberately never returns:
    ///
    /// - **`0x10` `generalReject`** — Annex A.1 says it "shall only be implemented in the server
    ///   if none of the negative response codes defined in this document meet the needs of the
    ///   implementation. At no means shall this NRC be a general replacement". It also appears in
    ///   none of the per-service NRC tables, so no service's `allowed_nack_codes` lists it.
    /// - **`0x11` `serviceNotSupported`** — an unrecognised SID does not fail to decode; it
    ///   becomes [`Request::Other`](crate::Request::Other) so a server can see the byte. Answering
    ///   `0x11` is therefore the server's call, on a SID it does not implement.
    ///
    /// Every code this does return is a legal NRC to put on the wire: never
    /// [`NegativeResponseCode::PositiveResponse`] and never a reserved value.
    #[must_use]
    pub const fn negative_response_code(&self) -> Option<NegativeResponseCode> {
        match self {
            // The frame is malformed.
            Self::InsufficientData(_)
            | Self::TrailingBytes(_)
            | Self::InvalidWidth(_)
            | Self::IncorrectMessageLengthOrInvalidFormat => {
                Some(NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat)
            }

            // The sub-function byte is not a defined value for the service.
            Self::InvalidDiagnosticSessionType(_)
            | Self::InvalidEcuResetType(_)
            | Self::InvalidSecurityAccessType(_)
            | Self::InvalidCommunicationControlType(_)
            | Self::InvalidTesterPresentType(_)
            | Self::InvalidRoutineControlSubFunction(_)
            | Self::InvalidDtcSubfunctionType(_)
            | Self::InvalidDtcSetting(_) => Some(NegativeResponseCode::SubFunctionNotSupported),

            // A parameter value is outside its permitted range.
            Self::InvalidCommunicationType(_)
            | Self::InvalidMemoryAddress(_)
            | Self::InvalidMemorySize(_)
            | Self::InvalidAddressAndLengthFormatIdentifier(_)
            | Self::InvalidDtcRecord(_)
            | Self::InvalidEncryptionCompressionMethod(_)
            | Self::InvalidFileOperationMode(_) => Some(NegativeResponseCode::RequestOutOfRange),

            // Transport failure. ISO does not model this as an NRC at all: clause 7.4.1.6
            // surfaces it to the server application as `A_Result = error`. There is no byte to
            // put on the wire, because the wire is what failed.
            Self::IoError(_) => None,
        }
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
mod nrc_mapping_tests {
    use super::*;
    use crate::NegativeResponseCode;

    /// Every variant, with the NRC byte a server should send back. Grouped by the ISO 14229-1
    /// classification the mapping implements: 0x13 for length/format, 0x12 for an unsupported
    /// sub-function byte, 0x31 for an in-range-but-unsupported parameter value.
    fn cases() -> impl Iterator<Item = (Error, u8, &'static str)> {
        [
            // --- 0x13 incorrectMessageLengthOrInvalidFormat: the frame is malformed ---
            (
                Error::InsufficientData(Incomplete {
                    needed: 4,
                    available: 1,
                }),
                0x13,
                "short read",
            ),
            (Error::TrailingBytes(TrailingBytes(2)), 0x13, "extra bytes"),
            (
                Error::InvalidWidth(InvalidWidth { max: 4, got: 9 }),
                0x13,
                "declared width too wide",
            ),
            (
                Error::IncorrectMessageLengthOrInvalidFormat,
                0x13,
                "explicit length/format",
            ),
            // --- 0x12 subFunctionNotSupported: the sub-function byte is not a known value ---
            (
                Error::InvalidDiagnosticSessionType(0x99),
                0x12,
                "0x10 sub-function",
            ),
            (Error::InvalidEcuResetType(0x99), 0x12, "0x11 sub-function"),
            (
                Error::InvalidSecurityAccessType(0x99),
                0x12,
                "0x27 sub-function",
            ),
            (
                Error::InvalidCommunicationControlType(0x99),
                0x12,
                "0x28 sub-function",
            ),
            (
                Error::InvalidTesterPresentType(0x99),
                0x12,
                "0x3E sub-function",
            ),
            (
                Error::InvalidRoutineControlSubFunction(0x99),
                0x12,
                "0x31 sub-function",
            ),
            (
                Error::InvalidDtcSubfunctionType(0x99),
                0x12,
                "0x19 sub-function",
            ),
            (Error::InvalidDtcSetting(0x99), 0x12, "0x85 sub-function"),
            // --- 0x31 requestOutOfRange: a parameter value, not a sub-function ---
            (
                Error::InvalidCommunicationType(0x99),
                0x31,
                "0x28 communicationType parameter",
            ),
            (
                Error::InvalidMemoryAddress(0x1_0000_0000_0000),
                0x31,
                "address beyond 5 bytes",
            ),
            (
                Error::InvalidEncryptionCompressionMethod(0x99),
                0x31,
                "nibble overflow",
            ),
            (
                Error::InvalidFileOperationMode(0x99),
                0x31,
                "0x38 modeOfOperation",
            ),
            (
                Error::InvalidDtcRecord(0xFF01_0203),
                0x31,
                "a u32 wider than three DTC bytes",
            ),
            (
                Error::InvalidAddressAndLengthFormatIdentifier(0x00),
                0x31,
                "addressAndLengthFormatIdentifier outside Table H.1",
            ),
            (
                Error::InvalidMemorySize(0x1_0000),
                0x31,
                "memorySize wider than its declared width",
            ),
        ]
        .into_iter()
    }

    #[test]
    fn every_error_maps_to_its_iso_negative_response_code() {
        for (err, want, why) in cases() {
            let nrc = err
                .negative_response_code()
                .expect("every case in this table is a protocol error, not a transport one");
            let got = u8::from(nrc);
            assert_eq!(
                got, want,
                "{err:?} ({why}): got 0x{got:02X}, want 0x{want:02X}"
            );
        }
    }

    #[test]
    fn mapping_never_produces_a_positive_or_reserved_code() {
        // This is deliberately narrower than "the codes the ISO tables allow": the per-service
        // tables are a floor, not a ceiling (clause 9.4 — the A.1 codes "shall be used in
        // addition to" them), so table membership is not the property to assert. What must hold
        // is that a decode failure never reports a positive response and never invents a
        // reserved code, either of which would put an illegal byte on the wire.
        for (err, _, why) in cases() {
            let nrc = err
                .negative_response_code()
                .expect("every case in this table maps to a code");
            assert_ne!(
                nrc,
                NegativeResponseCode::PositiveResponse,
                "{why}: decode failure mapped to PositiveResponse"
            );
            assert!(
                !matches!(
                    nrc,
                    NegativeResponseCode::IsoSaeReserved(_)
                        | NegativeResponseCode::ExtendedDataLinkSecurityReserved(_)
                        | NegativeResponseCode::ReservedForSpecificConditionsNotMet(_)
                ),
                "{why}: mapped to a reserved NRC ({nrc:?})"
            );
        }
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
