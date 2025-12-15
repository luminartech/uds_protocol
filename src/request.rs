//! Module for making and handling UDS Requests
use crate::{
    DiagnosticDefinition, Error, NegativeResponseCode, ReadDTCInfoRequest, ResetType,
    SecurityAccessType, SingleValueWireFormat, WireFormat,
    services::{
        ClearDiagnosticInfoRequest, CommunicationControlRequest, ControlDTCSettingsRequest,
        DiagnosticSessionControlRequest, EcuResetRequest, ReadDataByIdentifierRequest,
        RequestDownloadRequest, RoutineControlRequest, SecurityAccessRequest, TesterPresentRequest,
        TransferDataRequest, WriteDataByIdentifierRequest,
    },
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use super::{
    CommunicationControlType, CommunicationType, DTCRecord, DataFormatIdentifier,
    DiagnosticSessionType, DtcSettings, ReadDTCInfoSubFunction, RoutineControlSubFunction,
    service::UdsServiceType,
};

/// UDS Request types
/// Each variant corresponds to a request for a different UDS service
/// The variants contain all request data for each service
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq)]
pub enum Request<D: DiagnosticDefinition> {
    ClearDiagnosticInfo(ClearDiagnosticInfoRequest),
    CommunicationControl(CommunicationControlRequest),
    ControlDTCSettings(ControlDTCSettingsRequest),
    DiagnosticSessionControl(DiagnosticSessionControlRequest),
    EcuReset(EcuResetRequest),
    ReadDataByIdentifier(ReadDataByIdentifierRequest<D::DID>),
    ReadDTCInfo(ReadDTCInfoRequest),
    RequestDownload(RequestDownloadRequest),
    RequestTransferExit,
    RoutineControl(RoutineControlRequest<D::RID, D::RoutinePayload>),
    SecurityAccess(SecurityAccessRequest),
    TesterPresent(TesterPresentRequest),
    TransferData(TransferDataRequest),
    WriteDataByIdentifier(WriteDataByIdentifierRequest<D::DiagnosticPayload>),
}

impl<D: DiagnosticDefinition> Request<D> {
    /// Create a `ClearDiagnosticInfo` request, clears diagnostic information in one or more servers' memory
    #[must_use]
    pub fn clear_diagnostic_info(group_of_dtc: DTCRecord, memory_selection: u8) -> Self {
        Request::ClearDiagnosticInfo(ClearDiagnosticInfoRequest::new(
            group_of_dtc,
            memory_selection,
        ))
    }
    /// Create a `ClearDiagnosticInfo` request that clears all DTC information in one or more servers' memory
    #[must_use]
    pub fn clear_all_dtc_info(memory_selection: u8) -> Self {
        Request::ClearDiagnosticInfo(ClearDiagnosticInfoRequest::clear_all(memory_selection))
    }

    /// Create a `CommunicationControlRequest` with standard address information.
    ///
    /// # Panics
    ///
    ///  Panics if one of the extended address control types is passed.
    #[must_use]
    pub fn communication_control(
        communication_enable: CommunicationControlType,
        communication_type: CommunicationType,
        suppress_response: bool,
    ) -> Self {
        Request::CommunicationControl(CommunicationControlRequest::new(
            suppress_response,
            communication_enable,
            communication_type,
        ))
    }

    /// Create a `CommunicationControl` request with extended address information.
    /// This is used for the `EnableRxAndDisableTxWithEnhancedAddressInfo` and
    /// `EnableRxAndTxWithEnhancedAddressInfo` communication control types.
    ///
    /// # Panics
    ///
    /// Panics if one of the standard address control types is passed.
    #[must_use]
    pub fn communication_control_with_node_id(
        communication_enable: CommunicationControlType,
        communication_type: CommunicationType,
        node_id: u16,
        suppress_response: bool,
    ) -> Self {
        Request::CommunicationControl(CommunicationControlRequest::new_with_node_id(
            suppress_response,
            communication_enable,
            communication_type,
            node_id,
        ))
    }

    /// Create a new `ControlDTCSettings` request
    #[must_use]
    pub fn control_dtc_settings(setting: DtcSettings, suppress_response: bool) -> Self {
        Request::ControlDTCSettings(ControlDTCSettingsRequest::new(setting, suppress_response))
    }

    /// Create a new `DiagnosticSessionControl` request
    #[must_use]
    pub fn diagnostic_session_control(
        suppress_positive_response: bool,
        session_type: DiagnosticSessionType,
    ) -> Self {
        Request::DiagnosticSessionControl(DiagnosticSessionControlRequest::new(
            suppress_positive_response,
            session_type,
        ))
    }

    /// Create a new `EcuReset` request
    #[must_use]
    pub fn ecu_reset(suppress_positive_response: bool, reset_type: ResetType) -> Self {
        Request::EcuReset(EcuResetRequest::new(suppress_positive_response, reset_type))
    }

    /// Create a new `ReadDataByIdentifier` request
    pub fn read_data_by_identifier<I>(dids: I) -> Self
    where
        I: IntoIterator<Item = D::DID>,
    {
        Request::ReadDataByIdentifier(ReadDataByIdentifierRequest::new(dids))
    }

    #[must_use]
    pub fn read_dtc_information(sub_function: ReadDTCInfoSubFunction) -> Self {
        Request::ReadDTCInfo(ReadDTCInfoRequest::new(sub_function))
    }

    /// Create a new `RequestDownload` request
    ///     `encryption_method`: vehicle manufacturer specific (0x0 for no encryption)
    ///     `compression_method`: vehicle manufacturer specific (0x0 for no compression)
    ///     `memory_address`: the address in memory to start downloading from (Maximum 40 bits - 1024GB)
    ///     `memory_size`: the size of the memory to download (Max 4GB)
    ///
    /// # Errors
    /// Will generate an error of type `Error::InvalidEncryptionCompressionMethod()`.
    /// Generated when `compression_method` or `encryption_method` > 0x15
    pub fn request_download(
        encryption_method: u8,
        compression_method: u8,
        memory_address: u64,
        memory_size: u32,
    ) -> Result<Self, Error> {
        let data_format_identifier =
            DataFormatIdentifier::new(compression_method, encryption_method)?;

        Ok(Request::RequestDownload(RequestDownloadRequest::new(
            data_format_identifier,
            memory_address,
            memory_size,
        )?))
    }

    #[must_use]
    pub fn request_transfer_exit() -> Self {
        Self::RequestTransferExit
    }

    /// Create a new `RoutineControl` request with no payload
    ///
    /// **Note**: This does not check if the server requires a payload to perform the routine
    /// # Parameters:
    ///    * `sub_function`: The type of routine control to perform.
    ///      * [`RoutineControlSubFunction::StartRoutine`]
    ///      * [`RoutineControlSubFunction::StopRoutine`]
    ///      * [`RoutineControlSubFunction::RequestRoutineResults`]
    ///    * `routine_id`: The identifier of the routine to control
    pub fn routine_control(sub_function: RoutineControlSubFunction, routine_id: D::RID) -> Self {
        Request::RoutineControl(RoutineControlRequest::new(sub_function, routine_id, None))
    }

    /// Create a new `RoutineControl` request
    ///
    /// **Note**: This could be cleaner as the Identifier is technically represented in the `RoutinePayload`
    /// and if the `RoutinePayload` is a single value, then the `RoutineIdentifier` is not needed
    ///
    /// This does not check if the server requires a payload
    ///
    /// # Parameters:
    ///    * `sub_function`: The type of routine control to perform.
    ///      * [`RoutineControlSubFunction::StartRoutine`]
    ///      * [`RoutineControlSubFunction::StopRoutine`]
    ///      * [`RoutineControlSubFunction::RequestRoutineResults`]
    ///    * `routine_id`: The identifier of the routine to control. User defined routine identifiers and payloads are allowed
    ///      * General purpose/UDS defined: [`crate::UDSRoutineIdentifier`]
    ///    * `data`: Optional payload for the routine control request
    pub fn routine_control_payload(
        sub_function: RoutineControlSubFunction,
        routine_id: D::RID,
        data: Option<D::RoutinePayload>,
    ) -> Self {
        Request::RoutineControl(RoutineControlRequest::new(sub_function, routine_id, data))
    }

    #[must_use]
    pub fn security_access(
        suppress_positive_response: bool,
        access_type: SecurityAccessType,
        data_record: Vec<u8>,
    ) -> Self {
        Request::SecurityAccess(SecurityAccessRequest::new(
            suppress_positive_response,
            access_type,
            data_record,
        ))
    }

    #[must_use]
    pub fn tester_present(suppress_positive_response: bool) -> Self {
        Request::TesterPresent(TesterPresentRequest::new(suppress_positive_response))
    }

    #[must_use]
    pub fn transfer_data(sequence: u8, data: Vec<u8>) -> Self {
        Request::TransferData(TransferDataRequest::new(sequence, data))
    }

    pub fn write_data_by_identifier(payload: D::DiagnosticPayload) -> Self {
        Request::WriteDataByIdentifier(WriteDataByIdentifierRequest::new(payload))
    }

    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::ClearDiagnosticInfo(_) => UdsServiceType::ClearDiagnosticInfo,
            Self::CommunicationControl(_) => UdsServiceType::CommunicationControl,
            Self::ControlDTCSettings(_) => UdsServiceType::ControlDTCSettings,
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::ReadDataByIdentifier(_) => UdsServiceType::ReadDataByIdentifier,
            Self::ReadDTCInfo(_) => UdsServiceType::ReadDTCInfo,
            Self::RequestDownload(_) => UdsServiceType::RequestDownload,
            Self::RequestTransferExit => UdsServiceType::RequestTransferExit,
            Self::RoutineControl(_) => UdsServiceType::RoutineControl,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent(_) => UdsServiceType::TesterPresent,
            Self::TransferData(_) => UdsServiceType::TransferData,
            Self::WriteDataByIdentifier(_) => UdsServiceType::WriteDataByIdentifier,
        }
    }

    pub fn allowed_nack_codes(&self) -> &'static [NegativeResponseCode] {
        match self {
            Self::ClearDiagnosticInfo(_) => ClearDiagnosticInfoRequest::allowed_nack_codes(),
            Self::DiagnosticSessionControl(_) => {
                DiagnosticSessionControlRequest::allowed_nack_codes()
            }
            Self::EcuReset(_) => EcuResetRequest::allowed_nack_codes(),
            Self::SecurityAccess(_) => SecurityAccessRequest::allowed_nack_codes(),
            Self::RequestDownload(_) => RequestDownloadRequest::allowed_nack_codes(),
            _ => &[NegativeResponseCode::ServiceNotSupported],
        }
    }
}

impl<T: DiagnosticDefinition> WireFormat for Request<T> {
    /// Deserialization function to read a [`Request`] from a [`Reader`](std::io::Read)
    /// This function reads the service byte and then calls the appropriate
    /// deserialization function for the service in question
    ///
    /// *Note*:
    ///
    /// Some services allow for custom byte arrays at the end of the request
    /// It is important that only the request data is passed to this function
    /// or the deserialization could read unexpected data
    #[allow(clippy::too_many_lines)]
    fn decode<R: Read>(reader: &mut R) -> Result<Option<Self>, Error> {
        let service = UdsServiceType::service_from_request_byte(reader.read_u8()?);
        Ok(Some(match service {
            UdsServiceType::CommunicationControl => Self::CommunicationControl(
                CommunicationControlRequest::decode_single_value(reader)?,
            ),
            UdsServiceType::ControlDTCSettings => {
                Self::ControlDTCSettings(ControlDTCSettingsRequest::decode_single_value(reader)?)
            }
            UdsServiceType::DiagnosticSessionControl => Self::DiagnosticSessionControl(
                DiagnosticSessionControlRequest::decode_single_value(reader)?,
            ),
            UdsServiceType::EcuReset => {
                Self::EcuReset(EcuResetRequest::decode_single_value(reader)?)
            }
            UdsServiceType::ReadDataByIdentifier => Self::ReadDataByIdentifier(
                ReadDataByIdentifierRequest::decode_single_value(reader)?,
            ),
            UdsServiceType::ReadDTCInfo => {
                Self::ReadDTCInfo(ReadDTCInfoRequest::decode_single_value(reader)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(RequestDownloadRequest::decode_single_value(reader)?)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::RoutineControl => {
                Self::RoutineControl(RoutineControlRequest::decode_single_value(reader)?)
            }
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(SecurityAccessRequest::decode_single_value(reader)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(TesterPresentRequest::decode_single_value(reader)?)
            }
            UdsServiceType::TransferData => {
                Self::TransferData(TransferDataRequest::decode_single_value(reader)?)
            }
            UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(
                WriteDataByIdentifierRequest::decode_single_value(reader)?,
            ),
            UdsServiceType::Authentication => {
                return Err(Error::ServiceNotImplemented(UdsServiceType::Authentication));
            }
            UdsServiceType::AccessTimingParameters => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::AccessTimingParameters,
                ));
            }
            UdsServiceType::SecuredDataTransmission => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::SecuredDataTransmission,
                ));
            }
            UdsServiceType::ResponseOnEvent => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ResponseOnEvent,
                ));
            }
            UdsServiceType::LinkControl => {
                return Err(Error::ServiceNotImplemented(UdsServiceType::LinkControl));
            }
            UdsServiceType::ReadMemoryByAddress => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ReadMemoryByAddress,
                ));
            }
            UdsServiceType::ReadScalingDataByIdentifier => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ReadScalingDataByIdentifier,
                ));
            }
            UdsServiceType::ReadDataByIdentifierPeriodic => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ReadDataByIdentifierPeriodic,
                ));
            }
            UdsServiceType::DynamicallyDefinedDataIdentifier => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::DynamicallyDefinedDataIdentifier,
                ));
            }
            UdsServiceType::WriteMemoryByAddress => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::WriteMemoryByAddress,
                ));
            }
            UdsServiceType::ClearDiagnosticInfo => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::ClearDiagnosticInfo,
                ));
            }
            UdsServiceType::InputOutputControlByIdentifier => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::InputOutputControlByIdentifier,
                ));
            }
            UdsServiceType::RequestUpload => {
                return Err(Error::ServiceNotImplemented(UdsServiceType::RequestUpload));
            }
            UdsServiceType::RequestFileTransfer => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::RequestFileTransfer,
                ));
            }
            UdsServiceType::NegativeResponse => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::NegativeResponse,
                ));
            }
            UdsServiceType::UnsupportedDiagnosticService => {
                return Err(Error::ServiceNotImplemented(
                    UdsServiceType::UnsupportedDiagnosticService,
                ));
            }
        }))
    }

    fn required_size(&self) -> usize {
        1 + match self {
            Self::ClearDiagnosticInfo(cdi) => cdi.required_size(),
            Self::CommunicationControl(cc) => cc.required_size(),
            Self::ControlDTCSettings(ct) => ct.required_size(),
            Self::DiagnosticSessionControl(ds) => ds.required_size(),
            Self::EcuReset(er) => er.required_size(),
            Self::ReadDataByIdentifier(rd) => rd.required_size(),
            Self::ReadDTCInfo(rd) => rd.required_size(),
            Self::RequestDownload(rd) => rd.required_size(),
            Self::RequestTransferExit => 0,
            Self::RoutineControl(rc) => rc.required_size(),
            Self::SecurityAccess(sa) => sa.required_size(),
            Self::TesterPresent(tp) => tp.required_size(),
            Self::TransferData(td) => td.required_size(),
            Self::WriteDataByIdentifier(wd) => wd.required_size(),
        }
    }

    /// Serialization function to write a [`Request`] to a [`Writer`](std::io::Write)
    /// This function writes the service byte and then calls the appropriate
    /// serialization function for the service represented by self.
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        // Write the service byte
        writer.write_u8(self.service().request_service_to_byte())?;
        // Write the payload
        Ok(1 + match self {
            Self::ClearDiagnosticInfo(cdi) => cdi.encode(writer),
            Self::CommunicationControl(cc) => cc.encode(writer),
            Self::ControlDTCSettings(ct) => ct.encode(writer),
            Self::DiagnosticSessionControl(ds) => ds.encode(writer),
            Self::EcuReset(er) => er.encode(writer),
            Self::ReadDataByIdentifier(rd) => rd.encode(writer),
            Self::ReadDTCInfo(rd) => rd.encode(writer),
            Self::RequestDownload(rd) => rd.encode(writer),
            Self::RequestTransferExit => Ok(0),
            Self::RoutineControl(rc) => rc.encode(writer),
            Self::SecurityAccess(sa) => sa.encode(writer),
            Self::TesterPresent(tp) => tp.encode(writer),
            Self::TransferData(td) => td.encode(writer),
            Self::WriteDataByIdentifier(wd) => wd.encode(writer),
        }?)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        match self {
            Self::CommunicationControl(cc) => cc.suppress_positive_response(),
            Self::ControlDTCSettings(ct) => ct.is_positive_response_suppressed(),
            Self::DiagnosticSessionControl(ds) => ds.suppress_positive_response(),
            Self::EcuReset(er) => er.suppress_positive_response(),
            Self::SecurityAccess(sa) => sa.suppress_positive_response(),
            Self::TesterPresent(tp) => tp.suppress_positive_response(),
            _ => false,
        }
    }
}

impl<D: DiagnosticDefinition> SingleValueWireFormat for Request<D> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CommunicationControlType, CommunicationType, ProtocolRequest, ResetType, SecurityAccessType,
    };

    #[test]
    fn test_is_positive_response_suppressed() {
        let communication_control_request = ProtocolRequest::communication_control(
            CommunicationControlType::EnableRxAndTx,
            CommunicationType::Normal,
            true,
        );
        assert!(communication_control_request.is_positive_response_suppressed());

        let control_dtc_settings_request =
            ProtocolRequest::control_dtc_settings(DtcSettings::On, true);
        assert!(control_dtc_settings_request.is_positive_response_suppressed());

        let diagnostic_session_control_request = ProtocolRequest::diagnostic_session_control(
            true,
            DiagnosticSessionType::ProgrammingSession,
        );
        assert!(diagnostic_session_control_request.is_positive_response_suppressed());
        let diagnostic_session_control_request = ProtocolRequest::diagnostic_session_control(
            false,
            DiagnosticSessionType::ProgrammingSession,
        );
        let should_not_be_suppressed =
            diagnostic_session_control_request.is_positive_response_suppressed();
        assert!(!should_not_be_suppressed);

        let ecu_reset_request = ProtocolRequest::ecu_reset(true, ResetType::HardReset);
        assert!(ecu_reset_request.is_positive_response_suppressed());

        let security_access_request = ProtocolRequest::security_access(
            true,
            SecurityAccessType::ISO26021_2SendKeyValues,
            vec![0x01, 0x02],
        );
        assert!(security_access_request.is_positive_response_suppressed());

        let tester_present_request = ProtocolRequest::tester_present(true);
        assert!(tester_present_request.is_positive_response_suppressed());

        let clear_diagnostic_info_request =
            ProtocolRequest::clear_diagnostic_info(DTCRecord::new(0x01, 0x02, 0x03), 0x01);
        assert!(!clear_diagnostic_info_request.is_positive_response_suppressed());
    }
}
