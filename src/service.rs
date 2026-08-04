/// Identifies a UDS service (ISO 14229-1).
///
/// Without the `#[non_exhaustive]` annotation, adding additional diagnostic
/// commands would be a breaking semver change.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    ///
    /// Defined in ISO 14229-1:2013 and **removed in the 2020 edition**, which this crate
    /// otherwise targets. The variant is retained so a 2013-era `0x83`/`0xC3` byte round-trips
    /// as itself rather than becoming an unrecognized
    /// [`Request::Other`](crate::Request::Other); no request or response type models it.
    AccessTimingParameters,
    /// Transmit data using a security sub-layer (ISO 15764).
    SecuredDataTransmission,
    /// Enable or disable the detection of any or all errors.
    /// This is important when diagnostic work is performed in the car,
    /// which can cause an anomalous behavior of individual devices.
    ControlDtcSetting,
    /// Request the server to start or stop sending responses when a specified event occurs.
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
    /// Read the scaling information of a data record identified by a DID.
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
    DynamicallyDefineDataIdentifier,
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
    ReadDtcInfo,
    // ========================================================================
    // Input / Output Control
    /// Substitute or control an input/output signal using a DID.
    InputOutputControlByIdentifier,
    // ========================================================================
    // Remote Activation of Routine
    /// Start, stop, or request the results of a server-resident routine.
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
    /// A Negative Response Code will be included.
    NegativeResponse,
    /// While additional uds may exist, only the above are supported by this library
    UnsupportedDiagnosticService,
    // ========================================================================
}

impl UdsServiceType {
    /// Map a request-message service identifier (SID) byte to its [`UdsServiceType`].
    ///
    /// Unrecognised bytes map to [`UdsServiceType::UnsupportedDiagnosticService`].
    #[must_use]
    pub const fn from_request_sid(value: u8) -> Self {
        match value {
            0x10 => Self::DiagnosticSessionControl,
            0x11 => Self::EcuReset,
            0x27 => Self::SecurityAccess,
            0x28 => Self::CommunicationControl,
            0x29 => Self::Authentication,
            0x3E => Self::TesterPresent,
            0x83 => Self::AccessTimingParameters,
            0x84 => Self::SecuredDataTransmission,
            0x85 => Self::ControlDtcSetting,
            0x86 => Self::ResponseOnEvent,
            0x87 => Self::LinkControl,
            0x22 => Self::ReadDataByIdentifier,
            0x23 => Self::ReadMemoryByAddress,
            0x24 => Self::ReadScalingDataByIdentifier,
            0x2A => Self::ReadDataByIdentifierPeriodic,
            0x2C => Self::DynamicallyDefineDataIdentifier,
            0x2E => Self::WriteDataByIdentifier,
            0x3D => Self::WriteMemoryByAddress,
            0x14 => Self::ClearDiagnosticInfo,
            0x19 => Self::ReadDtcInfo,
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

    /// Return the request-message service identifier (SID) byte for this service type.
    ///
    /// # Caveat
    ///
    /// [`UdsServiceType::NegativeResponse`] and
    /// [`UdsServiceType::UnsupportedDiagnosticService`] have no request SID and both return
    /// `0x7F`, which is not a valid request SID. To re-encode an unmodeled request without
    /// loss, use [`Request::Other`](crate::Request::Other), which echoes the raw byte.
    #[must_use]
    pub const fn to_request_sid(self) -> u8 {
        match self {
            Self::DiagnosticSessionControl => 0x10,
            Self::EcuReset => 0x11,
            Self::SecurityAccess => 0x27,
            Self::CommunicationControl => 0x28,
            Self::Authentication => 0x29,
            Self::TesterPresent => 0x3E,
            Self::AccessTimingParameters => 0x83,
            Self::SecuredDataTransmission => 0x84,
            Self::ControlDtcSetting => 0x85,
            Self::ResponseOnEvent => 0x86,
            Self::LinkControl => 0x87,
            Self::ReadDataByIdentifier => 0x22,
            Self::ReadMemoryByAddress => 0x23,
            Self::ReadScalingDataByIdentifier => 0x24,
            Self::ReadDataByIdentifierPeriodic => 0x2A,
            Self::DynamicallyDefineDataIdentifier => 0x2C,
            Self::WriteDataByIdentifier => 0x2E,
            Self::WriteMemoryByAddress => 0x3D,
            Self::ClearDiagnosticInfo => 0x14,
            Self::ReadDtcInfo => 0x19,
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
    /// Map a positive-response service identifier (SID) byte to its [`UdsServiceType`].
    ///
    /// Unrecognised bytes map to [`UdsServiceType::UnsupportedDiagnosticService`].
    #[must_use]
    pub const fn from_response_sid(value: u8) -> Self {
        match value {
            0x50 => Self::DiagnosticSessionControl,
            0x51 => Self::EcuReset,
            0x67 => Self::SecurityAccess,
            0x68 => Self::CommunicationControl,
            0x69 => Self::Authentication,
            0x7E => Self::TesterPresent,
            0xC3 => Self::AccessTimingParameters,
            0xC4 => Self::SecuredDataTransmission,
            0xC5 => Self::ControlDtcSetting,
            0xC6 => Self::ResponseOnEvent,
            0xC7 => Self::LinkControl,
            0x62 => Self::ReadDataByIdentifier,
            0x63 => Self::ReadMemoryByAddress,
            0x64 => Self::ReadScalingDataByIdentifier,
            0x6A => Self::ReadDataByIdentifierPeriodic,
            0x6C => Self::DynamicallyDefineDataIdentifier,
            0x6E => Self::WriteDataByIdentifier,
            0x7D => Self::WriteMemoryByAddress,
            0x54 => Self::ClearDiagnosticInfo,
            0x59 => Self::ReadDtcInfo,
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

    /// Return the positive-response service identifier (SID) byte for this service type.
    ///
    /// [`UdsServiceType::UnsupportedDiagnosticService`] has no response SID and returns
    /// `0x7F`; see [`Response::Other`](crate::Response::Other) for lossless pass-through.
    #[must_use]
    pub const fn to_response_sid(self) -> u8 {
        match self {
            Self::DiagnosticSessionControl => 0x50,
            Self::EcuReset => 0x51,
            Self::SecurityAccess => 0x67,
            Self::CommunicationControl => 0x68,
            Self::Authentication => 0x69,
            Self::TesterPresent => 0x7E,
            Self::AccessTimingParameters => 0xC3,
            Self::SecuredDataTransmission => 0xC4,
            Self::ControlDtcSetting => 0xC5,
            Self::ResponseOnEvent => 0xC6,
            Self::LinkControl => 0xC7,
            Self::ReadDataByIdentifier => 0x62,
            Self::ReadMemoryByAddress => 0x63,
            Self::ReadScalingDataByIdentifier => 0x64,
            Self::ReadDataByIdentifierPeriodic => 0x6A,
            Self::DynamicallyDefineDataIdentifier => 0x6C,
            Self::WriteDataByIdentifier => 0x6E,
            Self::WriteMemoryByAddress => 0x7D,
            Self::ClearDiagnosticInfo => 0x54,
            Self::ReadDtcInfo => 0x59,
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

    /// Whether ISO 14229-1 gives this service a sub-function byte, and therefore whether its
    /// request can carry a suppressPosRspMsgIndicationBit (SPRMIB) in bit 7 of that byte.
    ///
    /// This is a fact about the standard rather than about this crate's coverage, so it is
    /// answered for all 26 services the 2020 edition defines and not only the 16 this crate
    /// models.
    ///
    /// # `None` means this crate cannot say
    ///
    /// Three variants have no answer, and `None` for them is a different report from
    /// `Some(false)` — keeping the two apart is the whole point of
    /// [`Request::is_positive_response_suppressed`](crate::Request::is_positive_response_suppressed)
    /// returning an `Option`:
    ///
    /// - [`UdsServiceType::UnsupportedDiagnosticService`], where every SID ISO 14229-1 does not
    ///   assign lands — including the vendor-specific range. This is the case that matters in
    ///   practice.
    /// - [`UdsServiceType::NegativeResponse`], which is not a request service at all.
    /// - [`UdsServiceType::AccessTimingParameters`]. The 2020 edition, which this crate targets,
    ///   withdrew the service; the variant is retained only so a 2013-era `0x83`/`0xC3` byte
    ///   round-trips as itself. The 2013 edition did give it a sub-function, but reporting that
    ///   here would state a fact from an edition the rest of this table is not drawn from, so
    ///   the answer is deferred to the caller.
    #[must_use]
    pub const fn has_sub_function(self) -> Option<bool> {
        match self {
            // Services whose first request byte is a sub-function, so bit 7 is the SPRMIB.
            Self::Authentication
            | Self::CommunicationControl
            | Self::ControlDtcSetting
            | Self::DiagnosticSessionControl
            | Self::DynamicallyDefineDataIdentifier
            | Self::EcuReset
            | Self::LinkControl
            | Self::ReadDtcInfo
            | Self::ResponseOnEvent
            | Self::RoutineControl
            | Self::SecurityAccess
            | Self::TesterPresent => Some(true),
            // Services with no sub-function. Several lead with an enumerated parameter
            // (`transmissionMode`, `modeOfOperation`, `controlOptionRecord`) that ISO does not
            // label a sub-function, so bit 7 of that byte is data rather than a SPRMIB.
            Self::ClearDiagnosticInfo
            | Self::InputOutputControlByIdentifier
            | Self::ReadDataByIdentifier
            | Self::ReadDataByIdentifierPeriodic
            | Self::ReadMemoryByAddress
            | Self::ReadScalingDataByIdentifier
            | Self::RequestDownload
            | Self::RequestFileTransfer
            | Self::RequestTransferExit
            | Self::RequestUpload
            | Self::SecuredDataTransmission
            | Self::TransferData
            | Self::WriteDataByIdentifier
            | Self::WriteMemoryByAddress => Some(false),
            // No answer: not a request service, or not a service in the 2020 edition.
            Self::AccessTimingParameters
            | Self::NegativeResponse
            | Self::UnsupportedDiagnosticService => None,
        }
    }
}

impl core::fmt::Display for UdsServiceType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Every service this crate models, paired with its request and response SID, and whether
    /// ISO 14229-1 gives it a sub-function byte.
    ///
    /// Written out rather than derived from the conversions, so the table is an independent
    /// statement of what ISO 14229-1 assigns and a transposed pair fails instead of agreeing
    /// with itself. `NegativeResponse` and `UnsupportedDiagnosticService` are absent because
    /// neither has a request SID; both are covered by
    /// `the_variants_that_are_not_a_2020_request_service_report_no_sub_function`.
    const SERVICES: &[(UdsServiceType, u8, u8, Option<bool>)] = &[
        (
            UdsServiceType::DiagnosticSessionControl,
            0x10,
            0x50,
            Some(true),
        ),
        (UdsServiceType::EcuReset, 0x11, 0x51, Some(true)),
        (UdsServiceType::ClearDiagnosticInfo, 0x14, 0x54, Some(false)),
        (UdsServiceType::ReadDtcInfo, 0x19, 0x59, Some(true)),
        (
            UdsServiceType::ReadDataByIdentifier,
            0x22,
            0x62,
            Some(false),
        ),
        (UdsServiceType::ReadMemoryByAddress, 0x23, 0x63, Some(false)),
        (
            UdsServiceType::ReadScalingDataByIdentifier,
            0x24,
            0x64,
            Some(false),
        ),
        (UdsServiceType::SecurityAccess, 0x27, 0x67, Some(true)),
        (UdsServiceType::CommunicationControl, 0x28, 0x68, Some(true)),
        (UdsServiceType::Authentication, 0x29, 0x69, Some(true)),
        (
            UdsServiceType::ReadDataByIdentifierPeriodic,
            0x2A,
            0x6A,
            Some(false),
        ),
        (
            UdsServiceType::DynamicallyDefineDataIdentifier,
            0x2C,
            0x6C,
            Some(true),
        ),
        (
            UdsServiceType::WriteDataByIdentifier,
            0x2E,
            0x6E,
            Some(false),
        ),
        (
            UdsServiceType::InputOutputControlByIdentifier,
            0x2F,
            0x6F,
            Some(false),
        ),
        (UdsServiceType::RoutineControl, 0x31, 0x71, Some(true)),
        (UdsServiceType::RequestDownload, 0x34, 0x74, Some(false)),
        (UdsServiceType::RequestUpload, 0x35, 0x75, Some(false)),
        (UdsServiceType::TransferData, 0x36, 0x76, Some(false)),
        (UdsServiceType::RequestTransferExit, 0x37, 0x77, Some(false)),
        (UdsServiceType::RequestFileTransfer, 0x38, 0x78, Some(false)),
        (UdsServiceType::TesterPresent, 0x3E, 0x7E, Some(true)),
        // Withdrawn in the 2020 edition; see `has_sub_function`.
        (UdsServiceType::AccessTimingParameters, 0x83, 0xC3, None),
        (
            UdsServiceType::SecuredDataTransmission,
            0x84,
            0xC4,
            Some(false),
        ),
        (UdsServiceType::ControlDtcSetting, 0x85, 0xC5, Some(true)),
        (UdsServiceType::ResponseOnEvent, 0x86, 0xC6, Some(true)),
        (UdsServiceType::LinkControl, 0x87, 0xC7, Some(true)),
        (
            UdsServiceType::WriteMemoryByAddress,
            0x3D,
            0x7D,
            Some(false),
        ),
    ];

    #[test]
    fn every_service_maps_to_the_sids_iso_assigns_it() {
        for &(service, request_sid, response_sid, _) in SERVICES {
            assert_eq!(
                UdsServiceType::to_request_sid(service),
                request_sid,
                "{service:?} request SID"
            );
            assert_eq!(
                UdsServiceType::to_response_sid(service),
                response_sid,
                "{service:?} response SID"
            );
            assert_eq!(
                UdsServiceType::from_request_sid(request_sid),
                service,
                "request SID {request_sid:#04X}"
            );
            assert_eq!(
                UdsServiceType::from_response_sid(response_sid),
                service,
                "response SID {response_sid:#04X}"
            );
        }
    }

    #[test]
    fn the_response_sid_is_the_request_sid_with_bit_6_set() {
        // ISO 14229-1 derives every positive response SID by adding 0x40 to the request SID.
        // Checking the rule as well as the table catches a single mistyped entry above, which a
        // table compared only against itself would not.
        for &(service, request_sid, response_sid, _) in SERVICES {
            assert_eq!(
                response_sid,
                request_sid + 0x40,
                "{service:?} breaks the +0x40 rule"
            );
        }
    }

    #[test]
    fn the_two_services_without_a_request_sid_say_so() {
        // Both have no request SID and return 0x7F, which is not a legal request SID -- it is the
        // negative-response SID. Callers needing lossless round-tripping of an unmodeled service
        // must use `Request::Other`, so this is pinned rather than left to be discovered.
        assert_eq!(
            UdsServiceType::to_request_sid(UdsServiceType::NegativeResponse),
            0x7F
        );
        assert_eq!(
            UdsServiceType::to_request_sid(UdsServiceType::UnsupportedDiagnosticService),
            0x7F
        );
        assert_eq!(
            UdsServiceType::from_response_sid(0x7F),
            UdsServiceType::NegativeResponse
        );
    }

    #[test]
    fn every_service_reports_the_sub_function_iso_gives_it() {
        // The fourth column is an independent statement of the standard, not something derived
        // from `has_sub_function` -- so a mistyped arm in the implementation fails here rather
        // than agreeing with itself. Only the 12 services with a sub-function can carry a
        // SPRMIB, which is what `Request::is_positive_response_suppressed` depends on.
        for &(service, request_sid, _, has_sub_function) in SERVICES {
            assert_eq!(
                service.has_sub_function(),
                has_sub_function,
                "{service:?} (request SID {request_sid:#04X})"
            );
        }
    }

    #[test]
    fn the_variants_that_are_not_a_2020_request_service_report_no_sub_function() {
        // None of these three is a request service in the edition this crate targets, so "does
        // it have a sub-function" has no answer.
        //
        // `UnsupportedDiagnosticService` is the one that matters most: it is where every SID ISO
        // does not assign lands, and returning `None` there is what lets a caller tell a
        // vendor-specific service apart from one that genuinely has no sub-function.
        assert_eq!(UdsServiceType::NegativeResponse.has_sub_function(), None);
        assert_eq!(
            UdsServiceType::UnsupportedDiagnosticService.has_sub_function(),
            None
        );
        assert_eq!(
            UdsServiceType::from_request_sid(0x40).has_sub_function(),
            None,
            "0x40 is unassigned"
        );
        // 0x83 is enumerated -- the variant exists so a 2013-era byte round-trips -- but the
        // 2020 edition withdrew the service, so this crate has no basis for an answer.
        assert_eq!(
            UdsServiceType::AccessTimingParameters.has_sub_function(),
            None
        );
    }

    #[test]
    fn every_unassigned_byte_classifies_as_unsupported() {
        // The conversions are total: no byte panics, and anything outside the table above lands
        // on `UnsupportedDiagnosticService` rather than being silently mapped to a real service.
        // Written with `any` rather than collecting, so this compiles without `alloc`.
        let is_request_sid = |b: u8| SERVICES.iter().any(|&(_, r, _, _)| r == b);
        let is_response_sid = |b: u8| b == 0x7F || SERVICES.iter().any(|&(_, _, s, _)| s == b);

        for byte in 0x00..=0xFFu8 {
            let from_request = UdsServiceType::from_request_sid(byte);
            if is_request_sid(byte) {
                assert_ne!(
                    from_request,
                    UdsServiceType::UnsupportedDiagnosticService,
                    "{byte:#04X} is an assigned request SID"
                );
            } else {
                assert_eq!(
                    from_request,
                    UdsServiceType::UnsupportedDiagnosticService,
                    "{byte:#04X} is unassigned and must not map to a service"
                );
            }

            let from_response = UdsServiceType::from_response_sid(byte);
            if !is_response_sid(byte) {
                assert_eq!(
                    from_response,
                    UdsServiceType::UnsupportedDiagnosticService,
                    "{byte:#04X} is unassigned and must not map to a service"
                );
            }
        }
    }
}
