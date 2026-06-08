//! Routine Control (0x31) Service is used to perform functions on the ECU that may not be covered by other services.
//!
//! It can also be used to check the ECU's health, erase memory, or other custom manufacturer/supplier routines.
//! However, some routines may have side effects or require certain preconditions to be met.
use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error};

/// What type of routine control to perform for a [`RoutineControlRequest`].
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RoutineControlSubFunction {
    /// Routine will be started sometime between completion of the `StartRoutine` request and the completion of the 1st response message
    /// which indicates that the routine has already been performed, or is in progress
    ///
    /// It might be necessary to switch the server to a specific Diagnostic Session via [`crate::DiagnosticSessionControlRequest`] before starting the routine,
    /// or unlock the server using [`crate::SecurityAccessRequest`] before starting the routine.
    StartRoutine,

    /// The server routine shall be stopped in the server's memory sometime between the completion of the `StopRoutine` request and the completion of the 1st response message
    /// which indicates that the routine has already been stopped, or is in progress
    StopRoutine,

    /// Request results for the specified routineIdentifier
    RequestRoutineResults,
}

impl From<RoutineControlSubFunction> for u8 {
    fn from(value: RoutineControlSubFunction) -> Self {
        match value {
            RoutineControlSubFunction::StartRoutine => 0x01,
            RoutineControlSubFunction::StopRoutine => 0x02,
            RoutineControlSubFunction::RequestRoutineResults => 0x03,
        }
    }
}

impl TryFrom<u8> for RoutineControlSubFunction {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(RoutineControlSubFunction::StartRoutine),
            0x02 => Ok(RoutineControlSubFunction::StopRoutine),
            0x03 => Ok(RoutineControlSubFunction::RequestRoutineResults),
            _ => Err(Error::IncorrectMessageLengthOrInvalidFormat),
        }
    }
}

/// Used by a client to execute a defined sequence of events and obtain any relevant results.
///
/// The payload is the routine identifier (2 bytes, big-endian) followed by any optional
/// routine input parameters, exactly as it appears on the wire after the sub-function byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequest<'d> {
    sub_function: SuppressablePositiveResponse<RoutineControlSubFunction>,
    raw_payload: &'d [u8],
}

impl<'d> RoutineControlRequest<'d> {
    /// Create a new `RoutineControlRequest`.
    #[must_use]
    pub const fn new(
        suppress_positive_response: bool,
        sub_function: RoutineControlSubFunction,
        raw_payload: &'d [u8],
    ) -> Self {
        Self {
            sub_function: SuppressablePositiveResponse::new(
                suppress_positive_response,
                sub_function,
            ),
            raw_payload,
        }
    }

    /// Whether the server should suppress the positive response (SPRMIB).
    #[must_use]
    pub fn suppress_positive_response(&self) -> bool {
        self.sub_function.suppress_positive_response()
    }

    /// The routine control operation (start, stop, or request results).
    #[must_use]
    pub fn sub_function(&self) -> RoutineControlSubFunction {
        self.sub_function.value()
    }

    /// The raw payload bytes: routine identifier followed by optional parameters.
    #[must_use]
    pub const fn raw_payload(&self) -> &[u8] {
        self.raw_payload
    }
}

impl Encode for RoutineControlRequest<'_> {
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

impl<'a> Decode<'a> for RoutineControlRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let sub_function = SuppressablePositiveResponse::try_from(buf[0])?;
        Ok((
            Self {
                sub_function,
                raw_payload: &buf[1..],
            },
            &[],
        ))
    }
}

/// `RoutineControlResponse` is a variable-length response that can contain routine status.
///
/// The status record is the routine identifier echo plus any routine-info / status bytes,
/// held as raw bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponse<'d> {
    /// The sub-function echoed from the routine control request.
    pub routine_control_type: RoutineControlSubFunction,
    /// Raw routine status record bytes (routine identifier + routine info + status).
    pub raw_status_record: &'d [u8],
}

impl<'d> RoutineControlResponse<'d> {
    /// Create a new `RoutineControlResponse`.
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

impl Encode for RoutineControlResponse<'_> {
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

impl<'a> Decode<'a> for RoutineControlResponse<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let routine_control_type = RoutineControlSubFunction::try_from(buf[0])?;
        Ok((
            Self {
                routine_control_type,
                raw_status_record: &buf[1..],
            },
            &[],
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Decode;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn encode_routine_control_request_tx() {
        // RID 0xFF00 (EraseMemory) + 1 parameter byte
        let payload = [0xFF, 0x00, 0xAA];
        let req =
            RoutineControlRequest::new(false, RoutineControlSubFunction::StartRoutine, &payload);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x01, 0xFF, 0x00, 0xAA]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_routine_control_response_tx() {
        let record = [0xFF, 0x00, 0x10];
        let resp = RoutineControlResponse::new(RoutineControlSubFunction::StartRoutine, &record);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x01, 0xFF, 0x00, 0x10]);
        assert_encode_size_agrees(&resp);
    }

    #[test]
    fn decode_routine_control_request_with_suppress_bit() {
        // sub 0x81 = StartRoutine (0x01) + SPRMIB (0x80), then RID 0xFF00 + param 0xAA
        let bytes = [0x81, 0xFF, 0x00, 0xAA];
        let (req, rest) = <RoutineControlRequest as Decode>::decode(&bytes).unwrap();
        assert!(rest.is_empty());
        assert!(req.suppress_positive_response());
        assert_eq!(req.sub_function(), RoutineControlSubFunction::StartRoutine);
        assert_eq!(req.raw_payload(), &[0xFF, 0x00, 0xAA]);
        // round-trips back to the same bytes
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &bytes);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn decode_routine_control_request_rejects_reserved_subfunction() {
        // 0x7F (low 7 bits = 0x7F) is a reserved routineControlType
        let bytes = [0x7F, 0xFF, 0x00];
        assert!(<RoutineControlRequest as Decode>::decode(&bytes).is_err());
    }

    #[test]
    fn decode_routine_control_response() {
        let bytes = [0x01, 0xFF, 0x00, 0x10];
        let (resp, rest) = <RoutineControlResponse as Decode>::decode(&bytes).unwrap();
        assert!(rest.is_empty());
        assert_eq!(
            resp.routine_control_type,
            RoutineControlSubFunction::StartRoutine
        );
        assert_eq!(resp.raw_status_record, &[0xFF, 0x00, 0x10]);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &bytes);
        assert_encode_size_agrees(&resp);
    }
}
