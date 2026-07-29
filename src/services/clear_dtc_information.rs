//! `ClearDiagnosticInformation` (0x14) service implementation
use crate::{CLEAR_ALL_DTCS, Decode, DtcRecord, Encode, NegativeResponseCode};
use automotive_wire_codec::write_u8;

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
    type Error = crate::Error;
    fn encode(&self, _writer: &mut impl embedded_io::Write) -> Result<usize, crate::Error> {
        Ok(0)
    }
}

impl<'a> Decode<'a> for ClearDiagnosticInfoResponse {
    type Error = crate::Error;

    /// Consumes zero bytes and returns the full buffer as the remainder.
    /// `decode_exact` at the call site (in `Response::decode`) enforces that no trailing bytes follow the SID.
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
    pub group_of_dtc: DtcRecord,
    /// Addresses a user-defined DTC memory, when the client is targeting one.
    ///
    /// `None` is the ordinary case and the only form in ISO 14229-1:2013: the parameter is
    /// marked `U` (user option) in ISO 14229-1:2020 Table 296, so it is absent from the wire
    /// unless the client is addressing user-defined DTC memory.
    pub memory_selection: Option<u8>,
}

impl ClearDiagnosticInfoRequest {
    /// Create a request to clear a specific DTC group, without addressing a user-defined
    /// DTC memory.
    #[must_use]
    pub const fn new(group_of_dtc: DtcRecord) -> Self {
        Self {
            group_of_dtc,
            memory_selection: None,
        }
    }

    /// Create a request to clear a specific DTC group from a user-defined DTC memory.
    #[must_use]
    pub const fn new_with_memory_selection(group_of_dtc: DtcRecord, memory_selection: u8) -> Self {
        Self {
            group_of_dtc,
            memory_selection: Some(memory_selection),
        }
    }

    /// Create a request to clear all DTCs, without addressing a user-defined DTC memory.
    #[must_use]
    pub const fn clear_all() -> Self {
        Self::new(CLEAR_ALL_DTCS)
    }

    /// Create a request to clear all DTCs from a user-defined DTC memory.
    #[must_use]
    pub const fn clear_all_in_memory(memory_selection: u8) -> Self {
        Self::new_with_memory_selection(CLEAR_ALL_DTCS, memory_selection)
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &CLEAR_DIAG_INFO_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for ClearDiagnosticInfoRequest {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, crate::Error> {
        let mut written = Encode::encode(&self.group_of_dtc, writer)?;
        if let Some(memory_selection) = self.memory_selection {
            written += write_u8(writer, memory_selection).map_err(crate::Error::io)?;
        }
        Ok(written)
    }
}

impl<'a> Decode<'a> for ClearDiagnosticInfoRequest {
    type Error = crate::Error;

    /// The 3 `groupOfDTC` bytes are mandatory; a 4th byte, if present, is the optional
    /// `MemorySelection` (ISO 14229-1:2020 Table 296, `Cvt` = `U`).
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), crate::Error> {
        let (group_of_dtc, rest) = <DtcRecord as Decode>::decode(buf)?;
        let (memory_selection, rest) = match rest {
            [] => (None, rest),
            [selection, tail @ ..] => (Some(*selection), tail),
        };
        Ok((
            Self {
                group_of_dtc,
                memory_selection,
            },
            rest,
        ))
    }
}

/// test
#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, Encode, Incomplete, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn decode_clear_dtc_info_request() {
        let bytes = [0xFF, 0xFF, 0xFF, 0x00];
        let compare = ClearDiagnosticInfoRequest::clear_all_in_memory(0);
        let (req, _) = <ClearDiagnosticInfoRequest as Decode>::decode(&bytes).unwrap();
        assert_eq!(req, compare);

        let mut buf = vec![];
        let written = Encode::encode(&req, &mut buf).unwrap();
        assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0x00]);
        assert_eq!(req.encoded_size().unwrap(), written);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn clear_all() {
        let all = ClearDiagnosticInfoRequest::clear_all();
        let compare = ClearDiagnosticInfoRequest::new(CLEAR_ALL_DTCS);
        assert_eq!(all, compare);
    }

    #[test]
    fn three_byte_request_decodes_without_a_memory_selection() {
        // ISO 14229-1:2020 Table 296 marks MemorySelection `U` (user option), and the
        // Table 300 flow example is exactly this 3-byte form.
        let (req, rest) =
            <ClearDiagnosticInfoRequest as Decode>::decode(&[0xFF, 0xFF, 0xFF]).unwrap();
        assert_eq!(req.memory_selection, None);
        assert_eq!(req.group_of_dtc, CLEAR_ALL_DTCS);
        assert!(rest.is_empty());
    }

    #[test]
    fn a_request_without_a_memory_selection_encodes_three_bytes() {
        let req = ClearDiagnosticInfoRequest::clear_all();
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0xFF, 0xFF, 0xFF]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn four_byte_request_decodes_the_memory_selection() {
        let (req, rest) =
            <ClearDiagnosticInfoRequest as Decode>::decode(&[0x01, 0x02, 0x03, 0x2A]).unwrap();
        assert_eq!(req.memory_selection, Some(0x2A));
        assert_eq!(req.group_of_dtc, DtcRecord::from(0x01_0203));
        assert!(rest.is_empty());
    }

    #[test]
    fn a_request_with_a_memory_selection_encodes_four_bytes() {
        let req =
            ClearDiagnosticInfoRequest::new_with_memory_selection(DtcRecord::from(0x01_0203), 0x2A);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x01, 0x02, 0x03, 0x2A]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn clear_all_in_memory_targets_a_user_defined_memory() {
        let req = ClearDiagnosticInfoRequest::clear_all_in_memory(0x02);
        assert_eq!(req.group_of_dtc, CLEAR_ALL_DTCS);
        assert_eq!(req.memory_selection, Some(0x02));
    }

    #[test]
    fn a_truncated_group_of_dtc_is_still_rejected() {
        // The 3 groupOfDTC bytes stay mandatory; only the 4th byte is optional.
        let got = <ClearDiagnosticInfoRequest as Decode>::decode(&[0xFF, 0xFF]);
        assert!(
            matches!(
                got,
                Err(crate::Error::InsufficientData(Incomplete {
                    needed: 3,
                    available: 2
                }))
            ),
            "expected a 3-byte shortfall, got {got:?}"
        );
    }

    #[test]
    fn both_wire_forms_round_trip_through_the_request_frame() {
        for wire in [
            [0x14, 0xFF, 0xFF, 0xFF].as_slice(),
            [0x14, 0x01, 0x02, 0x03, 0x2A].as_slice(),
        ] {
            let (req, _) = crate::Request::decode(wire).unwrap();
            let mut buf = [0u8; 8];
            let written = req.encode_to_slice(&mut buf).unwrap();
            assert_eq!(&buf[..written], wire, "round trip failed for {wire:02X?}");
        }
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
        let (decoded, remaining) =
            <ClearDiagnosticInfoResponse as Decode>::decode(&buf[..0]).unwrap();
        assert_eq!(decoded, resp);
        assert!(remaining.is_empty());
    }
}
