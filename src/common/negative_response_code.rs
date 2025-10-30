/// `NegativeResponseCode` is a shared error mechanism
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum NegativeResponseCode {
    /// This response code shall not be used in a negative response message.
    /// This positiveResponse parameter value is reserved for server internal implementation
    PositiveResponse = 0x00,
    /// This range of values is reserved for future definition.
    #[cfg_attr(feature = "clap", clap(skip))]
    ISOSAEReserved(u8),
    /// This response code indicates that the requested action has been rejected by the server.
    /// The `GeneralReject` response code shall only be implemented in the server if none of the negative response codes meet the needs of the implementation.
    /// This response code shall not be used as a general replacement for other response codes defined.
    GeneralReject = 0x10,
    /// This response code indicates that the requested action will not be taken because the server does not support the requested service.
    /// The server shall send this response code in the case where the client has sent a request message with a service identifier which is either unknown or not supported.
    /// This negative response code is not shown in the list of negative response codes to be supported for a diagnostic service,
    /// because this negative response code is not applicable for supported services.
    ServiceNotSupported = 0x11,
    /// This response code indicates that the requested action will not be taken because the server does not support the service specific parameters of the request message.
    /// The server shall send this response code in case the client has sent a request message with a known and supported service identifier but with "sub function“ which is either unknown or not supported.
    SubFunctionNotSupported = 0x12,
    /// This response code indicates that the requested action will not be taken because the length of the received request message does not match the prescribed length for the specified service,
    /// or that the format of the parameters do not match the prescribed format for the specified service.
    IncorrectMessageLengthOrInvalidFormat = 0x13,
    /// This response code shall be reported by the server if the response to be generated exceeds the maximum number of bytes available by the underlying network layer.
    ResponseTooLong = 0x14,
    // -- ISO SAE Reserved: 0x15 to 0x20 --
    /// This response code indicates that the server is temporarily too busy to perform the requested operation.
    /// In this circumstance the client shall perform repetition of the request message or "try again later".
    /// The repetition of the request shall be delayed by a time specified in the respective implementation documents.
    BusyRepeatRequest = 0x21,
    /// This response code indicates that the requested action will not be taken because the server prerequisite conditions are not met.
    ConditionsNotCorrect = 0x22,
    // -- ISO SAE Reserved: 0x23 --
    /// This response code indicates that the requested action will not be taken because the server expects a different sequence of request messages or message as sent by the client.
    /// This may occur when sequence sensitive requests are issued in the wrong order.
    RequestSequenceError = 0x24,
    /// This NRC indicates that the server has received the request but the requested action could not be performed by the server as a subnet component which is
    /// necessary to supply the requested information did not respond within the specified time.
    ///
    /// The noResponseFromSubnetComponent negative response shall be implemented by gateways in electronic systems which contain electronic
    /// subnet components and which do not directly respond to the client's request.
    ///
    /// The gateway may receive the request for the subnet component and then request the necessary information from the subnet component. If the subnet
    /// component fails to respond, the server shall use this negative response to inform the client about the failure of the subnet component.
    ///
    /// This NRC is in general supported by each diagnostic service, as not otherwise stated in the data link specific implementation document, therefore it is not
    /// listed `FailurePreventsExecutionOfRequestedAction` in the list of applicable response codes of the diagnostic services. `NegativeResponseCode` is a shared error mechanism
    NoResponseFromSubnetComponent = 0x25,
    /// This NRC indicates that the requested action will not be taken because a failure condition, identified by a DTC (with at least one DTC status bit for
    /// `TestFailed`, `Pending`, `Confirmed` or `TestFailedSinceLastClear` set to 1), has occurred and that this failure condition prevents the server from performing the requested action.
    ///
    /// This NRC can, for example, direct the technician to read DTCs in order to identify and fix the problem.
    FailurePreventsExecutionOfRequestedAction = 0x26,
    /// This response code indicates that the requested action will not be taken because the server has detected that the request message contains a parameter which attempts to substitute a value beyond its range of authority
    /// (e.g. attempting to substitute a data byte of 111 when the data is only defined to 100),
    /// or which attempts to access a dataIdentifier/routineIdentifer that is not supported or not supported in active session.
    ///
    /// This response code shall be implemented for all services which allow the client to read data,
    /// write data, or adjust functions by data in the server.
    RequestOutOfRange = 0x31,
    /// This response code indicates that the requested action will not be taken because the server's security strategy has not been satisfied by the client. The server shall send this response code if one of the following cases occur:
    /// - the test conditions of the server are not met,
    /// - the required message sequence e.g. `DiagnosticSessionControl`, securityAccess is not met,
    /// - the client has sent a request message which requires an unlocked server.
    ///
    /// Beside the mandatory use of this negative response code as specified in the applicable services within this standard,
    /// this negative response code can also be used for any case where security is required and is not yet granted to perform the required service.
    SecurityAccessDenied = 0x33,
    /// This response code indicates that the requested action will not be taken because the client has insufficient rights based on its Authentication state.
    AuthenticationRequired = 0x34,
    /// This response code indicates that the server has not given security access because the key sent by the client did not match with the key in the server's memory.
    /// This counts as an attempt to gain security.
    /// The server shall remain locked and increment its internal securityAccessFailed counter.
    InvalidKey = 0x35,
    /// This response code indicates that the requested action will not be taken because the client has unsuccessfully attempted to gain security access more times than the server's security strategy will allow.
    ExceedNumberOfAttempts = 0x36,
    /// This response code indicates that the requested action will not be taken because the client's latest attempt to gain security access was initiated before the server's required timeout period had elapsed.
    RequiredTimeDelayNotExpired = 0x37,
    /// Reserved by ISO 15764
    #[cfg_attr(feature = "clap", clap(skip))]
    ExtendedDataLinkSecurityReserved(u8),
    /// This response code indicates that an attempt to upload/download to a server's memory cannot be accomplished due to some fault conditions.
    UploadDownloadNotAccepted = 0x70,
    /// This response code indicates that a data transfer operation was halted due to some fault.
    /// The active transferData sequence shall be aborted.
    TransferDataSuspended = 0x71,
    /// This response code indicates that the server detected an error when erasing or programming a memory location in the permanent memory device (e.g. Flash Memory).
    GeneralProgrammingFailure = 0x72,
    /// This response code indicates that the server detected an error in the sequence of `BlockSequenceCounter` values.
    /// Note that the repetition of a `TransferDataRequest` message with a `BlockSequenceCounter` equal to the one included in the previous `TransferDataRequest` message shall be accepted by the server.
    WrongBlockSequenceCounter = 0x73,
    /// This response code indicates that the server detected an error in the sequence of `BlockSequenceCounter` values.
    RequestCorrectlyReceivedResponsePending = 0x78,
    // -- ISO SAE Reserved: 0x79 to 0x7D --
    /// This response code indicates that the requested action will not be taken because the server does not support the requested sub-function in the session currently active.
    /// Within the programmingSession negative response code 0x12 (subFunctionNotSupported) may optionally be reported instead of negative response code 0x7F (subFunctionNotSupportedInActiveSession).
    /// This response code shall only be used when the requested sub-function is known to be supported in another session,
    /// otherwise response code 0x12 (subFunctionNotSupported) shall be used.
    /// This response code shall be supported by each diagnostic service with a sub-function parameter,
    /// if not otherwise stated in the data link specific implementation document,
    /// therefore it is not listed in the list of applicable response codes of the diagnostic services.
    SubFunctionNotSupportedInActiveSession = 0x7E,
    /// This response code indicates that the requested action will not be taken because the server does not support the requested service in the session currently active.
    /// This response code shall only be used when the requested service is known to be supported in another session, otherwise response code 0x11 (serviceNotSupported) shall be used.
    /// This response code is in general supported by each diagnostic service,
    /// as not otherwise stated in the data link specific implementation document,
    /// therefore it is not listed in the list of applicable response codes of the diagnostic services.
    ServiceNotSupportedInActiveSession = 0x7F,
    // -- ISO SAE Reserved: 0x80 --
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for RPM is not met (current RPM is above a pre-programmed maximum threshold).
    RPMTooHigh = 0x81,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for RPM is not met (current RPM is below a pre-programmed minimum threshold).
    RPMTooLow = 0x82,
    /// This is required for those actuator tests which cannot be actuated while the Engine is running.
    /// This is different from RPM too high negative response and needs to be allowed.
    EngineIsRunning = 0x83,
    /// This is required for those actuator tests which cannot be actuated unless the Engine is running.
    /// This is different from RPM too low negative response, and needs to be allowed.
    EngineIsNotRunning = 0x84,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for engine run time is not met
    /// (current engine run time is below a preprogrammed limit).
    EngineRunTimeTooLow = 0x85,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for temperature is not met
    /// (current temperature is above a preprogrammed maximum threshold).
    TemperatureTooHigh = 0x86,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for temperature is not met
    /// (current temperature is below a preprogrammed minimum threshold).
    TemperatureTooLow = 0x87,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for vehicle speed is not met
    /// (current VS is above a pre-programmed maximum threshold).
    VehicleSpeedTooHigh = 0x88,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for vehicle speed is not met
    /// (current VS is below a pre-programmed minimum threshold).
    VehicleSpeedTooLow = 0x89,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for throttle/pedal position is not met
    /// (current TP/APP is above a preprogrammed maximum threshold).
    ThrottleOrPedalTooHigh = 0x8A,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for throttle/pedal position is not met
    /// (current TP/APP is below a preprogrammed minimum threshold).
    ThrottleOrPedalTooLow = 0x8B,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for being in neutral is not met
    /// (current transmission range is not in neutral).
    TransmissionRangeNotInNeutral = 0x8C,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for being in gear is not met
    /// (current transmission range is not in gear).
    TransmissionRangeNotInGear = 0x8D,
    // -- ISO SAE Reserved: 0x8E --
    /// For safety reasons, this is required for certain tests before it begins,
    /// and must be maintained for the entire duration of the test.
    BrakeSwitchNotClosed = 0x8F,
    /// For safety reasons, this is required for certain tests before it begins,
    /// and must be maintained for the entire duration of the test.
    ShifterLeverNotInPark = 0x90,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for torque converter clutch is not met
    /// (current TCC status above a preprogrammed limit or locked).
    TorqueConverterClutchLocked = 0x91,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for voltage at the primary pin of the server
    /// (ECU) is not met
    /// (current voltage is above a pre-programmed maximum threshold).
    VoltageTooHigh = 0x92,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for voltage at the primary pin of the server
    /// (ECU) is not met
    /// (current voltage is below a pre-programmed maximum threshold).
    VoltageTooLow = 0x93,
    /// This response code indicates that the server has received the request but the requested action could not be performed by the server because an application which is
    /// necessary to supply the requested information is temporality not available. This NRC is in general supported by each diagnostic service, as not otherwise
    /// stated in the data link specific implementation document, therefore it is not listed reservedForSpecificConditionsNotCorrect
    /// in the list of applicable response codes of the diagnostic services.
    ResourceTemporarilyNotAvailable = 0x94,
    /// This range of values is reserved for future definition.
    #[cfg_attr(feature = "clap", clap(skip))]
    ReservedForSpecificConditionsNotMet(u8),
}

impl From<NegativeResponseCode> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: NegativeResponseCode) -> Self {
        match value {
            NegativeResponseCode::PositiveResponse => 0x00,
            NegativeResponseCode::ISOSAEReserved(value) => value,
            NegativeResponseCode::GeneralReject => 0x10,
            NegativeResponseCode::ServiceNotSupported => 0x11,
            NegativeResponseCode::SubFunctionNotSupported => 0x12,
            NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat => 0x13,
            NegativeResponseCode::ResponseTooLong => 0x14,
            NegativeResponseCode::BusyRepeatRequest => 0x21,
            NegativeResponseCode::ConditionsNotCorrect => 0x22,
            NegativeResponseCode::RequestSequenceError => 0x24,
            NegativeResponseCode::NoResponseFromSubnetComponent => 0x25,
            NegativeResponseCode::FailurePreventsExecutionOfRequestedAction => 0x26,
            NegativeResponseCode::RequestOutOfRange => 0x31,
            NegativeResponseCode::SecurityAccessDenied => 0x33,
            NegativeResponseCode::AuthenticationRequired => 0x34,
            NegativeResponseCode::InvalidKey => 0x35,
            NegativeResponseCode::ExceedNumberOfAttempts => 0x36,
            NegativeResponseCode::RequiredTimeDelayNotExpired => 0x37,
            NegativeResponseCode::ExtendedDataLinkSecurityReserved(value) => value,
            NegativeResponseCode::UploadDownloadNotAccepted => 0x70,
            NegativeResponseCode::TransferDataSuspended => 0x71,
            NegativeResponseCode::GeneralProgrammingFailure => 0x72,
            NegativeResponseCode::WrongBlockSequenceCounter => 0x73,
            NegativeResponseCode::RequestCorrectlyReceivedResponsePending => 0x78,
            NegativeResponseCode::SubFunctionNotSupportedInActiveSession => 0x7E,
            NegativeResponseCode::ServiceNotSupportedInActiveSession => 0x7F,
            NegativeResponseCode::RPMTooHigh => 0x81,
            NegativeResponseCode::RPMTooLow => 0x82,
            NegativeResponseCode::EngineIsRunning => 0x83,
            NegativeResponseCode::EngineIsNotRunning => 0x84,
            NegativeResponseCode::EngineRunTimeTooLow => 0x85,
            NegativeResponseCode::TemperatureTooHigh => 0x86,
            NegativeResponseCode::TemperatureTooLow => 0x87,
            NegativeResponseCode::VehicleSpeedTooHigh => 0x88,
            NegativeResponseCode::VehicleSpeedTooLow => 0x89,
            NegativeResponseCode::ThrottleOrPedalTooHigh => 0x8A,
            NegativeResponseCode::ThrottleOrPedalTooLow => 0x8B,
            NegativeResponseCode::TransmissionRangeNotInNeutral => 0x8C,
            NegativeResponseCode::TransmissionRangeNotInGear => 0x8D,
            NegativeResponseCode::BrakeSwitchNotClosed => 0x8F,
            NegativeResponseCode::ShifterLeverNotInPark => 0x90,
            NegativeResponseCode::TorqueConverterClutchLocked => 0x91,
            NegativeResponseCode::VoltageTooHigh => 0x92,
            NegativeResponseCode::VoltageTooLow => 0x93,
            NegativeResponseCode::ResourceTemporarilyNotAvailable => 0x94,
            NegativeResponseCode::ReservedForSpecificConditionsNotMet(value) => value,
        }
    }
}

impl From<u8> for NegativeResponseCode {
    #[allow(clippy::match_same_arms)]
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::PositiveResponse,
            0x01..=0x0F => Self::ISOSAEReserved(value),
            0x10 => Self::GeneralReject,
            0x11 => Self::ServiceNotSupported,
            0x12 => Self::SubFunctionNotSupported,
            0x13 => Self::IncorrectMessageLengthOrInvalidFormat,
            0x14 => Self::ResponseTooLong,
            0x15..=0x20 => Self::ISOSAEReserved(value),
            0x21 => Self::BusyRepeatRequest,
            0x22 => Self::ConditionsNotCorrect,
            0x23 => Self::ISOSAEReserved(value),
            0x24 => Self::RequestSequenceError,
            0x25 => Self::NoResponseFromSubnetComponent,
            0x26 => Self::FailurePreventsExecutionOfRequestedAction,
            0x27..=0x30 => Self::ISOSAEReserved(value),
            0x31 => Self::RequestOutOfRange,
            0x32 => Self::ISOSAEReserved(value),
            0x33 => Self::SecurityAccessDenied,
            0x34 => Self::AuthenticationRequired,
            0x35 => Self::InvalidKey,
            0x36 => Self::ExceedNumberOfAttempts,
            0x37 => Self::RequiredTimeDelayNotExpired,
            0x38..=0x4F => Self::ExtendedDataLinkSecurityReserved(value),
            0x50..=0x6F => Self::ISOSAEReserved(value),
            0x70 => Self::UploadDownloadNotAccepted,
            0x71 => Self::TransferDataSuspended,
            0x72 => Self::GeneralProgrammingFailure,
            0x73 => Self::WrongBlockSequenceCounter,
            0x74..=0x77 => Self::ISOSAEReserved(value),
            0x78 => Self::RequestCorrectlyReceivedResponsePending,
            0x79..=0x7D => Self::ISOSAEReserved(value),
            0x7E => Self::SubFunctionNotSupportedInActiveSession,
            0x7F => Self::ServiceNotSupportedInActiveSession,
            0x80 => Self::ISOSAEReserved(value),
            0x81 => Self::RPMTooHigh,
            0x82 => Self::RPMTooLow,
            0x83 => Self::EngineIsRunning,
            0x84 => Self::EngineIsNotRunning,
            0x85 => Self::EngineRunTimeTooLow,
            0x86 => Self::TemperatureTooHigh,
            0x87 => Self::TemperatureTooLow,
            0x88 => Self::VehicleSpeedTooHigh,
            0x89 => Self::VehicleSpeedTooLow,
            0x8A => Self::ThrottleOrPedalTooHigh,
            0x8B => Self::ThrottleOrPedalTooLow,
            0x8C => Self::TransmissionRangeNotInNeutral,
            0x8D => Self::TransmissionRangeNotInGear,
            0x8E => Self::ISOSAEReserved(value),
            0x8F => Self::BrakeSwitchNotClosed,
            0x90 => Self::ShifterLeverNotInPark,
            0x91 => Self::TorqueConverterClutchLocked,
            0x92 => Self::VoltageTooHigh,
            0x93 => Self::VoltageTooLow,
            0x94 => Self::ResourceTemporarilyNotAvailable,
            0x95..=0xFE => Self::ReservedForSpecificConditionsNotMet(value),
            0xFF => Self::ISOSAEReserved(value),
        }
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nrc_from_u8() {
        let nrc = NegativeResponseCode::from(0x13);
        assert_eq!(
            nrc,
            NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat
        );
    }
    #[test]
    fn test_nrc_to_u8() {
        let nrc = NegativeResponseCode::RequestOutOfRange;
        let nrc_byte: u8 = nrc.into();
        assert_eq!(nrc_byte, 0x31);
    }
    #[test]
    fn test_iso_reserved_nrc_direct() {
        let nrc = NegativeResponseCode::ISOSAEReserved(0x94);
        let nrc_byte: u8 = nrc.into();
        assert_eq!(nrc_byte, 0x94);
        let check_same = NegativeResponseCode::from(0x94);
        assert_ne!(nrc, check_same);
    }
}
