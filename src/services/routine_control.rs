//! Routine Control (0x31) Service is used to perform functions on the ECU that may not be covered by other services.
//!
//! It can also be used to check the ECU's health, erase memory, or other custom manufacturer/supplier routines.
//! However, some routines may have side effects or require certain preconditions to be met.
use crate::{Encode, Error, RoutineControlSubFunction};

/// Used by a client to execute a defined sequence of events and obtain any relevant results.
///
/// The payload is the routine identifier (2 bytes, big-endian) followed by any optional
/// routine input parameters, exactly as it appears on the wire after the sub-function byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequestTx<'d> {
    /// The routine control operation (start, stop, or request results).
    pub sub_function: RoutineControlSubFunction,
    /// The raw payload bytes: routine identifier followed by optional parameters.
    pub raw_payload: &'d [u8],
}

impl<'d> RoutineControlRequestTx<'d> {
    /// Create a new `RoutineControlRequestTx`.
    #[must_use]
    pub const fn new(sub_function: RoutineControlSubFunction, raw_payload: &'d [u8]) -> Self {
        Self {
            sub_function,
            raw_payload,
        }
    }
}

impl Encode for RoutineControlRequestTx<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.raw_payload.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.sub_function)])
            .map_err(Error::io)?;
        writer.write_all(self.raw_payload).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

/// `RoutineControlResponseTx` is a variable-length response that can contain routine status.
///
/// The status record is the routine identifier echo plus any routine-info / status bytes,
/// held as raw bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponseTx<'d> {
    /// The sub-function echoed from the routine control request.
    pub routine_control_type: RoutineControlSubFunction,
    /// Raw routine status record bytes (routine identifier + routine info + status).
    pub raw_status_record: &'d [u8],
}

impl<'d> RoutineControlResponseTx<'d> {
    /// Create a new `RoutineControlResponseTx`.
    #[must_use]
    pub const fn new(
        routine_control_type: RoutineControlSubFunction,
        raw_status_record: &'d [u8],
    ) -> Self {
        Self {
            routine_control_type,
            raw_status_record,
        }
    }
}

impl Encode for RoutineControlResponseTx<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.raw_status_record.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.routine_control_type)])
            .map_err(Error::io)?;
        writer
            .write_all(self.raw_status_record)
            .map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn encode_routine_control_request_tx() {
        // RID 0xFF00 (EraseMemory) + 1 parameter byte
        let payload = [0xFF, 0x00, 0xAA];
        let req = RoutineControlRequestTx::new(RoutineControlSubFunction::StartRoutine, &payload);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x01, 0xFF, 0x00, 0xAA]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_routine_control_response_tx() {
        let record = [0xFF, 0x00, 0x10];
        let resp =
            RoutineControlResponseTx::new(RoutineControlSubFunction::StartRoutine, &record);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x01, 0xFF, 0x00, 0x10]);
        assert_encode_size_agrees(&resp);
    }
}
