use std::io::{Error, Read, Write};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub trait UdsService {
    fn get_service_type(&self) -> UdsServiceType;
    fn write<T: Write>(&self, writer: T) -> Result<usize, Error>;
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
// Without the non-exhaustive annotation, adding additional diagnostic commands would be a breaking semver change.
#[non_exhaustive]
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
    DiagnosticSessionControl,
    /// The service "ECU reset" is used to restart the control unit (ECU).
    /// Depending on the control unit hardware and implementation, different forms of reset can be used:
    ///
    /// - "Hard Reset" simulates a shutdown of the power supply.
    /// - "key off on Reset" simulates the drain and turn on the ignition with the key.
    /// - Soft Reset" allows the initialization of certain program units and their storage structures.
    ///
    /// Again, there are reserved values that can be defined for vehicle manufacturers and vehicle suppliers specific use.
    EcuReset,
    /// Security check is available to enable the most security-critical uds.
    /// For this purpose a "Seed" is generated and sent to the client by the control unit.
    /// From this "Seed" the client has to compute a "Key" and send it back to the control unit to unlock the security-critical uds.
    SecurityAccess,
    /// With this service, both the sending and receiving of messages can be turned off in the control unit.
    CommunicationControl,
    /// An update (2020) of the standard added this service to provide a standardized approach to more modern methods of authentication than are permitted by the Security Access (0x27) service,
    /// including bidirectional authentication with PKI-based Certificate Exchange.
    Authentication,
    /// If no communication is exchanged with the client for a long time,
    /// the control unit automatically exits the current session and returns to the "Default Session".
    /// It might even go to sleep mode.
    /// This service is to signal to the device that the client is still present.
    TesterPresent,
    /// In the communication between the controllers and the client,
    /// certain timing must be preserved.
    /// If these timings are exceeded without a message being sent,
    /// it must be assumed that the connection was interrupted.
    /// These timings can be read and changed through this service.
    AccessTimingParameters,
    SecuredDataTransmission,
    /// Enable or disable the detection of any or all errors.
    /// This is important when diagnostic work is performed in the car,
    /// which can cause an anomalous behavior of individual devices.
    ControlDTCSettings,
    ResponseOnEvent,
    /// The Service Link Control is used to set the baud rate of the diagnostic access.
    /// It is usually implemented only at the central gateway.
    LinkControl,

    // ========================================================================
    // Data Transmission
    /// With this service, it is possible to retrieve one or more values of a control unit.
    /// This can be information of all kinds and of different lengths such as part numbers or the software version.
    ReadDataByIdentifier,
    /// Read data from the physical memory at the provided address.
    /// This function can be used by a testing tool to read the internal behavior of the software.
    ReadMemoryByAddress,
    ReadScalingDataByIdentifier,
    /// With this service, values are sent periodically by a ecu.
    /// The values to be sent must only use the "Dynamically Defined Data Identifier".
    ReadDataByIdentifierPeriodic,
    /// This service offers the possibility of a fix for a device specified Data Identifier (DID) pool to configure another Data Identifier.
    /// This is usually a combination of parts of different DIDs or simply a concatenation of complete DIDs.
    /// The requested data may be configured or grouped in the following manner:
    ///
    /// - Source DID, position, length (in bytes), Sub-Function Byte: defineByIdentifier
    /// - Memory address length (in bytes), Sub-Function Byte: defineByMemoryAddress
    /// - Combinations of the two above methods through multiple requests.
    DynamicallyDefinedDataIdentifier,
    /// Change the specified Data Identifier (DID) to the provided value
    WriteDataByIdentifier,
    /// Write information into the ECU at one or more contiguous memory locations.
    WriteMemoryByAddress,

    // ========================================================================
    // Stored Data Transmission
    /// Delete all stored Diagnostic Trouble Codes (DTC)
    ClearDiagnosticInfo,
    /// DTC stands for "Diagnostic Trouble Codes".
    /// Each DTC handled by the ECU is stored with its own code in the error memory.
    /// These DTCs can be read via this service.
    ///  In addition to the errors themselves,
    /// additional diagnostic information is stored.
    ReadDTCInfo,
    // ========================================================================
    // Input / Output Control
    InputOutputControlByIdentifier,
    // ========================================================================
    // Remote Activation of Routine
    RoutineControl,
    // ========================================================================
    // Upload / Download
    /// Downloading new software or other data into the control unit is initiated using the "Request Download".
    /// Here, the location and size of the data is specified.
    /// In response, the controller specifies how large the data packets can be.
    RequestDownload,
    /// Request the transfer of data from the ECU to the tester.
    /// The location and size must be specified.
    /// The size of the data blocks are specified by the tester.
    RequestUpload,
    /// For the actual transmission of data, the service "Transfer Data" is used
    /// This service is used for both uploading and downloading data.
    /// The transfer direction is established in advance by the service "Request Download" or "Upload Request".
    /// This service should try to send packets at maximum length, as specified in previous uds.
    /// If the data set is larger than the maximum, the "Transfer Data" service must be used several times in succession until all data has arrived.
    TransferData,
    /// A data transmission can be 'completed' when using the "Transfer Exit" service.
    /// This service is used for comparison between the control unit and the tester.
    /// When it is running, a control unit can answer negatively on this request to stop a data transfer request.
    /// This will be used when the amount of data (set in "Request Download" or "Upload Request") has not been transferred.
    RequestTransferExit,
    /// This service is used to initiate a file download from the client to the server or upload from the server to the client.
    /// Additionally information about the file system are available by this service.
    RequestFileTransfer,
    /// This response is given when a service request could not be performed,
    /// for example a request for an unsupported Data Identifier.
    /// for example a request for an unsupported Data Identifier.
    /// A Negative Response Code will be included.
    NegativeResponse,
    /// While additional uds may exist, only the above are supported by this library
    UnsupportedDiagnosticService,
    // ========================================================================
}

impl UdsServiceType {
    pub fn request_from_byte(value: u8) -> Self {
        match value {
            0x10 => Self::DiagnosticSessionControl,
            0x11 => Self::EcuReset,
            0x27 => Self::SecurityAccess,
            0x28 => Self::CommunicationControl,
            0x29 => Self::Authentication,
            0x3E => Self::TesterPresent,
            0x83 => Self::AccessTimingParameters,
            0x84 => Self::SecuredDataTransmission,
            0x85 => Self::ControlDTCSettings,
            0x86 => Self::ResponseOnEvent,
            0x87 => Self::LinkControl,
            0x22 => Self::ReadDataByIdentifier,
            0x23 => Self::ReadMemoryByAddress,
            0x24 => Self::ReadScalingDataByIdentifier,
            0x2A => Self::ReadDataByIdentifierPeriodic,
            0x2C => Self::DynamicallyDefinedDataIdentifier,
            0x2E => Self::WriteDataByIdentifier,
            0x3D => Self::WriteMemoryByAddress,
            0x14 => Self::ClearDiagnosticInfo,
            0x19 => Self::ReadDTCInfo,
            0x2F => Self::InputOutputControlByIdentifier,
            0x31 => Self::RoutineControl,
            0x34 => Self::RequestDownload,
            0x35 => Self::RequestUpload,
            0x36 => Self::TransferData,
            0x37 => Self::RequestTransferExit,
            0x38 => Self::RequestFileTransfer,
            _ => Self::UnsupportedDiagnosticService,
        }
    }

    pub fn request_to_byte(&self) -> u8 {
        match self {
            Self::DiagnosticSessionControl => 0x10,
            Self::EcuReset => 0x11,
            Self::SecurityAccess => 0x27,
            Self::CommunicationControl => 0x28,
            Self::Authentication => 0x29,
            Self::TesterPresent => 0x3E,
            Self::AccessTimingParameters => 0x83,
            Self::SecuredDataTransmission => 0x84,
            Self::ControlDTCSettings => 0x85,
            Self::ResponseOnEvent => 0x86,
            Self::LinkControl => 0x87,
            Self::ReadDataByIdentifier => 0x22,
            Self::ReadMemoryByAddress => 0x23,
            Self::ReadScalingDataByIdentifier => 0x24,
            Self::ReadDataByIdentifierPeriodic => 0x2A,
            Self::DynamicallyDefinedDataIdentifier => 0x2C,
            Self::WriteDataByIdentifier => 0x2E,
            Self::WriteMemoryByAddress => 0x3D,
            Self::ClearDiagnosticInfo => 0x14,
            Self::ReadDTCInfo => 0x19,
            Self::InputOutputControlByIdentifier => 0x2F,
            Self::RoutineControl => 0x31,
            Self::RequestDownload => 0x34,
            Self::RequestUpload => 0x35,
            Self::TransferData => 0x36,
            Self::RequestTransferExit => 0x37,
            Self::RequestFileTransfer => 0x38,
            _ => 0x7F,
        }
    }
    pub fn response_from_byte(value: u8) -> Self {
        match value {
            0x50 => Self::DiagnosticSessionControl,
            0x51 => Self::EcuReset,
            0x67 => Self::SecurityAccess,
            0x68 => Self::CommunicationControl,
            0x69 => Self::Authentication,
            0x7E => Self::TesterPresent,
            0xC3 => Self::AccessTimingParameters,
            0xC4 => Self::SecuredDataTransmission,
            0xC5 => Self::ControlDTCSettings,
            0xC6 => Self::ResponseOnEvent,
            0xC7 => Self::LinkControl,
            0x62 => Self::ReadDataByIdentifier,
            0x63 => Self::ReadMemoryByAddress,
            0x64 => Self::ReadScalingDataByIdentifier,
            0x6A => Self::ReadDataByIdentifierPeriodic,
            0x6C => Self::DynamicallyDefinedDataIdentifier,
            0x6E => Self::WriteDataByIdentifier,
            0x7D => Self::WriteMemoryByAddress,
            0x54 => Self::ClearDiagnosticInfo,
            0x59 => Self::ReadDTCInfo,
            0x6F => Self::InputOutputControlByIdentifier,
            0x71 => Self::RoutineControl,
            0x74 => Self::RequestDownload,
            0x75 => Self::RequestUpload,
            0x76 => Self::TransferData,
            0x77 => Self::RequestTransferExit,
            0x78 => Self::RequestFileTransfer,
            0x7F => Self::NegativeResponse,
            _ => Self::UnsupportedDiagnosticService,
        }
    }

    pub fn response_to_byte(self) -> u8 {
        match self {
            Self::DiagnosticSessionControl => 0x50,
            Self::EcuReset => 0x51,
            Self::SecurityAccess => 0x67,
            Self::CommunicationControl => 0x68,
            Self::Authentication => 0x69,
            Self::TesterPresent => 0x7E,
            Self::AccessTimingParameters => 0xC3,
            Self::SecuredDataTransmission => 0xC4,
            Self::ControlDTCSettings => 0xC5,
            Self::ResponseOnEvent => 0xC6,
            Self::LinkControl => 0xC7,
            Self::ReadDataByIdentifier => 0x62,
            Self::ReadMemoryByAddress => 0x63,
            Self::ReadScalingDataByIdentifier => 0x64,
            Self::ReadDataByIdentifierPeriodic => 0x6A,
            Self::DynamicallyDefinedDataIdentifier => 0x6C,
            Self::WriteDataByIdentifier => 0x6E,
            Self::WriteMemoryByAddress => 0x7D,
            Self::ClearDiagnosticInfo => 0x54,
            Self::ReadDTCInfo => 0x59,
            Self::InputOutputControlByIdentifier => 0x6F,
            Self::RoutineControl => 0x71,
            Self::RequestDownload => 0x74,
            Self::RequestUpload => 0x75,
            Self::TransferData => 0x76,
            Self::RequestTransferExit => 0x77,
            Self::RequestFileTransfer => 0x78,
            _ => 0x7F,
        }
    }
}

impl std::fmt::Display for UdsServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
