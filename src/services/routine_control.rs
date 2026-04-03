//! Routine Control (0x31) Service is used to perform functions on the ECU that are may not be covered by other services.
//!
//! It can also be used to check the ECU's health, erase memory, or other custom manufacturer/supplier routines.
//! However, some routines may have side effects or require certain preconditions to be met.
use crate::{
    Encode, Error, Identifier, RoutineControlSubFunction,
};

/// Used by a client to execute a defined sequence of events and obtain any relevant results
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequest<RoutineIdentifier, RoutinePayload> {
    /// The routine control operation (start, stop, or request results).
    pub sub_function: RoutineControlSubFunction,
    /// The identifier of the routine to control.
    pub routine_id: RoutineIdentifier,
    /// Optional payload data for the routine (e.g. input parameters).
    pub data: Option<RoutinePayload>,
}

impl<RI: Identifier, RP: Encode> RoutineControlRequest<RI, RP> {
    pub(crate) fn new(
        sub_function: RoutineControlSubFunction,
        routine_id: RI,
        data: Option<RP>,
    ) -> Self {
        Self {
            sub_function,
            routine_id,
            data,
        }
    }
}

impl<RI: Identifier, RP: Encode> Encode for RoutineControlRequest<RI, RP> {
    fn encoded_size(&self) -> usize {
        1 + 2 + self.data.as_ref().map_or(0, Encode::encoded_size)
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.sub_function)])
            .map_err(Error::io)?;
        Encode::encode(&self.routine_id, writer)?;
        if let Some(payload) = &self.data {
            Encode::encode(payload, writer)?;
        }
        Ok(self.encoded_size())
    }
}

/// `RoutineControlResponse` is a variable length field that can contain the status of the routine
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponse<RoutineInfoStatusRecord> {
    /// The sub-function echoes the routine control request
    pub routine_control_type: RoutineControlSubFunction,

    /// Should contain the `routine_info` (u8) and the `routine_status_record` (u8 * n) information. n can be 0
    pub routine_status_record: RoutineInfoStatusRecord,
}

impl<RSR: Encode> RoutineControlResponse<RSR> {
    pub(crate) fn new(
        routine_control_type: RoutineControlSubFunction,
        routine_status_record: RSR,
    ) -> Self {
        Self {
            routine_control_type,
            routine_status_record,
        }
    }
}

impl<RSR: Encode> Encode for RoutineControlResponse<RSR> {
    fn encoded_size(&self) -> usize {
        1 + self.routine_status_record.encoded_size()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.routine_control_type)])
            .map_err(Error::io)?;
        Encode::encode(&self.routine_status_record, writer)?;
        Ok(self.encoded_size())
    }
}
