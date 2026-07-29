//! `ControlDTCSetting` (0x85) service implementation
use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, Incomplete, NegativeResponseCode};
use automotive_wire_codec::{write_all, write_u8};

/// Controls whether the server should enable or disable DTC status-bit updates.
///
/// Used by [`ControlDtcSettingRequest`] to instruct the server.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8", into = "u8"))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DtcSettingType {
    /// Re-enable DTC status-bit updates.
    On,
    /// Disable DTC status-bit updates.
    Off,
    /// Reserved for use by vehicle manufacturers (`0x40`-`0x5F`, ISO 14229-1:2020 Table 128).
    ///
    /// Construct through [`DtcSettingType::try_from`] so the raw byte is range-checked and can
    /// never collide with the SPRMIB bit.
    #[cfg_attr(feature = "clap", clap(skip))]
    #[non_exhaustive]
    VehicleManufacturerSpecific(u8),
    /// Reserved for use by system suppliers (`0x60`-`0x7E`, ISO 14229-1:2020 Table 128).
    ///
    /// Construct through [`DtcSettingType::try_from`] so the raw byte is range-checked and can
    /// never collide with the SPRMIB bit.
    #[cfg_attr(feature = "clap", clap(skip))]
    #[non_exhaustive]
    SystemSupplierSpecific(u8),
}

impl From<DtcSettingType> for u8 {
    fn from(value: DtcSettingType) -> Self {
        match value {
            DtcSettingType::On => 0x01,
            DtcSettingType::Off => 0x02,
            DtcSettingType::VehicleManufacturerSpecific(value)
            | DtcSettingType::SystemSupplierSpecific(value) => value,
        }
    }
}

impl TryFrom<u8> for DtcSettingType {
    type Error = Error;

    /// ISO 14229-1:2020 Table 128 defines `0x01`/`0x02` and two manufacturer-defined ranges,
    /// and reserves `0x00`, `0x03`-`0x3F` and `0x7F`.
    ///
    /// # Errors
    /// Returns [`Error::InvalidDtcSetting`] for a reserved value, which maps to
    /// [`NegativeResponseCode::SubFunctionNotSupported`] as Table 132 requires.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::On),
            0x02 => Ok(Self::Off),
            0x40..=0x5F => Ok(Self::VehicleManufacturerSpecific(value)),
            0x60..=0x7E => Ok(Self::SystemSupplierSpecific(value)),
            _ => Err(Error::InvalidDtcSetting(value)),
        }
    }
}

/// The `ControlDtcSetting` service is used to control the DTC settings of the ECU.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ControlDtcSettingRequest<'d> {
    /// Whether the server should suppress the positive response (SPRMIB).
    pub suppress_positive_response: bool,
    /// The requested DTC logging setting.
    pub setting: DtcSettingType,
    /// Optional `DTCSettingControlOptionRecord`, empty when absent.
    ///
    /// Marked `U` (user option) in ISO 14229-1:2020 Table 127. Table 129 describes it as
    /// vehicle-manufacturer specific data qualifying the request — for example a list of the
    /// DTCs to turn on or off — and Table 132 reserves
    /// [`NegativeResponseCode::RequestOutOfRange`] for a server that detects an error in it.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub option_record: &'d [u8],
}

const CONTROL_DTC_SETTING_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
];

impl<'d> ControlDtcSettingRequest<'d> {
    /// Create a new `ControlDtcSettingRequest` with no `DTCSettingControlOptionRecord`.
    #[must_use]
    pub const fn new(suppress_positive_response: bool, setting: DtcSettingType) -> Self {
        Self {
            suppress_positive_response,
            setting,
            option_record: &[],
        }
    }

    /// Create a request carrying a `DTCSettingControlOptionRecord`.
    ///
    /// The record's contents are vehicle-manufacturer specific (ISO 14229-1:2020 Table 129).
    #[must_use]
    pub const fn new_with_option_record(
        suppress_positive_response: bool,
        setting: DtcSettingType,
        option_record: &'d [u8],
    ) -> Self {
        Self {
            suppress_positive_response,
            setting,
            option_record,
        }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &CONTROL_DTC_SETTING_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for ControlDtcSettingRequest<'_> {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        let sub_function =
            SuppressablePositiveResponse::new(self.suppress_positive_response, self.setting);
        let mut written = write_u8(writer, u8::from(sub_function)).map_err(Error::io)?;
        written += write_all(writer, self.option_record).map_err(Error::io)?;
        Ok(written)
    }
}

impl<'a> Decode<'a> for ControlDtcSettingRequest<'a> {
    type Error = crate::Error;

    /// The sub-function byte is mandatory; everything after it is the optional
    /// `DTCSettingControlOptionRecord` (ISO 14229-1:2020 Table 127, `Cvt` = `U`), whose length
    /// is not on the wire — the record runs to the end of the message.
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        let [sub_function, option_record @ ..] = buf else {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        };
        let sub_function = SuppressablePositiveResponse::<DtcSettingType>::try_from(*sub_function)?;
        Ok((
            Self {
                suppress_positive_response: sub_function.suppress_positive_response(),
                setting: sub_function.value(),
                option_record,
            },
            &[],
        ))
    }
}

/// Positive response to a `ControlDtcSettingRequest`
///
/// The ECU will respond with a `ControlDtcSettingResponse` if the request was successful.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ControlDtcSettingResponse {
    /// The DTC logging setting that was set in the request
    pub setting: DtcSettingType,
}

impl ControlDtcSettingResponse {
    /// Create a new `ControlDtcSettingResponse`.
    #[must_use]
    pub const fn new(setting: DtcSettingType) -> Self {
        Self { setting }
    }
}

impl Encode for ControlDtcSettingResponse {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        write_u8(writer, u8::from(self.setting)).map_err(Error::io)
    }
}

impl<'a> Decode<'a> for ControlDtcSettingResponse {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        let setting = DtcSettingType::try_from(buf[0])?;
        Ok((Self { setting }, &buf[1..]))
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, Encode, NegativeResponseCode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::{vec, vec::Vec};

    #[cfg(feature = "alloc")]
    #[test]
    fn simple_request() {
        let req = ControlDtcSettingRequest::new(true, DtcSettingType::On);
        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x81]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.encoded_size().unwrap(), buffer.len());

        let (parsed, _) = <ControlDtcSettingRequest as Decode>::decode(&buffer).unwrap();
        assert_eq!(parsed.setting, DtcSettingType::On);
        assert!(parsed.suppress_positive_response);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn manufacturer_and_supplier_specific_setting_types_are_supported() {
        // ISO 14229-1:2020 Table 128 reserves 0x40-0x5F for vehicle-manufacturer use and
        // 0x60-0x7E for system-supplier use. Both were rejected outright, so a server could
        // not even see the byte in order to answer SubFunctionNotSupported for it, and a
        // client could not send a manufacturer-defined setting at all. Every sibling
        // sub-function enum in the crate models these ranges.
        for (byte, expected) in [
            (0x40u8, DtcSettingType::VehicleManufacturerSpecific(0x40)),
            (0x5F, DtcSettingType::VehicleManufacturerSpecific(0x5F)),
            (0x60, DtcSettingType::SystemSupplierSpecific(0x60)),
            (0x7E, DtcSettingType::SystemSupplierSpecific(0x7E)),
        ] {
            let wire = [byte];
            let (req, _) = <ControlDtcSettingRequest as Decode>::decode(&wire).unwrap();
            assert_eq!(req.setting, expected, "for sub-function {byte:#04X}");

            let mut buf = [0u8; 4];
            let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
            assert_eq!(&buf[..written], &[byte], "round trip for {byte:#04X}");
        }
    }

    #[test]
    fn reserved_setting_types_are_still_rejected() {
        // Table 128 reserves 0x00, 0x03-0x3F and 0x7F. ControlDTCSetting and RoutineControl are
        // the two services that validate their sub-function, so both must answer NRC 0x12.
        for byte in [0x00u8, 0x03, 0x3F, 0x7F] {
            let err = <ControlDtcSettingRequest as Decode>::decode(&[byte])
                .expect_err("a reserved DTCSettingType must be rejected");
            assert!(
                matches!(err, Error::InvalidDtcSetting(got) if got == byte),
                "wrong error for {byte:#04X}: {err:?}"
            );
            assert_eq!(
                err.negative_response_code(),
                NegativeResponseCode::SubFunctionNotSupported
            );
        }
    }

    #[test]
    fn a_dtc_setting_control_option_record_round_trips() {
        // Table 127 marks DTCSettingControlOptionRecord `U`, and Table 129 describes it as
        // carrying e.g. a list of DTCs to turn on or off. Table 132 reserves NRC 0x31 for an
        // error *in that record*, which a server can only detect if it is decoded at all.
        // Without it, `85 02 AA BB CC` was rejected as having trailing bytes.
        let wire = [0x85, 0x02, 0xAA, 0xBB, 0xCC];
        let (req, _) = crate::Request::decode(&wire).unwrap();
        let crate::Request::ControlDtcSetting(inner) = req else {
            panic!("expected a ControlDtcSetting request, got {req:?}");
        };
        assert_eq!(inner.setting, DtcSettingType::Off);
        assert_eq!(inner.option_record, &[0xAA, 0xBB, 0xCC]);

        let mut buf = [0u8; 8];
        let written = req.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &wire);
    }

    #[test]
    fn an_absent_option_record_is_an_empty_slice() {
        // Table 133's flow example is the bare two-byte form, so absent must stay absent
        // rather than becoming a zero byte on the wire.
        let (req, _) = crate::Request::decode(&[0x85, 0x02]).unwrap();
        let crate::Request::ControlDtcSetting(inner) = req else {
            panic!("expected a ControlDtcSetting request");
        };
        assert!(inner.option_record.is_empty());

        let mut buf = [0u8; 8];
        let written = req.encode_to_slice(&mut buf).unwrap();
        assert_eq!(&buf[..written], &[0x85, 0x02]);
    }

    #[test]
    fn invalid_setting_byte_carries_the_value() {
        // An unrecognized setting must surface the offending byte, like every other
        // service's Invalid<Service>Type error, not the generic length/format error.
        let err = <ControlDtcSettingRequest as Decode>::decode(&[0x09]).unwrap_err();
        assert!(matches!(err, Error::InvalidDtcSetting(0x09)));
    }

    #[test]
    fn exposes_allowed_nack_codes() {
        assert!(!ControlDtcSettingRequest::allowed_nack_codes().is_empty());
        assert!(
            ControlDtcSettingRequest::allowed_nack_codes()
                .contains(&NegativeResponseCode::RequestOutOfRange)
        );
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::{vec, vec::Vec};

    #[cfg(feature = "alloc")]
    #[test]
    fn simple_response() {
        let req = ControlDtcSettingResponse::new(DtcSettingType::On);
        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(buffer, vec![0x01]);
        assert_eq!(written, buffer.len());
        assert_eq!(req.encoded_size().unwrap(), buffer.len());

        let (parsed, _) = <ControlDtcSettingResponse as Decode>::decode(&buffer).unwrap();
        assert_eq!(parsed.setting, DtcSettingType::On);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn response_is_eq() {
        crate::test_util::assert_impl_eq::<ControlDtcSettingResponse>();
    }
}
