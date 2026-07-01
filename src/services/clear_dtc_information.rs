//! `ClearDiagnosticInformation` (0x14) service implementation
use crate::{CLEAR_ALL_DTCS, DTCRecord, Decode, Encode, NegativeResponseCode};

/// Positive response to `ClearDiagnosticInformation`. Carries no payload.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct ClearDiagnosticInfoResponse;

impl ClearDiagnosticInfoResponse {
    /// Create a `ClearDiagnosticInfoResponse`.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Encode for ClearDiagnosticInfoResponse {
    fn encoded_size(&self) -> usize {
        0
    }
    fn encode(&self, _writer: &mut impl embedded_io::Write) -> Result<usize, crate::Error> {
        Ok(0)
    }
}

impl<'a> Decode<'a> for ClearDiagnosticInfoResponse {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), crate::Error> {
        Ok((Self, buf))
    }
}

/// Negative response codes
const CLEAR_DIAG_INFO_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::GeneralProgrammingFailure,
];

/// Request for the server to clear diagnostic information
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ClearDiagnosticInfoRequest {
    /// Can be either a DTC group (such as chassis/powertrain) or a single DTC
    pub group_of_dtc: DTCRecord,
    /// Used to address a specific memory location of user-defined DTC memory
    pub memory_selection: u8,
}

impl ClearDiagnosticInfoRequest {
    /// Create a request to clear a specific DTC group from the given memory location.
    #[must_use]
    pub const fn new(group_of_dtc: DTCRecord, memory_selection: u8) -> Self {
        Self {
            group_of_dtc,
            memory_selection,
        }
    }

    /// Create a request to clear all DTCs from the given memory location.
    #[must_use]
    pub const fn clear_all(memory_selection: u8) -> Self {
        Self {
            group_of_dtc: CLEAR_ALL_DTCS,
            memory_selection,
        }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &CLEAR_DIAG_INFO_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for ClearDiagnosticInfoRequest {
    fn encoded_size(&self) -> usize {
        4 // DTCRecord (3) + memory_selection (1)
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, crate::Error> {
        let size = Encode::encode(&self.group_of_dtc, writer)?;
        writer
            .write_all(&[self.memory_selection])
            .map_err(crate::Error::io)?;
        Ok(size + 1)
    }
}

impl<'a> Decode<'a> for ClearDiagnosticInfoRequest {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), crate::Error> {
        let (group_of_dtc, buf) = <DTCRecord as Decode>::decode(buf)?;
        if buf.is_empty() {
            return Err(crate::Error::InsufficientData(4));
        }
        let memory_selection = buf[0];
        Ok((
            Self {
                group_of_dtc,
                memory_selection,
            },
            &buf[1..],
        ))
    }
}

/// test
#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn decode_clear_dtc_info_request() {
        let bytes = [0xFF, 0xFF, 0xFF, 0x00];
        let compare = ClearDiagnosticInfoRequest::new(CLEAR_ALL_DTCS, 0);
        let (req, _) = <ClearDiagnosticInfoRequest as Decode>::decode(&bytes).unwrap();
        assert_eq!(req, compare);

        let mut buf = vec![];
        let written = Encode::encode(&req, &mut buf).unwrap();
        assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0x00]);
        assert_eq!(req.encoded_size(), written);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn clear_all() {
        let all = ClearDiagnosticInfoRequest::clear_all(0);
        let compare = ClearDiagnosticInfoRequest::new(CLEAR_ALL_DTCS, 0);
        assert_eq!(all, compare);
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode};

    #[test]
    fn clear_dtc_response_roundtrips_empty() {
        let resp = ClearDiagnosticInfoResponse::new();
        let mut buf = [0u8; 4];
        let n = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(n, 0);
        let (decoded, rest) = <ClearDiagnosticInfoResponse as Decode>::decode(&buf[..0]).unwrap();
        assert_eq!(decoded, resp);
        assert!(rest.is_empty());
    }
}
