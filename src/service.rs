#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
// Without the non-exhaustive annotation, adding additional diagnostic commands would be a breaking semver change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[repr(u8)]
pub enum UdsServiceType {
    // ========================================================================
    // Diagnostics and Communications Management
    /// UDS uses different session types which can be changed using "Diagnostic Session Control".
    /// Different services are available depending on which session is active.
    /// "Default Session" is the initial session type.
    ///  Other session types are defined, but may not be implemented depending on the device:
    ///
    /// - 0x01 "Default Session" used for basic uds
    /// - 0x02 "Programming Session" used to upload software.
    /// - 0x03 "Extended Diagnostic Session" used to unlock additional diagnostic functions, such as the adjustment of sensors.
    /// - 0x04 "Safety system diagnostic session" used to test all safety-critical diagnostic functions, such as airbag tests.
    ///
    /// In addition, there are reserved session identifiers that can be defined for vehicle manufacturers and vehicle suppliers specific use.
    DiagnosticSessionControl = Self::DIAGNOSTIC_SESSION_CONTROL,
    /// The service "ECU reset" is used to restart the control unit (ECU).
    /// Depending on the control unit hardware and implementation, different forms of reset can be used:
    ///
    /// - "Hard Reset" simulates a shutdown of the power supply.
    /// - "key off on Reset" simulates the drain and turn on the ignition with the key.
    /// - Soft Reset" allows the initialization of certain program units and their storage structures.
    ///
    /// Again, there are reserved values that can be defined for vehicle manufacturers and vehicle suppliers specific use.
    EcuReset = Self::ECU_RESET,
    /// Security check is available to enable the most security-critical uds.
    /// For this purpose a "Seed" is generated and sent to the client by the control unit.
    /// From this "Seed" the client has to compute a "Key" and send it back to the control unit to unlock the security-critical uds.
    SecurityAccess = Self::SECURITY_ACCESS,
    /// With this service, both the sending and receiving of messages can be turned off in the control unit.
    CommunicationControl = Self::COMMUNICATION_CONTROL,
    /// An update (2020) of the standard added this service to provide a standardized approach to more modern methods of authentication than are permitted by the Security Access (0x27) service,
    /// including bidirectional authentication with PKI-based Certificate Exchange.
    Authentication = Self::AUTHENTICATION,
    /// If no communication is exchanged with the client for a long time,
    /// the control unit automatically exits the current session and returns to the "Default Session".
    /// It might even go to sleep mode.
    /// This service is to signal to the device that the client is still present.
    TesterPresent = Self::TESTER_PRESENT,
    /// In the communication between the controllers and the client,
    /// certain timing must be preserved.
    /// If these timings are exceeded without a message being sent,
    /// it must be assumed that the connection was interrupted.
    /// These timings can be read and changed through this service.
    AccessTimingParameters = Self::ACCESS_TIMING_PARAMETERS,
    SecuredDataTransmission = Self::SECURED_DATA_TRANSMISSION,
    /// Enable or disable the detection of any or all errors.
    /// This is important when diagnostic work is performed in the car,
    /// which can cause an anomalous behavior of individual devices.
    ControlDTCSettings = Self::CONTROL_DTC_SETTINGS,
    ResponseOnEvent = Self::RESPONSE_ON_EVENT,
    /// The Service Link Control is used to set the baud rate of the diagnostic access.
    /// It is usually implemented only at the central gateway.
    LinkControl = Self::LINK_CONTROL,

    // ========================================================================
    // Data Transmission
    /// With this service, it is possible to retrieve one or more values of a control unit.
    /// This can be information of all kinds and of different lengths such as part numbers or the software version.
    ReadDataByIdentifier = Self::READ_DATA_BY_IDENTIFIER,
    /// Read data from the physical memory at the provided address.
    /// This function can be used by a testing tool to read the internal behavior of the software.
    ReadMemoryByAddress = Self::READ_MEMORY_BY_ADDRESS,
    ReadScalingDataByIdentifier = Self::READ_SCALING_DATA_BY_IDENTIFIER,
    /// With this service, values are sent periodically by a ecu.
    /// The values to be sent must only use the "Dynamically Defined Data Identifier".
    ReadDataByIdentifierPeriodic = Self::READ_DATA_BY_IDENTIFIER_PERIODIC,
    /// This service offers the possibility of a fix for a device specified Data Identifier (DID) pool to configure another Data Identifier.
    /// This is usually a combination of parts of different DIDs or simply a concatenation of complete DIDs.
    /// The requested data may be configured or grouped in the following manner:
    ///
    /// - Source DID, position, length (in bytes), Sub-Function Byte: defineByIdentifier
    /// - Memory address length (in bytes), Sub-Function Byte: defineByMemoryAddress
    /// - Combinations of the two above methods through multiple requests.
    DynamicallyDefinedDataIdentifier = Self::DYNAMICALLY_DEFINED_DATA_IDENTIFIER,
    /// Change the specified Data Identifier (DID) to the provided value
    WriteDataByIdentifier = Self::WRITE_DATA_BY_IDENTIFIER,
    /// Write information into the ECU at one or more contiguous memory locations.
    WriteMemoryByAddress = Self::WRITE_MEMORY_BY_ADDRESS,

    // ========================================================================
    // Stored Data Transmission
    /// Delete all stored Diagnostic Trouble Codes (DTC)
    ClearDiagnosticInfo = Self::CLEAR_DIAGNOSTIC_INFO,
    /// DTC stands for "Diagnostic Trouble Codes".
    /// Each DTC handled by the ECU is stored with its own code in the error memory.
    /// These DTCs can be read via this service.
    ///  In addition to the errors themselves,
    /// additional diagnostic information is stored.
    ReadDTCInfo = Self::READ_DTC_INFO,
    // ========================================================================
    // Input / Output Control
    InputOutputControlByIdentifier = Self::INPUT_OUTPUT_CONTROL_BY_IDENTIFIER,
    // ========================================================================
    // Remote Activation of Routine
    RoutineControl = Self::ROUTINE_CONTROL,
    // ========================================================================
    // Upload / Download
    /// Downloading new software or other data into the control unit is initiated using the "Request Download".
    /// Here, the location and size of the data is specified.
    /// In response, the controller specifies how large the data packets can be.
    RequestDownload = Self::REQUEST_DOWNLOAD,
    /// Request the transfer of data from the ECU to the tester.
    /// The location and size must be specified.
    /// The size of the data blocks are specified by the tester.
    RequestUpload = Self::REQUEST_UPLOAD,
    /// For the actual transmission of data, the service "Transfer Data" is used
    /// This service is used for both uploading and downloading data.
    /// The transfer direction is established in advance by the service "Request Download" or "Upload Request".
    /// This service should try to send packets at maximum length, as specified in previous uds.
    /// If the data set is larger than the maximum, the "Transfer Data" service must be used several times in succession until all data has arrived.
    TransferData = Self::TRANSFER_DATA,
    /// A data transmission can be 'completed' when using the "Transfer Exit" service.
    /// This service is used for comparison between the control unit and the tester.
    /// When it is running, a control unit can answer negatively on this request to stop a data transfer request.
    /// This will be used when the amount of data (set in "Request Download" or "Upload Request") has not been transferred.
    RequestTransferExit = Self::REQUEST_TRANSFER_EXIT,
    /// This service is used to initiate a file download from the client to the server or upload from the server to the client.
    /// Additionally information about the file system are available by this service.
    RequestFileTransfer = Self::REQUEST_FILE_TRANSFER,
    /// This response is given when a service request could not be performed,
    /// for example a request for an unsupported Data Identifier.
    /// for example a request for an unsupported Data Identifier.
    /// A Negative Response Code will be included.
    NegativeResponse = Self::NEGATIVE_RESPONSE,
    /// While additional uds may exist, only the above are supported by this library
    UnsupportedDiagnosticService = Self::UNSUPPORTED_DIAGNOSTIC_SERVICE,
    // ========================================================================
}

impl UdsServiceType {
    pub const DIAGNOSTIC_SESSION_CONTROL: u8 = 0x10;
    pub const ECU_RESET: u8 = 0x11;
    pub const SECURITY_ACCESS: u8 = 0x27;
    pub const COMMUNICATION_CONTROL: u8 = 0x28;
    pub const AUTHENTICATION: u8 = 0x29;
    pub const TESTER_PRESENT: u8 = 0x3E;
    pub const ACCESS_TIMING_PARAMETERS: u8 = 0x83;
    pub const SECURED_DATA_TRANSMISSION: u8 = 0x84;
    pub const CONTROL_DTC_SETTINGS: u8 = 0x85;
    pub const RESPONSE_ON_EVENT: u8 = 0x86;
    pub const LINK_CONTROL: u8 = 0x87;
    pub const READ_DATA_BY_IDENTIFIER: u8 = 0x22;
    pub const READ_MEMORY_BY_ADDRESS: u8 = 0x23;
    pub const READ_SCALING_DATA_BY_IDENTIFIER: u8 = 0x24;
    pub const READ_DATA_BY_IDENTIFIER_PERIODIC: u8 = 0x2A;
    pub const DYNAMICALLY_DEFINED_DATA_IDENTIFIER: u8 = 0x2C;
    pub const WRITE_DATA_BY_IDENTIFIER: u8 = 0x2E;
    pub const WRITE_MEMORY_BY_ADDRESS: u8 = 0x3D;
    pub const CLEAR_DIAGNOSTIC_INFO: u8 = 0x14;
    pub const READ_DTC_INFO: u8 = 0x19;
    pub const INPUT_OUTPUT_CONTROL_BY_IDENTIFIER: u8 = 0x2F;
    pub const ROUTINE_CONTROL: u8 = 0x31;
    pub const REQUEST_DOWNLOAD: u8 = 0x34;
    pub const REQUEST_UPLOAD: u8 = 0x35;
    pub const TRANSFER_DATA: u8 = 0x36;
    pub const REQUEST_TRANSFER_EXIT: u8 = 0x37;
    pub const REQUEST_FILE_TRANSFER: u8 = 0x38;
    pub const NEGATIVE_RESPONSE: u8 = 0x7F;
    pub const UNSUPPORTED_DIAGNOSTIC_SERVICE: u8 = 0xFF;
    pub const POSITIVE_RESPONSE_OFFSET: u8 = 0x40;

    pub const DIAGNOSTIC_SESSION_CONTROL_RESPONSE: u8 =
        Self::DIAGNOSTIC_SESSION_CONTROL + Self::POSITIVE_RESPONSE_OFFSET;
    pub const ECU_RESET_RESPONSE: u8 = Self::ECU_RESET + Self::POSITIVE_RESPONSE_OFFSET;
    pub const SECURITY_ACCESS_RESPONSE: u8 = Self::SECURITY_ACCESS + Self::POSITIVE_RESPONSE_OFFSET;
    pub const COMMUNICATION_CONTROL_RESPONSE: u8 =
        Self::COMMUNICATION_CONTROL + Self::POSITIVE_RESPONSE_OFFSET;
    pub const AUTHENTICATION_RESPONSE: u8 = Self::AUTHENTICATION + Self::POSITIVE_RESPONSE_OFFSET;
    pub const TESTER_PRESENT_RESPONSE: u8 = Self::TESTER_PRESENT + Self::POSITIVE_RESPONSE_OFFSET;
    pub const ACCESS_TIMING_PARAMETERS_RESPONSE: u8 =
        Self::ACCESS_TIMING_PARAMETERS + Self::POSITIVE_RESPONSE_OFFSET;
    pub const SECURED_DATA_TRANSMISSION_RESPONSE: u8 =
        Self::SECURED_DATA_TRANSMISSION + Self::POSITIVE_RESPONSE_OFFSET;
    pub const CONTROL_DTC_SETTINGS_RESPONSE: u8 =
        Self::CONTROL_DTC_SETTINGS + Self::POSITIVE_RESPONSE_OFFSET;
    pub const RESPONSE_ON_EVENT_RESPONSE: u8 =
        Self::RESPONSE_ON_EVENT + Self::POSITIVE_RESPONSE_OFFSET;
    pub const LINK_CONTROL_RESPONSE: u8 = Self::LINK_CONTROL + Self::POSITIVE_RESPONSE_OFFSET;
    pub const READ_DATA_BY_IDENTIFIER_RESPONSE: u8 =
        Self::READ_DATA_BY_IDENTIFIER + Self::POSITIVE_RESPONSE_OFFSET;
    pub const READ_MEMORY_BY_ADDRESS_RESPONSE: u8 =
        Self::READ_MEMORY_BY_ADDRESS + Self::POSITIVE_RESPONSE_OFFSET;
    pub const READ_SCALING_DATA_BY_IDENTIFIER_RESPONSE: u8 =
        Self::READ_SCALING_DATA_BY_IDENTIFIER + Self::POSITIVE_RESPONSE_OFFSET;
    pub const READ_DATA_BY_IDENTIFIER_PERIODIC_RESPONSE: u8 =
        Self::READ_DATA_BY_IDENTIFIER_PERIODIC + Self::POSITIVE_RESPONSE_OFFSET;
    pub const DYNAMICALLY_DEFINED_DATA_IDENTIFIER_RESPONSE: u8 =
        Self::DYNAMICALLY_DEFINED_DATA_IDENTIFIER + Self::POSITIVE_RESPONSE_OFFSET;
    pub const WRITE_DATA_BY_IDENTIFIER_RESPONSE: u8 =
        Self::WRITE_DATA_BY_IDENTIFIER + Self::POSITIVE_RESPONSE_OFFSET;
    pub const WRITE_MEMORY_BY_ADDRESS_RESPONSE: u8 =
        Self::WRITE_MEMORY_BY_ADDRESS + Self::POSITIVE_RESPONSE_OFFSET;
    pub const CLEAR_DIAGNOSTIC_INFO_RESPONSE: u8 =
        Self::CLEAR_DIAGNOSTIC_INFO + Self::POSITIVE_RESPONSE_OFFSET;
    pub const READ_DTC_INFO_RESPONSE: u8 = Self::READ_DTC_INFO + Self::POSITIVE_RESPONSE_OFFSET;
    pub const INPUT_OUTPUT_CONTROL_BY_IDENTIFIER_RESPONSE: u8 =
        Self::INPUT_OUTPUT_CONTROL_BY_IDENTIFIER + Self::POSITIVE_RESPONSE_OFFSET;
    pub const ROUTINE_CONTROL_RESPONSE: u8 = Self::ROUTINE_CONTROL + Self::POSITIVE_RESPONSE_OFFSET;
    pub const REQUEST_DOWNLOAD_RESPONSE: u8 =
        Self::REQUEST_DOWNLOAD + Self::POSITIVE_RESPONSE_OFFSET;
    pub const REQUEST_UPLOAD_RESPONSE: u8 = Self::REQUEST_UPLOAD + Self::POSITIVE_RESPONSE_OFFSET;
    pub const TRANSFER_DATA_RESPONSE: u8 = Self::TRANSFER_DATA + Self::POSITIVE_RESPONSE_OFFSET;
    pub const REQUEST_TRANSFER_EXIT_RESPONSE: u8 =
        Self::REQUEST_TRANSFER_EXIT + Self::POSITIVE_RESPONSE_OFFSET;
    pub const REQUEST_FILE_TRANSFER_RESPONSE: u8 =
        Self::REQUEST_FILE_TRANSFER + Self::POSITIVE_RESPONSE_OFFSET;

    #[must_use]
    pub fn service_from_request_byte(value: u8) -> Self {
        match value {
            Self::DIAGNOSTIC_SESSION_CONTROL => Self::DiagnosticSessionControl,
            Self::ECU_RESET => Self::EcuReset,
            Self::SECURITY_ACCESS => Self::SecurityAccess,
            Self::COMMUNICATION_CONTROL => Self::CommunicationControl,
            Self::AUTHENTICATION => Self::Authentication,
            Self::TESTER_PRESENT => Self::TesterPresent,
            Self::ACCESS_TIMING_PARAMETERS => Self::AccessTimingParameters,
            Self::SECURED_DATA_TRANSMISSION => Self::SecuredDataTransmission,
            Self::CONTROL_DTC_SETTINGS => Self::ControlDTCSettings,
            Self::RESPONSE_ON_EVENT => Self::ResponseOnEvent,
            Self::LINK_CONTROL => Self::LinkControl,
            Self::READ_DATA_BY_IDENTIFIER => Self::ReadDataByIdentifier,
            Self::READ_MEMORY_BY_ADDRESS => Self::ReadMemoryByAddress,
            Self::READ_SCALING_DATA_BY_IDENTIFIER => Self::ReadScalingDataByIdentifier,
            Self::READ_DATA_BY_IDENTIFIER_PERIODIC => Self::ReadDataByIdentifierPeriodic,
            Self::DYNAMICALLY_DEFINED_DATA_IDENTIFIER => Self::DynamicallyDefinedDataIdentifier,
            Self::WRITE_DATA_BY_IDENTIFIER => Self::WriteDataByIdentifier,
            Self::WRITE_MEMORY_BY_ADDRESS => Self::WriteMemoryByAddress,
            Self::CLEAR_DIAGNOSTIC_INFO => Self::ClearDiagnosticInfo,
            Self::READ_DTC_INFO => Self::ReadDTCInfo,
            Self::INPUT_OUTPUT_CONTROL_BY_IDENTIFIER => Self::InputOutputControlByIdentifier,
            Self::ROUTINE_CONTROL => Self::RoutineControl,
            Self::REQUEST_DOWNLOAD => Self::RequestDownload,
            Self::REQUEST_UPLOAD => Self::RequestUpload,
            Self::TRANSFER_DATA => Self::TransferData,
            Self::REQUEST_TRANSFER_EXIT => Self::RequestTransferExit,
            Self::REQUEST_FILE_TRANSFER => Self::RequestFileTransfer,
            _ => Self::UnsupportedDiagnosticService,
        }
    }

    #[must_use]
    pub fn request_service_to_byte(&self) -> u8 {
        match self {
            Self::DiagnosticSessionControl => Self::DIAGNOSTIC_SESSION_CONTROL,
            Self::EcuReset => Self::ECU_RESET,
            Self::SecurityAccess => Self::SECURITY_ACCESS,
            Self::CommunicationControl => Self::COMMUNICATION_CONTROL,
            Self::Authentication => Self::AUTHENTICATION,
            Self::TesterPresent => Self::TESTER_PRESENT,
            Self::AccessTimingParameters => Self::ACCESS_TIMING_PARAMETERS,
            Self::SecuredDataTransmission => Self::SECURED_DATA_TRANSMISSION,
            Self::ControlDTCSettings => Self::CONTROL_DTC_SETTINGS,
            Self::ResponseOnEvent => Self::RESPONSE_ON_EVENT,
            Self::LinkControl => Self::LINK_CONTROL,
            Self::ReadDataByIdentifier => Self::READ_DATA_BY_IDENTIFIER,
            Self::ReadMemoryByAddress => Self::READ_MEMORY_BY_ADDRESS,
            Self::ReadScalingDataByIdentifier => Self::READ_SCALING_DATA_BY_IDENTIFIER,
            Self::ReadDataByIdentifierPeriodic => Self::READ_DATA_BY_IDENTIFIER_PERIODIC,
            Self::DynamicallyDefinedDataIdentifier => Self::DYNAMICALLY_DEFINED_DATA_IDENTIFIER,
            Self::WriteDataByIdentifier => Self::WRITE_DATA_BY_IDENTIFIER,
            Self::WriteMemoryByAddress => Self::WRITE_MEMORY_BY_ADDRESS,
            Self::ClearDiagnosticInfo => Self::CLEAR_DIAGNOSTIC_INFO,
            Self::ReadDTCInfo => Self::READ_DTC_INFO,
            Self::InputOutputControlByIdentifier => Self::INPUT_OUTPUT_CONTROL_BY_IDENTIFIER,
            Self::RoutineControl => Self::ROUTINE_CONTROL,
            Self::RequestDownload => Self::REQUEST_DOWNLOAD,
            Self::RequestUpload => Self::REQUEST_UPLOAD,
            Self::TransferData => Self::TRANSFER_DATA,
            Self::RequestTransferExit => Self::REQUEST_TRANSFER_EXIT,
            Self::RequestFileTransfer => Self::REQUEST_FILE_TRANSFER,
            Self::UnsupportedDiagnosticService => Self::UNSUPPORTED_DIAGNOSTIC_SERVICE,
            _ => Self::NEGATIVE_RESPONSE,
        }
    }

    #[allow(clippy::match_same_arms)]
    #[must_use]
    pub fn response_from_byte(value: u8) -> Self {
        match value {
            Self::DIAGNOSTIC_SESSION_CONTROL_RESPONSE => Self::DiagnosticSessionControl,
            Self::ECU_RESET_RESPONSE => Self::EcuReset,
            Self::SECURITY_ACCESS_RESPONSE => Self::SecurityAccess,
            Self::COMMUNICATION_CONTROL_RESPONSE => Self::CommunicationControl,
            Self::AUTHENTICATION_RESPONSE => Self::Authentication,
            Self::TESTER_PRESENT_RESPONSE => Self::TesterPresent,
            Self::ACCESS_TIMING_PARAMETERS_RESPONSE => Self::AccessTimingParameters,
            Self::SECURED_DATA_TRANSMISSION_RESPONSE => Self::SecuredDataTransmission,
            Self::CONTROL_DTC_SETTINGS_RESPONSE => Self::ControlDTCSettings,
            Self::RESPONSE_ON_EVENT_RESPONSE => Self::ResponseOnEvent,
            Self::LINK_CONTROL_RESPONSE => Self::LinkControl,
            Self::READ_DATA_BY_IDENTIFIER_RESPONSE => Self::ReadDataByIdentifier,
            Self::READ_MEMORY_BY_ADDRESS_RESPONSE => Self::ReadMemoryByAddress,
            Self::READ_SCALING_DATA_BY_IDENTIFIER_RESPONSE => Self::ReadScalingDataByIdentifier,
            Self::READ_DATA_BY_IDENTIFIER_PERIODIC_RESPONSE => Self::ReadDataByIdentifierPeriodic,
            Self::DYNAMICALLY_DEFINED_DATA_IDENTIFIER_RESPONSE => {
                Self::DynamicallyDefinedDataIdentifier
            }
            Self::WRITE_DATA_BY_IDENTIFIER_RESPONSE => Self::WriteDataByIdentifier,
            Self::WRITE_MEMORY_BY_ADDRESS_RESPONSE => Self::WriteMemoryByAddress,
            Self::CLEAR_DIAGNOSTIC_INFO_RESPONSE => Self::ClearDiagnosticInfo,
            Self::READ_DTC_INFO_RESPONSE => Self::ReadDTCInfo,
            Self::INPUT_OUTPUT_CONTROL_BY_IDENTIFIER_RESPONSE => {
                Self::InputOutputControlByIdentifier
            }
            Self::ROUTINE_CONTROL_RESPONSE => Self::RoutineControl,
            Self::REQUEST_DOWNLOAD_RESPONSE => Self::RequestDownload,
            Self::REQUEST_UPLOAD_RESPONSE => Self::RequestUpload,
            Self::TRANSFER_DATA_RESPONSE => Self::TransferData,
            Self::REQUEST_TRANSFER_EXIT_RESPONSE => Self::RequestTransferExit,
            Self::REQUEST_FILE_TRANSFER_RESPONSE => Self::RequestFileTransfer,
            Self::NEGATIVE_RESPONSE => Self::NegativeResponse,
            Self::UNSUPPORTED_DIAGNOSTIC_SERVICE => Self::UnsupportedDiagnosticService,
            _ => Self::UnsupportedDiagnosticService,
        }
    }

    #[must_use]
    pub fn response_to_byte(self) -> u8 {
        match self {
            Self::DiagnosticSessionControl => Self::DIAGNOSTIC_SESSION_CONTROL_RESPONSE,
            Self::EcuReset => Self::ECU_RESET_RESPONSE,
            Self::SecurityAccess => Self::SECURITY_ACCESS_RESPONSE,
            Self::CommunicationControl => Self::COMMUNICATION_CONTROL_RESPONSE,
            Self::Authentication => Self::AUTHENTICATION_RESPONSE,
            Self::TesterPresent => Self::TESTER_PRESENT_RESPONSE,
            Self::AccessTimingParameters => Self::ACCESS_TIMING_PARAMETERS_RESPONSE,
            Self::SecuredDataTransmission => Self::SECURED_DATA_TRANSMISSION_RESPONSE,
            Self::ControlDTCSettings => Self::CONTROL_DTC_SETTINGS_RESPONSE,
            Self::ResponseOnEvent => Self::RESPONSE_ON_EVENT_RESPONSE,
            Self::LinkControl => Self::LINK_CONTROL_RESPONSE,
            Self::ReadDataByIdentifier => Self::READ_DATA_BY_IDENTIFIER_RESPONSE,
            Self::ReadMemoryByAddress => Self::READ_MEMORY_BY_ADDRESS_RESPONSE,
            Self::ReadScalingDataByIdentifier => Self::READ_SCALING_DATA_BY_IDENTIFIER_RESPONSE,
            Self::ReadDataByIdentifierPeriodic => Self::READ_DATA_BY_IDENTIFIER_PERIODIC_RESPONSE,
            Self::DynamicallyDefinedDataIdentifier => {
                Self::DYNAMICALLY_DEFINED_DATA_IDENTIFIER_RESPONSE
            }
            Self::WriteDataByIdentifier => Self::WRITE_DATA_BY_IDENTIFIER_RESPONSE,
            Self::WriteMemoryByAddress => Self::WRITE_MEMORY_BY_ADDRESS_RESPONSE,
            Self::ClearDiagnosticInfo => Self::CLEAR_DIAGNOSTIC_INFO_RESPONSE,
            Self::ReadDTCInfo => Self::READ_DTC_INFO_RESPONSE,
            Self::InputOutputControlByIdentifier => {
                Self::INPUT_OUTPUT_CONTROL_BY_IDENTIFIER_RESPONSE
            }
            Self::RoutineControl => Self::ROUTINE_CONTROL_RESPONSE,
            Self::RequestDownload => Self::REQUEST_DOWNLOAD_RESPONSE,
            Self::RequestUpload => Self::REQUEST_UPLOAD_RESPONSE,
            Self::TransferData => Self::TRANSFER_DATA_RESPONSE,
            Self::RequestTransferExit => Self::REQUEST_TRANSFER_EXIT_RESPONSE,
            Self::RequestFileTransfer => Self::REQUEST_FILE_TRANSFER_RESPONSE,
            _ => Self::NEGATIVE_RESPONSE,
        }
    }
}

impl std::fmt::Display for UdsServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
