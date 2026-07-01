//! Routine Control (0x31) Service is used to perform functions on the ECU that may not be covered by other services.
//!
//! It can also be used to check the ECU's health, erase memory, or other custom manufacturer/supplier routines.
//! However, some routines may have side effects or require certain preconditions to be met.
use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, NegativeResponseCode};

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

const ROUTINE_CONTROL_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 7] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestSequenceError,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::GeneralProgrammingFailure,
];

/// Used by a client to execute a defined sequence of events and obtain any relevant results.
///
/// The 2-byte big-endian routine identifier is decoded into a typed `u16`, followed by
/// optional routine input parameters in `option_record`.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequest<'d> {
    /// Whether the server should suppress the positive response (SPRMIB).
    pub suppress_positive_response: bool,
    /// The routine control operation (start, stop, or request results).
    pub sub_function: RoutineControlSubFunction,
    /// The 16-bit routine identifier.
    pub routine_id: u16,
    /// Optional routine input parameters (may be empty).
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub option_record: &'d [u8],
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
            suppress_positive_response,
            sub_function,
            routine_id,
            option_record,
        }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &ROUTINE_CONTROL_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for RoutineControlRequest<'_> {
    fn encoded_size(&self) -> usize {
        1 + 2 + self.option_record.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let sub_function =
            SuppressablePositiveResponse::new(self.suppress_positive_response, self.sub_function);
        writer
            .write_all(&[u8::from(sub_function)])
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
        let sub_function =
            SuppressablePositiveResponse::<RoutineControlSubFunction>::try_from(buf[0])?;
        let routine_id = u16::from_be_bytes([buf[1], buf[2]]);
        Ok((
            Self {
                suppress_positive_response: sub_function.suppress_positive_response(),
                sub_function: sub_function.value(),
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponse<'d> {
    /// The routine control operation echoed from the request (start, stop, or request results).
    pub sub_function: RoutineControlSubFunction,
    /// The 16-bit routine identifier echoed from the request.
    pub routine_id: u16,
    /// Optional routine status bytes (may be empty).
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub status_record: &'d [u8],
}

impl<'d> RoutineControlResponse<'d> {
    /// Create a new `RoutineControlResponse`.
    #[must_use]
    pub const fn new(
        sub_function: RoutineControlSubFunction,
        routine_id: u16,
        status_record: &'d [u8],
    ) -> Self {
        Self {
            sub_function,
            routine_id,
            status_record,
        }
    }
}

impl Encode for RoutineControlResponse<'_> {
    fn encoded_size(&self) -> usize {
        1 + 2 + self.status_record.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.sub_function)])
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
        let sub_function = RoutineControlSubFunction::try_from(buf[0])?;
        let routine_id = u16::from_be_bytes([buf[1], buf[2]]);
        Ok((
            Self {
                sub_function,
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
    use crate::{Decode, NegativeResponseCode};
    use crate::test_util::{assert_encode_size_agrees, assert_impl_eq};

    #[test]
    fn derive_contract() {
        assert_impl_eq::<RoutineControlRequest<'_>>();
        assert_impl_eq::<RoutineControlResponse<'_>>();

        #[cfg(feature = "serde")]
        {
            use crate::test_util::assert_impl_serde;
            assert_impl_serde::<RoutineControlRequest<'_>>();
            assert_impl_serde::<RoutineControlResponse<'_>>();
        }
    }

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
        assert!(d.suppress_positive_response);
        assert_eq!(d.sub_function, RoutineControlSubFunction::StartRoutine);
        assert_eq!(d.routine_id, 0xFF00);
        assert_eq!(d.option_record, &[0xAA]);
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
        assert_eq!(d.sub_function, RoutineControlSubFunction::StartRoutine);
        assert_eq!(d.routine_id, 0xFF00);
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

    #[test]
    fn exposes_allowed_nack_codes() {
        assert!(!RoutineControlRequest::allowed_nack_codes().is_empty());
        assert!(RoutineControlRequest::allowed_nack_codes()
            .contains(&NegativeResponseCode::SecurityAccessDenied));
    }
}
