use crate::{CLEAR_ALL_DTCS, DTCRecord, NegativeResponseCode, SingleValueWireFormat, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};

/// Negative response codes
const CLEAR_DIAG_INFO_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::GeneralProgrammingFailure,
];

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClearDiagnosticInfoRequest {
    /// Can be either a DTC group (such as chassis/powertrain) or a single DTC
    pub group_of_dtc: DTCRecord,
    /// Used to address a specific memory location of user-defined DTC memory
    pub memory_selection: u8,
}

impl ClearDiagnosticInfoRequest {
    #[must_use]
    pub fn new(group_of_dtc: DTCRecord, memory_selection: u8) -> Self {
        Self {
            group_of_dtc,
            memory_selection,
        }
    }

    #[must_use]
    pub fn clear_all(memory_selection: u8) -> Self {
        Self {
            group_of_dtc: CLEAR_ALL_DTCS,
            memory_selection,
        }
    }

    /// Get the allowed Nack codes for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &CLEAR_DIAG_INFO_NEGATIVE_RESPONSE_CODES
    }
}

impl WireFormat for ClearDiagnosticInfoRequest {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, crate::Error> {
        let group_of_dtc = DTCRecord::option_from_reader(reader)?;
        if group_of_dtc.is_none() {
            return Ok(None);
        }
        let memory_selection = reader.read_u8()?;

        Ok(Some(Self {
            group_of_dtc: group_of_dtc.unwrap(),
            memory_selection,
        }))
    }

    fn required_size(&self) -> usize {
        self.group_of_dtc.required_size() + 1
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, crate::Error> {
        let mut size = 0;
        size += self.group_of_dtc.encode(writer)?;
        writer.write_u8(self.memory_selection)?;
        size += 1;
        Ok(size)
    }
}

impl SingleValueWireFormat for ClearDiagnosticInfoRequest {}

/// test
#[cfg(test)]
mod request {
    use super::*;

    #[test]
    fn decode_clear_dtc_info_request() {
        let bytes = [0xFF, 0xFF, 0xFF, 0x00];
        let compare = ClearDiagnosticInfoRequest::new(CLEAR_ALL_DTCS, 0);
        let req = ClearDiagnosticInfoRequest::from_reader(&mut &bytes[..]).unwrap();
        assert_eq!(req, compare);

        let mut bytes = vec![];
        let written = req.encode(&mut bytes).unwrap();
        assert_eq!(bytes, [0xFF, 0xFF, 0xFF, 0x00]);
        assert_eq!(req.required_size(), written);
    }

    #[test]
    fn clear_all() {
        let all = ClearDiagnosticInfoRequest::clear_all(0);
        let compare = ClearDiagnosticInfoRequest::new(CLEAR_ALL_DTCS, 0);
        assert_eq!(all, compare);
    }
}
