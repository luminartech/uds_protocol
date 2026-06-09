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
/// The 2-byte big-endian routine identifier is decoded into a typed `u16`, followed by
/// optional routine input parameters in `option_record`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequest<'d> {
    sub_function: SuppressablePositiveResponse<RoutineControlSubFunction>,
    routine_id: u16,
    option_record: &'d [u8],
}

impl<'d> RoutineControlRequest<'d> {
    /// Create a new `RoutineControlRequest`.
    #[must_use]
    pub const fn new(
        suppress_positive_response: bool,
        sub_function: RoutineControlSubFunction,
        routine_id: u16,
        option_record: &'d [u8],
    ) -> Self {
        Self {
            sub_function: SuppressablePositiveResponse::new(
                suppress_positive_response,
                sub_function,
            ),
            routine_id,
            option_record,
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

    /// The 16-bit routine identifier.
    #[must_use]
    pub const fn routine_id(&self) -> u16 {
        self.routine_id
    }

    /// Optional routine input parameters (may be empty).
    #[must_use]
    pub const fn option_record(&self) -> &[u8] {
        self.option_record
    }
}

impl Encode for RoutineControlRequest<'_> {
    fn encoded_size(&self) -> usize {
        1 + 2 + self.option_record.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.sub_function)])
            .map_err(Error::io)?;
        writer
            .write_all(&self.routine_id.to_be_bytes())
            .map_err(Error::io)?;
        writer.write_all(self.option_record).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for RoutineControlRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 3 {
            return Err(Error::InsufficientData(3));
        }
        let sub_function = SuppressablePositiveResponse::try_from(buf[0])?;
        let routine_id = u16::from_be_bytes([buf[1], buf[2]]);
        Ok((
            Self {
                sub_function,
                routine_id,
                option_record: &buf[3..],
            },
            &[],
        ))
    }
}

/// `RoutineControlResponse` is a variable-length response that can contain routine status.
///
/// The 2-byte big-endian routine identifier echo is decoded into a typed `u16`, followed
/// by optional routine status bytes in `status_record`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponse<'d> {
    routine_control_type: RoutineControlSubFunction,
    routine_id: u16,
    status_record: &'d [u8],
}

impl<'d> RoutineControlResponse<'d> {
    /// Create a new `RoutineControlResponse`.
    #[must_use]
    pub const fn new(
        routine_control_type: RoutineControlSubFunction,
        routine_id: u16,
        status_record: &'d [u8],
    ) -> Self {
        Self {
            routine_control_type,
            routine_id,
            status_record,
        }
    }

    /// The routine control type echoed from the request.
    #[must_use]
    pub const fn routine_control_type(&self) -> RoutineControlSubFunction {
        self.routine_control_type
    }

    /// The 16-bit routine identifier echoed from the request.
    #[must_use]
    pub const fn routine_id(&self) -> u16 {
        self.routine_id
    }

    /// Optional routine status bytes (may be empty).
    #[must_use]
    pub const fn status_record(&self) -> &[u8] {
        self.status_record
    }
}

impl Encode for RoutineControlResponse<'_> {
    fn encoded_size(&self) -> usize {
        1 + 2 + self.status_record.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.routine_control_type)])
            .map_err(Error::io)?;
        writer
            .write_all(&self.routine_id.to_be_bytes())
            .map_err(Error::io)?;
        writer.write_all(self.status_record).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for RoutineControlResponse<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 3 {
            return Err(Error::InsufficientData(3));
        }
        // Plain try_from (no SPRMIB mask): a set 0x80 bit on a response is malformed.
        let routine_control_type = RoutineControlSubFunction::try_from(buf[0])?;
        let routine_id = u16::from_be_bytes([buf[1], buf[2]]);
        Ok((
            Self {
                routine_control_type,
                routine_id,
                status_record: &buf[3..],
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
    fn rc_request_round_trips_with_suppress() {
        let req = RoutineControlRequest::new(
            true,
            RoutineControlSubFunction::StartRoutine,
            0xFF00,
            &[0xAA],
        );
        let mut buf = [0u8; 8];
        let n = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..n], &[0x81, 0xFF, 0x00, 0xAA]); // 0x81 = StartRoutine | SPRMIB
        let (d, rest) = <RoutineControlRequest as Decode>::decode(&buf[..n]).unwrap();
        assert!(rest.is_empty());
        assert!(d.suppress_positive_response());
        assert_eq!(d.sub_function(), RoutineControlSubFunction::StartRoutine);
        assert_eq!(d.routine_id(), 0xFF00);
        assert_eq!(d.option_record(), &[0xAA]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn rc_request_rejects_short_buffer() {
        assert!(<RoutineControlRequest as Decode>::decode(&[0x01, 0xFF]).is_err());
    }

    #[test]
    fn rc_response_round_trips_and_rejects_sprmib_bit() {
        let resp =
            RoutineControlResponse::new(RoutineControlSubFunction::StartRoutine, 0xFF00, &[0x10]);
        let mut buf = [0u8; 8];
        let n = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..n], &[0x01, 0xFF, 0x00, 0x10]);
        let (d, _) = <RoutineControlResponse as Decode>::decode(&buf[..n]).unwrap();
        assert_eq!(
            d.routine_control_type(),
            RoutineControlSubFunction::StartRoutine
        );
        assert_eq!(d.routine_id(), 0xFF00);
        // A response with the SPRMIB bit set (0x81) is malformed and rejected.
        assert!(<RoutineControlResponse as Decode>::decode(&[0x81, 0xFF, 0x00]).is_err());
        assert_encode_size_agrees(&resp);
    }

    #[test]
    fn decode_routine_control_request_rejects_reserved_subfunction() {
        // 0x7F (low 7 bits = 0x7F) is a reserved routineControlType
        let bytes = [0x7F, 0xFF, 0x00];
        assert!(<RoutineControlRequest as Decode>::decode(&bytes).is_err());
    }
}
