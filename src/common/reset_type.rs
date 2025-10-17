use crate::Error;

/// UDS defines a number of different types of resets that can be requested
/// The reset type is used to specify the type of reset that the ECU should perform
///
/// *Note*:
///
/// Conversions from `u8` to `ResetType` are fallible and will return an [`Error`] if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ResetType {
    /// This value is reserved
    #[cfg_attr(feature = "clap", clap(skip))]
    ISOSAEReserved(u8),
    /// This `SubFunction` identifies a "hard reset" condition which simulates the power-on/start-up sequence
    /// typically performed after a server has been previously disconnected from its power supply (i.e. battery).
    /// The performed action is implementation specific and not defined by the spec.
    /// It might result in the re-initialization of both volatile memory and non-volatile memory locations to predetermined values.
    HardReset = Self::HARD_RESET,
    /// This `SubFunction` identifies a condition similar to the driver turning the ignition key off and back on.
    /// This reset condition should simulate a key-off-on sequence (i.e. interrupting the switched power supply).
    /// The performed action is implementation specific and not defined by this document.
    /// Typically the values of non-volatile mmemory locations are preserved;
    /// volatile memory will be initialized.
    KeyOffOnReset = Self::KEY_OFF_ON_RESET,
    /// This `SubFunction` identifies a "soft reset" condition, which causes the server to immediately restart the application program if applicable.
    /// The performed action is implementation specific and not defined by the spec.
    /// A typical action is to restart the application without reinitializing of previously applied configuration data,
    /// adaptive factors and other long-term adjustments.
    SoftReset = Self::SOFT_RESET,
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
    EnableRapidPowerShutDown = Self::ENABLE_RAPID_POWER_SHUTDOWN,
    /// This `SubFunction` requests the server to disable the previously enabled "rapid power shut down" function.
    DisableRapidPowerShutDown = Self::DISABLE_RAPID_POWER_SHUTDOWN,
    /// Reserved for use by vehicle manufacturers
    #[cfg_attr(feature = "clap", clap(skip))]
    VehicleManufacturerSpecific(u8),
    /// Reserved for use by system suppliers
    #[cfg_attr(feature = "clap", clap(skip))]
    SystemSupplierSpecific(u8),
}

impl ResetType {
    pub const ISO_RESERVED: u8 = 0x00;
    pub const HARD_RESET: u8 = 0x01;
    pub const KEY_OFF_ON_RESET: u8 = 0x02;
    pub const SOFT_RESET: u8 = 0x03;
    pub const ENABLE_RAPID_POWER_SHUTDOWN: u8 = 0x04;
    pub const DISABLE_RAPID_POWER_SHUTDOWN: u8 = 0x05;
    pub const ISO_RESERVED_START: u8 = 0x06;
    pub const ISO_RESERVED_END: u8 = 0x3F;
    pub const VEHICLE_MANUFACTURER_START: u8 = 0x40;
    pub const VEHICLE_MANUFACTURER_END: u8 = 0x5F;
    pub const SYSTEM_SUPPLIER_START: u8 = 0x60;
    pub const SYSTEM_SUPPLIER_END: u8 = 0x7E;
    pub const ISO_RESERVED_EXTENSION: u8 = 0x7F;
}

impl From<ResetType> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: ResetType) -> Self {
        match value {
            ResetType::ISOSAEReserved(val) => val,
            ResetType::HardReset => ResetType::HARD_RESET,
            ResetType::KeyOffOnReset => ResetType::KEY_OFF_ON_RESET,
            ResetType::SoftReset => ResetType::SOFT_RESET,
            ResetType::EnableRapidPowerShutDown => ResetType::ENABLE_RAPID_POWER_SHUTDOWN,
            ResetType::DisableRapidPowerShutDown => ResetType::DISABLE_RAPID_POWER_SHUTDOWN,
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
            Self::ISO_RESERVED => Ok(Self::ISOSAEReserved(value)),
            Self::HARD_RESET => Ok(Self::HardReset),
            Self::KEY_OFF_ON_RESET => Ok(Self::KeyOffOnReset),
            Self::SOFT_RESET => Ok(Self::SoftReset),
            Self::ENABLE_RAPID_POWER_SHUTDOWN => Ok(Self::EnableRapidPowerShutDown),
            Self::DISABLE_RAPID_POWER_SHUTDOWN => Ok(Self::DisableRapidPowerShutDown),
            Self::ISO_RESERVED_START..=Self::ISO_RESERVED_END => Ok(Self::ISOSAEReserved(value)),
            Self::VEHICLE_MANUFACTURER_START..=Self::VEHICLE_MANUFACTURER_END => {
                Ok(Self::VehicleManufacturerSpecific(value))
            }
            Self::SYSTEM_SUPPLIER_START..=Self::SYSTEM_SUPPLIER_END => {
                Ok(Self::SystemSupplierSpecific(value))
            }
            Self::ISO_RESERVED_EXTENSION => Ok(Self::ISOSAEReserved(value)),
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
