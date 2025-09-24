use crate::Error;

/// UDS defines a number of different types of resets that can be requested
/// The reset type is used to specify the type of reset that the ECU should perform
///
/// *Note*:
///
/// Conversions from `u8` to `ResetType` are fallible and will return an [`Error`] if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, utoipa::ToSchema, clap::ValueEnum)]
pub enum ResetType {
    /// This value is reserved
    #[clap(skip)]
    ISOSAEReserved(u8),
    /// This `SubFunction` identifies a "hard reset" condition which simulates the power-on/start-up sequence
    /// typically performed after a server has been previously disconnected from its power supply (i.e. battery).
    /// The performed action is implementation specific and not defined by the spec.
    /// It might result in the re-initialization of both volatile memory and non-volatile memory locations to predetermined values.
    HardReset,
    /// This `SubFunction` identifies a condition similar to the driver turning the ignition key off and back on.
    /// This reset condition should simulate a key-off-on sequence (i.e. interrupting the switched power supply).
    /// The performed action is implementation specific and not defined by this document.
    /// Typically the values of non-volatile mmemory locations are preserved;
    /// volatile memory will be initialized.
    KeyOffOnReset,
    /// This `SubFunction` identifies a "soft reset" condition, which causes the server to immediately restart the application program if applicable.
    /// The performed action is implementation specific and not defined by the spec.
    /// A typical action is to restart the application without reinitializing of previously applied configuration data,
    /// adaptive factors and other long-term adjustments.
    SoftReset,
    /// This `SubFunction` applies to ECUs which are not ignition powered but battery powered only.
    /// Therefore a shutdown forces the sleep mode rather than a power off.
    /// Sleep means power off but still ready for wake-up (battery powered).
    /// The intention of the `SubFunction` is to reduce the stand-by time of an ECU after ignition is turned into the off position.
    /// This value requests the server to enable and perform a "rapid powershut down" function.
    /// The server shall execute the function immediately once the "key/ignition” is switched off.
    /// While the server executes the power down function,
    /// it shall transition either directly or after a defined stand-by-time to sleep mode.
    /// If the client requires a response message and the server is already prepared to execute the "rapid power shutdown" function,
    /// the server shall send the positive response message prior to the start of the "rapid power shut down" function.
    /// The next occurrence of a "key on" or "ignition on" signal terminates the "rapid power shut down" function.
    /// **NOTE** This `SubFunction` is only applicable to a server supporting a stand-by-mode.
    EnableRapidPowerShutDown,
    /// This `SubFunction` requests the server to disable the previously enabled "rapid power shut down" function.
    DisableRapidPowerShutDown,
    /// Reserved for use by vehicle manufacturers
    #[clap(skip)]
    VehicleManufacturerSpecific(u8),
    /// Reserved for use by system suppliers
    #[clap(skip)]
    SystemSupplierSpecific(u8),
}

impl From<ResetType> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: ResetType) -> Self {
        match value {
            ResetType::ISOSAEReserved(val) => val,
            ResetType::HardReset => 0x01,
            ResetType::KeyOffOnReset => 0x02,
            ResetType::SoftReset => 0x03,
            ResetType::EnableRapidPowerShutDown => 0x04,
            ResetType::DisableRapidPowerShutDown => 0x05,
            ResetType::VehicleManufacturerSpecific(val) => val,
            ResetType::SystemSupplierSpecific(val) => val,
        }
    }
}

impl TryFrom<u8> for ResetType {
    type Error = Error;
    #[allow(clippy::match_same_arms)]
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(Self::ISOSAEReserved(0)),
            0x01 => Ok(Self::HardReset),
            0x02 => Ok(Self::KeyOffOnReset),
            0x03 => Ok(Self::SoftReset),
            0x04 => Ok(Self::EnableRapidPowerShutDown),
            0x05 => Ok(Self::DisableRapidPowerShutDown),
            0x06..=0x3F => Ok(Self::ISOSAEReserved(value)),
            0x40..=0x5F => Ok(Self::VehicleManufacturerSpecific(value)),
            0x60..=0x7E => Ok(Self::SystemSupplierSpecific(value)),
            0x7F => Ok(Self::ISOSAEReserved(value)),
            _ => Err(Error::InvalidEcuResetType(value)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    #[allow(clippy::match_same_arms)]
    fn reset_type_from_all_u8_values() {
        for i in 0..=u8::MAX {
            let reset_type: Result<ResetType, Error> = ResetType::try_from(i);
            match i {
                0x00 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::ISOSAEReserved(_)),
                )),
                0x01 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::HardReset),
                )),
                0x02 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::KeyOffOnReset),
                )),
                0x03 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::SoftReset),
                )),
                0x04 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::EnableRapidPowerShutDown),
                )),
                0x05 => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::DisableRapidPowerShutDown),
                )),
                0x06..=0x3F => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::ISOSAEReserved(_)),
                )),
                0x40..=0x5F => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::VehicleManufacturerSpecific(_)),
                )),
                0x60..=0x7E => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::SystemSupplierSpecific(_)),
                )),
                0x7F => assert!(matches!(
                    reset_type,
                    Ok::<ResetType, Error>(ResetType::ISOSAEReserved(_)),
                )),
                _ => assert!(matches!(
                    reset_type,
                    Err::<ResetType, Error>(Error::InvalidEcuResetType(_)),
                )),
            }
        }
    }

    #[test]
    fn reset_type_to_all_u8_values() {
        assert_eq!(u8::from(ResetType::ISOSAEReserved(0)), 0x00);
        assert_eq!(u8::from(ResetType::HardReset), 0x01);
        assert_eq!(u8::from(ResetType::KeyOffOnReset), 0x02);
        assert_eq!(u8::from(ResetType::SoftReset), 0x03);
        assert_eq!(u8::from(ResetType::EnableRapidPowerShutDown), 0x04);
        assert_eq!(u8::from(ResetType::DisableRapidPowerShutDown), 0x05);
    }
}
