/// `NegativeResponseCode` is a shared error mechanism
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum NegativeResponseCode {
    /// This response code shall not be used in a negative response message.
    /// This positiveResponse parameter value is reserved for server internal implementation
    PositiveResponse = Self::POSITIVE_RESPONSE,
    /// This range of values is reserved for future definition.
    #[cfg_attr(feature = "clap", clap(skip))]
    ISOSAEReserved(u8),
    /// This response code indicates that the requested action has been rejected by the server.
    /// The `GeneralReject` response code shall only be implemented in the server if none of the negative response codes meet the needs of the implementation.
    /// This response code shall not be used as a general replacement for other response codes defined.
    GeneralReject = Self::GENERAL_REJECT,
    /// This response code indicates that the requested action will not be taken because the server does not support the requested service.
    /// The server shall send this response code in the case where the client has sent a request message with a service identifier which is either unknown or not supported.
    /// This negative response code is not shown in the list of negative response codes to be supported for a diagnostic service,
    /// because this negative response code is not applicable for supported services.
    ServiceNotSupported = Self::SERVICE_NOT_SUPPORTED,
    /// This response code indicates that the requested action will not be taken because the server does not support the service specific parameters of the request message.
    /// The server shall send this response code in case the client has sent a request message with a known and supported service identifier but with "sub function“ which is either unknown or not supported.
    SubFunctionNotSupported = Self::SUB_FUNCTION_NOT_SUPPORTED,
    /// This response code indicates that the requested action will not be taken because the length of the received request message does not match the prescribed length for the specified service,
    /// or that the format of the parameters do not match the prescribed format for the specified service.
    IncorrectMessageLengthOrInvalidFormat = Self::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT,
    /// This response code shall be reported by the server if the response to be generated exceeds the maximum number of bytes available by the underlying network layer.
    ResponseTooLong = Self::RESPONSE_TOO_LONG,
    /// This response code indicates that the server is temporarily too busy to perform the requested operation.
    /// In this circumstance the client shall perform repetition of the request message or "try again later".
    /// The repetition of the request shall be delayed by a time specified in the respective implementation documents.
    BusyRepeatRequest = Self::BUSY_REPEAT_REQUEST,
    /// This response code indicates that the requested action will not be taken because the server prerequisite conditions are not met.
    ConditionsNotCorrect = Self::CONDITIONS_NOT_CORRECT,
    /// This response code indicates that the requested action will not be taken because the server expects a different sequence of request messages or message as sent by the client.
    /// This may occur when sequence sensitive requests are issued in the wrong order.
    RequestSequenceError = Self::REQUEST_SEQUENCE_ERROR,
    /// This response code indicates that the requested action will not be taken because the server has detected that the request message contains a parameter which attempts to substitute a value beyond its range of authority
    /// (e.g. attempting to substitute a data byte of 111 when the data is only defined to 100),
    /// or which attempts to access a dataIdentifier/routineIdentifer that is not supported or not supported in active session.
    /// This response code shall be implemented for all services which allow the client to read data,
    /// write data, or adjust functions by data in the server.
    RequestOutOfRange = Self::REQUEST_OUT_OF_RANGE,
    /// This response code indicates that the requested action will not be taken because the server's security strategy has not been satisfied by the client. The server shall send this response code if one of the following cases occur:
    /// - the test conditions of the server are not met,
    /// - the required message sequence e.g. `DiagnosticSessionControl`, securityAccess is not met,
    /// - the client has sent a request message which requires an unlocked server.
    ///
    /// Beside the mandatory use of this negative response code as specified in the applicable services within this standard,
    /// this negative response code can also be used for any case where security is required and is not yet granted to perform the required service.
    SecurityAccessDenied = Self::SECURITY_ACCESS_DENIED,
    /// This response code indicates that the requested action will not be taken because the client has insufficient rights based on its Authentication state.
    AuthenticationRequired = Self::AUTHENTICATION_REQUIRED,
    /// This response code indicates that the server has not given security access because the key sent by the client did not match with the key in the server's memory.
    /// This counts as an attempt to gain security.
    /// The server shall remain locked and increment its internal securityAccessFailed counter.
    InvalidKey = Self::INVALID_KEY,
    /// This response code indicates that the requested action will not be taken because the client has unsuccessfully attempted to gain security access more times than the server's security strategy will allow.
    ExceedNumberOfAttempts = Self::EXCEED_NUMBER_OF_ATTEMPTS,
    /// This response code indicates that the requested action will not be taken because the client's latest attempt to gain security access was initiated before the server's required timeout period had elapsed.
    RequiredTimeDelayNotExpired = Self::REQUIRED_TIME_DELAY_NOT_EXPIRED,
    /// Reserved by ISO 15764
    #[cfg_attr(feature = "clap", clap(skip))]
    ExtendedDataLinkSecurityReserved(u8),
    /// This response code indicates that an attempt to upload/download to a server's memory cannot be accomplished due to some fault conditions.
    UploadDownloadNotAccepted = Self::UPLOAD_DOWNLOAD_NOT_ACCEPTED,
    /// This response code indicates that a data transfer operation was halted due to some fault.
    /// The active transferData sequence shall be aborted.
    TransferDataSuspended = Self::TRANSFER_DATA_SUSPENDED,
    /// This response code indicates that the server detected an error when erasing or programming a memory location in the permanent memory device (e.g. Flash Memory).
    GeneralProgrammingFailure = Self::GENERAL_PROGRAMMING_FAILURE,
    /// This response code indicates that the server detected an error in the sequence of `BlockSequenceCounter` values.
    /// Note that the repetition of a `TransferDataRequest` message with a `BlockSequenceCounter` equal to the one included in the previous `TransferDataRequest` message shall be accepted by the server.
    WrongBlockSequenceCounter = Self::WRONG_BLOCK_SEQUENCE_COUNTER,
    /// This response code indicates that the server detected an error in the sequence of `BlockSequenceCounter` values.
    RequestCorrectlyReceivedResponsePending = Self::REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING,
    /// This response code indicates that the requested action will not be taken because the server does not support the requested sub-function in the session currently active.
    /// Within the programmingSession negative response code 0x12 (subFunctionNotSupported) may optionally be reported instead of negative response code 0x7F (subFunctionNotSupportedInActiveSession).
    /// This response code shall only be used when the requested sub-function is known to be supported in another session,
    /// otherwise response code 0x12 (subFunctionNotSupported) shall be used.
    /// This response code shall be supported by each diagnostic service with a sub-function parameter,
    /// if not otherwise stated in the data link specific implementation document,
    /// therefore it is not listed in the list of applicable response codes of the diagnostic services.
    SubFunctionNotSupportedInActiveSession = Self::SUB_FUNCTION_NOT_SUPPORTED_IN_ACTIVE_SESSION,
    /// This response code indicates that the requested action will not be taken because the server does not support the requested service in the session currently active.
    /// This response code shall only be used when the requested service is known to be supported in another session, otherwise response code 0x11 (serviceNotSupported) shall be used.
    /// This response code is in general supported by each diagnostic service,
    /// as not otherwise stated in the data link specific implementation document,
    /// therefore it is not listed in the list of applicable response codes of the diagnostic services.
    ServiceNotSupportedInActiveSession = Self::SERVICE_NOT_SUPPORTED_IN_ACTIVE_SESSION,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for RPM is not met (current RPM is above a pre-programmed maximum threshold).
    RPMTooHigh = Self::RPM_TOO_HIGH,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for RPM is not met (current RPM is below a pre-programmed minimum threshold).
    RPMTooLow = Self::RPM_TOO_LOW,
    /// This is required for those actuator tests which cannot be actuated while the Engine is running.
    /// This is different from RPM too high negative response and needs to be allowed.
    EngineIsRunning = Self::ENGINE_IS_RUNNING,
    /// This is required for those actuator tests which cannot be actuated unless the Engine is running.
    /// This is different from RPM too low negative response, and needs to be allowed.
    EngineIsNotRunning = Self::ENGINE_IS_NOT_RUNNING,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for engine run time is not met
    /// (current engine run time is below a preprogrammed limit).
    EngineRunTimeTooLow = Self::ENGINE_RUN_TIME_TOO_LOW,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for temperature is not met
    /// (current temperature is above a preprogrammed maximum threshold).
    TemperatureTooHigh = Self::TEMPERATURE_TOO_HIGH,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for temperature is not met
    /// (current temperature is below a preprogrammed minimum threshold).
    TemperatureTooLow = Self::TEMPERATURE_TOO_LOW,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for vehicle speed is not met
    /// (current VS is above a pre-programmed maximum threshold).
    VehicleSpeedTooHigh = Self::VEHICLE_SPEED_TOO_HIGH,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for vehicle speed is not met
    /// (current VS is below a pre-programmed minimum threshold).
    VehicleSpeedTooLow = Self::VEHICLE_SPEED_TOO_LOW,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for throttle/pedal position is not met
    /// (current TP/APP is above a preprogrammed maximum threshold).
    ThrottleOrPedalTooHigh = Self::THROTTLE_OR_PEDAL_TOO_HIGH,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for throttle/pedal position is not met
    /// (current TP/APP is below a preprogrammed minimum threshold).
    ThrottleOrPedalTooLow = Self::THROTTLE_OR_PEDAL_TOO_LOW,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for being in neutral is not met
    /// (current transmission range is not in neutral).
    TransmissionRangeNotInNeutral = Self::TRANSMISSION_RANGE_NOT_IN_NEUTRAL,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for being in gear is not met
    /// (current transmission range is not in gear).
    TransmissionRangeNotInGear = Self::TRANSMISSION_RANGE_NOT_IN_GEAR,
    /// For safety reasons, this is required for certain tests before it begins,
    /// and must be maintained for the entire duration of the test.
    BrakeSwitchNotClosed = Self::BRAKE_SWITCH_NOT_CLOSED,
    /// For safety reasons, this is required for certain tests before it begins,
    /// and must be maintained for the entire duration of the test.
    ShifterLeverNotInPark = Self::SHIFTER_LEVER_NOT_IN_PARK,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for torque converter clutch is not met
    /// (current TCC status above a preprogrammed limit or locked).
    TorqueConverterClutchLocked = Self::TORQUE_CONVERTER_CLUTCH_LOCKED,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for voltage at the primary pin of the server
    /// (ECU) is not met
    /// (current voltage is above a pre-programmed maximum threshold).
    VoltageTooHigh = Self::VOLTAGE_TOO_HIGH,
    /// This response code indicates that the requested action will not be taken because the server prerequisite condition for voltage at the primary pin of the server
    /// (ECU) is not met
    /// (current voltage is below a pre-programmed maximum threshold).
    VoltageTooLow = Self::VOLTAGE_TOO_LOW,
    /// This range of values is reserved for future definition.
    #[cfg_attr(feature = "clap", clap(skip))]
    ReservedForSpecificConditionsNotMet(u8),
}

impl NegativeResponseCode {
    pub const POSITIVE_RESPONSE: u8 = 0x00;
    pub const GENERAL_REJECT: u8 = 0x10;
    pub const SERVICE_NOT_SUPPORTED: u8 = 0x11;
    pub const SUB_FUNCTION_NOT_SUPPORTED: u8 = 0x12;
    pub const INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT: u8 = 0x13;
    pub const RESPONSE_TOO_LONG: u8 = 0x14;
    pub const BUSY_REPEAT_REQUEST: u8 = 0x21;
    pub const CONDITIONS_NOT_CORRECT: u8 = 0x22;
    pub const REQUEST_SEQUENCE_ERROR: u8 = 0x24;
    pub const REQUEST_OUT_OF_RANGE: u8 = 0x31;
    pub const SECURITY_ACCESS_DENIED: u8 = 0x33;
    pub const AUTHENTICATION_REQUIRED: u8 = 0x34;
    pub const INVALID_KEY: u8 = 0x35;
    pub const EXCEED_NUMBER_OF_ATTEMPTS: u8 = 0x36;
    pub const REQUIRED_TIME_DELAY_NOT_EXPIRED: u8 = 0x37;
    pub const UPLOAD_DOWNLOAD_NOT_ACCEPTED: u8 = 0x70;
    pub const TRANSFER_DATA_SUSPENDED: u8 = 0x71;
    pub const GENERAL_PROGRAMMING_FAILURE: u8 = 0x72;
    pub const WRONG_BLOCK_SEQUENCE_COUNTER: u8 = 0x73;
    pub const REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING: u8 = 0x78;
    pub const SUB_FUNCTION_NOT_SUPPORTED_IN_ACTIVE_SESSION: u8 = 0x7E;
    pub const SERVICE_NOT_SUPPORTED_IN_ACTIVE_SESSION: u8 = 0x7F;
    pub const RPM_TOO_HIGH: u8 = 0x81;
    pub const RPM_TOO_LOW: u8 = 0x82;
    pub const ENGINE_IS_RUNNING: u8 = 0x83;
    pub const ENGINE_IS_NOT_RUNNING: u8 = 0x84;
    pub const ENGINE_RUN_TIME_TOO_LOW: u8 = 0x85;
    pub const TEMPERATURE_TOO_HIGH: u8 = 0x86;
    pub const TEMPERATURE_TOO_LOW: u8 = 0x87;
    pub const VEHICLE_SPEED_TOO_HIGH: u8 = 0x88;
    pub const VEHICLE_SPEED_TOO_LOW: u8 = 0x89;
    pub const THROTTLE_OR_PEDAL_TOO_HIGH: u8 = 0x8A;
    pub const THROTTLE_OR_PEDAL_TOO_LOW: u8 = 0x8B;
    pub const TRANSMISSION_RANGE_NOT_IN_NEUTRAL: u8 = 0x8C;
    pub const TRANSMISSION_RANGE_NOT_IN_GEAR: u8 = 0x8D;
    pub const BRAKE_SWITCH_NOT_CLOSED: u8 = 0x8F;
    pub const SHIFTER_LEVER_NOT_IN_PARK: u8 = 0x90;
    pub const TORQUE_CONVERTER_CLUTCH_LOCKED: u8 = 0x91;
    pub const VOLTAGE_TOO_HIGH: u8 = 0x92;
    pub const VOLTAGE_TOO_LOW: u8 = 0x93;
}

impl From<NegativeResponseCode> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: NegativeResponseCode) -> Self {
        match value {
            NegativeResponseCode::PositiveResponse => NegativeResponseCode::POSITIVE_RESPONSE,
            NegativeResponseCode::ISOSAEReserved(value) => value,
            NegativeResponseCode::GeneralReject => NegativeResponseCode::GENERAL_REJECT,
            NegativeResponseCode::ServiceNotSupported => {
                NegativeResponseCode::SERVICE_NOT_SUPPORTED
            }
            NegativeResponseCode::SubFunctionNotSupported => {
                NegativeResponseCode::SUB_FUNCTION_NOT_SUPPORTED
            }
            NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat => {
                NegativeResponseCode::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT
            }
            NegativeResponseCode::ResponseTooLong => NegativeResponseCode::RESPONSE_TOO_LONG,
            NegativeResponseCode::BusyRepeatRequest => NegativeResponseCode::BUSY_REPEAT_REQUEST,
            NegativeResponseCode::ConditionsNotCorrect => {
                NegativeResponseCode::CONDITIONS_NOT_CORRECT
            }
            NegativeResponseCode::RequestSequenceError => {
                NegativeResponseCode::REQUEST_SEQUENCE_ERROR
            }
            NegativeResponseCode::RequestOutOfRange => NegativeResponseCode::REQUEST_OUT_OF_RANGE,
            NegativeResponseCode::SecurityAccessDenied => {
                NegativeResponseCode::SECURITY_ACCESS_DENIED
            }
            NegativeResponseCode::AuthenticationRequired => {
                NegativeResponseCode::AUTHENTICATION_REQUIRED
            }
            NegativeResponseCode::InvalidKey => NegativeResponseCode::INVALID_KEY,
            NegativeResponseCode::ExceedNumberOfAttempts => {
                NegativeResponseCode::EXCEED_NUMBER_OF_ATTEMPTS
            }
            NegativeResponseCode::RequiredTimeDelayNotExpired => {
                NegativeResponseCode::REQUIRED_TIME_DELAY_NOT_EXPIRED
            }
            NegativeResponseCode::ExtendedDataLinkSecurityReserved(value) => value,
            NegativeResponseCode::UploadDownloadNotAccepted => {
                NegativeResponseCode::UPLOAD_DOWNLOAD_NOT_ACCEPTED
            }
            NegativeResponseCode::TransferDataSuspended => {
                NegativeResponseCode::TRANSFER_DATA_SUSPENDED
            }
            NegativeResponseCode::GeneralProgrammingFailure => {
                NegativeResponseCode::GENERAL_PROGRAMMING_FAILURE
            }
            NegativeResponseCode::WrongBlockSequenceCounter => {
                NegativeResponseCode::WRONG_BLOCK_SEQUENCE_COUNTER
            }
            NegativeResponseCode::RequestCorrectlyReceivedResponsePending => {
                NegativeResponseCode::REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING
            }
            NegativeResponseCode::SubFunctionNotSupportedInActiveSession => {
                NegativeResponseCode::SUB_FUNCTION_NOT_SUPPORTED_IN_ACTIVE_SESSION
            }
            NegativeResponseCode::ServiceNotSupportedInActiveSession => {
                NegativeResponseCode::SERVICE_NOT_SUPPORTED_IN_ACTIVE_SESSION
            }
            NegativeResponseCode::RPMTooHigh => NegativeResponseCode::RPM_TOO_HIGH,
            NegativeResponseCode::RPMTooLow => NegativeResponseCode::RPM_TOO_LOW,
            NegativeResponseCode::EngineIsRunning => NegativeResponseCode::ENGINE_IS_RUNNING,
            NegativeResponseCode::EngineIsNotRunning => NegativeResponseCode::ENGINE_IS_NOT_RUNNING,
            NegativeResponseCode::EngineRunTimeTooLow => {
                NegativeResponseCode::ENGINE_RUN_TIME_TOO_LOW
            }
            NegativeResponseCode::TemperatureTooHigh => NegativeResponseCode::TEMPERATURE_TOO_HIGH,
            NegativeResponseCode::TemperatureTooLow => NegativeResponseCode::TEMPERATURE_TOO_LOW,
            NegativeResponseCode::VehicleSpeedTooHigh => {
                NegativeResponseCode::VEHICLE_SPEED_TOO_HIGH
            }
            NegativeResponseCode::VehicleSpeedTooLow => NegativeResponseCode::VEHICLE_SPEED_TOO_LOW,
            NegativeResponseCode::ThrottleOrPedalTooHigh => {
                NegativeResponseCode::THROTTLE_OR_PEDAL_TOO_HIGH
            }
            NegativeResponseCode::ThrottleOrPedalTooLow => {
                NegativeResponseCode::THROTTLE_OR_PEDAL_TOO_LOW
            }
            NegativeResponseCode::TransmissionRangeNotInNeutral => {
                NegativeResponseCode::TRANSMISSION_RANGE_NOT_IN_NEUTRAL
            }
            NegativeResponseCode::TransmissionRangeNotInGear => {
                NegativeResponseCode::TRANSMISSION_RANGE_NOT_IN_GEAR
            }
            NegativeResponseCode::BrakeSwitchNotClosed => {
                NegativeResponseCode::BRAKE_SWITCH_NOT_CLOSED
            }
            NegativeResponseCode::ShifterLeverNotInPark => {
                NegativeResponseCode::SHIFTER_LEVER_NOT_IN_PARK
            }
            NegativeResponseCode::TorqueConverterClutchLocked => {
                NegativeResponseCode::TORQUE_CONVERTER_CLUTCH_LOCKED
            }
            NegativeResponseCode::VoltageTooHigh => NegativeResponseCode::VOLTAGE_TOO_HIGH,
            NegativeResponseCode::VoltageTooLow => NegativeResponseCode::VOLTAGE_TOO_LOW,
            NegativeResponseCode::ReservedForSpecificConditionsNotMet(value) => value,
        }
    }
}

impl From<u8> for NegativeResponseCode {
    #[allow(clippy::match_same_arms)]
    fn from(value: u8) -> Self {
        match value {
            NegativeResponseCode::POSITIVE_RESPONSE => Self::PositiveResponse,
            0x01..=0x0F => Self::ISOSAEReserved(value),
            NegativeResponseCode::GENERAL_REJECT => Self::GeneralReject,
            NegativeResponseCode::SERVICE_NOT_SUPPORTED => Self::ServiceNotSupported,
            NegativeResponseCode::SUB_FUNCTION_NOT_SUPPORTED => Self::SubFunctionNotSupported,
            NegativeResponseCode::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT => {
                Self::IncorrectMessageLengthOrInvalidFormat
            }
            NegativeResponseCode::RESPONSE_TOO_LONG => Self::ResponseTooLong,
            0x15..=0x20 => Self::ISOSAEReserved(value),
            NegativeResponseCode::BUSY_REPEAT_REQUEST => Self::BusyRepeatRequest,
            NegativeResponseCode::CONDITIONS_NOT_CORRECT => Self::ConditionsNotCorrect,
            0x23 => Self::ISOSAEReserved(value),
            NegativeResponseCode::REQUEST_SEQUENCE_ERROR => Self::RequestSequenceError,
            0x25..=0x30 => Self::ISOSAEReserved(value),
            NegativeResponseCode::REQUEST_OUT_OF_RANGE => Self::RequestOutOfRange,
            0x32 => Self::ISOSAEReserved(value),
            NegativeResponseCode::SECURITY_ACCESS_DENIED => Self::SecurityAccessDenied,
            NegativeResponseCode::AUTHENTICATION_REQUIRED => Self::AuthenticationRequired,
            NegativeResponseCode::INVALID_KEY => Self::InvalidKey,
            NegativeResponseCode::EXCEED_NUMBER_OF_ATTEMPTS => Self::ExceedNumberOfAttempts,
            NegativeResponseCode::REQUIRED_TIME_DELAY_NOT_EXPIRED => {
                Self::RequiredTimeDelayNotExpired
            }
            0x38..=0x4F => Self::ExtendedDataLinkSecurityReserved(value),
            0x50..=0x6F => Self::ISOSAEReserved(value),
            NegativeResponseCode::UPLOAD_DOWNLOAD_NOT_ACCEPTED => Self::UploadDownloadNotAccepted,
            NegativeResponseCode::TRANSFER_DATA_SUSPENDED => Self::TransferDataSuspended,
            NegativeResponseCode::GENERAL_PROGRAMMING_FAILURE => Self::GeneralProgrammingFailure,
            NegativeResponseCode::WRONG_BLOCK_SEQUENCE_COUNTER => Self::WrongBlockSequenceCounter,
            0x74..=0x77 => Self::ISOSAEReserved(value),
            NegativeResponseCode::REQUEST_CORRECTLY_RECEIVED_RESPONSE_PENDING => {
                Self::RequestCorrectlyReceivedResponsePending
            }
            0x79..=0x7D => Self::ISOSAEReserved(value),
            NegativeResponseCode::SUB_FUNCTION_NOT_SUPPORTED_IN_ACTIVE_SESSION => {
                Self::SubFunctionNotSupportedInActiveSession
            }
            NegativeResponseCode::SERVICE_NOT_SUPPORTED_IN_ACTIVE_SESSION => {
                Self::ServiceNotSupportedInActiveSession
            }
            0x80 => Self::ISOSAEReserved(value),
            NegativeResponseCode::RPM_TOO_HIGH => Self::RPMTooHigh,
            NegativeResponseCode::RPM_TOO_LOW => Self::RPMTooLow,
            NegativeResponseCode::ENGINE_IS_RUNNING => Self::EngineIsRunning,
            NegativeResponseCode::ENGINE_IS_NOT_RUNNING => Self::EngineIsNotRunning,
            NegativeResponseCode::ENGINE_RUN_TIME_TOO_LOW => Self::EngineRunTimeTooLow,
            NegativeResponseCode::TEMPERATURE_TOO_HIGH => Self::TemperatureTooHigh,
            NegativeResponseCode::TEMPERATURE_TOO_LOW => Self::TemperatureTooLow,
            NegativeResponseCode::VEHICLE_SPEED_TOO_HIGH => Self::VehicleSpeedTooHigh,
            NegativeResponseCode::VEHICLE_SPEED_TOO_LOW => Self::VehicleSpeedTooLow,
            NegativeResponseCode::THROTTLE_OR_PEDAL_TOO_HIGH => Self::ThrottleOrPedalTooHigh,
            NegativeResponseCode::THROTTLE_OR_PEDAL_TOO_LOW => Self::ThrottleOrPedalTooLow,
            NegativeResponseCode::TRANSMISSION_RANGE_NOT_IN_NEUTRAL => {
                Self::TransmissionRangeNotInNeutral
            }
            NegativeResponseCode::TRANSMISSION_RANGE_NOT_IN_GEAR => {
                Self::TransmissionRangeNotInGear
            }
            0x8E => Self::ISOSAEReserved(value),
            NegativeResponseCode::BRAKE_SWITCH_NOT_CLOSED => Self::BrakeSwitchNotClosed,
            NegativeResponseCode::SHIFTER_LEVER_NOT_IN_PARK => Self::ShifterLeverNotInPark,
            NegativeResponseCode::TORQUE_CONVERTER_CLUTCH_LOCKED => {
                Self::TorqueConverterClutchLocked
            }
            NegativeResponseCode::VOLTAGE_TOO_HIGH => Self::VoltageTooHigh,
            NegativeResponseCode::VOLTAGE_TOO_LOW => Self::VoltageTooLow,
            0x94..=0xFE => Self::ReservedForSpecificConditionsNotMet(value),
            0xFF => Self::ISOSAEReserved(value),
        }
    }
}
