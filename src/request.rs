//! Module for making and handling UDS Requests
use crate::{
    services::{
        CommunicationControlRequest, ControlDTCSettingsRequest, DiagnosticSessionControlRequest,
        EcuResetRequest, ReadDataByIdentifierRequest, RequestDownloadRequest,
        RoutineControlRequest, SecurityAccessRequest, TesterPresentRequest, TransferDataRequest,
        WriteDataByIdentifierRequest,
    },
    Error, IterableWireFormat, NegativeResponseCode, ResetType, SecurityAccessType,
    SingleValueWireFormat, WireFormat,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use super::{
    service::UdsServiceType, CommunicationControlType, CommunicationType, DataFormatIdentifier,
    DiagnosticSessionType, DtcSettings, RoutineControlSubFunction,
};

/// UDS Request types
/// Each variant corresponds to a request for a different UDS service
/// The variants contain all request data for each service
pub enum Request<DiagnosticIdentifier, DiagnosticPayload> {
    CommunicationControl(CommunicationControlRequest),
    ControlDTCSettings(ControlDTCSettingsRequest),
    DiagnosticSessionControl(DiagnosticSessionControlRequest),
    EcuReset(EcuResetRequest),
    ReadDataByIdentifier(ReadDataByIdentifierRequest<DiagnosticIdentifier>),
    RequestDownload(RequestDownloadRequest),
    RequestTransferExit,
    RoutineControl(RoutineControlRequest),
    SecurityAccess(SecurityAccessRequest),
    TesterPresent(TesterPresentRequest),
    TransferData(TransferDataRequest),
    WriteDataByIdentifier(WriteDataByIdentifierRequest<DiagnosticIdentifier, DiagnosticPayload>),
}

impl<DiagnosticIdentifier: IterableWireFormat, DiagnosticPayload: IterableWireFormat>
    Request<DiagnosticIdentifier, DiagnosticPayload>
{
    /// Create a `CommunicationControlRequest` with standard address information.
    ///
    /// # Panics
    ///
    ///  Panics if one of the extended address control types is passed.
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
    pub fn control_dtc_settings(setting: DtcSettings, suppress_response: bool) -> Self {
        Request::ControlDTCSettings(ControlDTCSettingsRequest::new(setting, suppress_response))
    }

    /// Create a new `DiagnosticSessionControl` request
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
    pub fn ecu_reset(suppress_positive_response: bool, reset_type: ResetType) -> Self {
        Request::EcuReset(EcuResetRequest::new(suppress_positive_response, reset_type))
    }

    /// Create a new `ReadDataByIdentifier` request
    pub fn read_data_by_identifier(dids: Vec<DiagnosticIdentifier>) -> Self {
        Request::ReadDataByIdentifier(ReadDataByIdentifierRequest::new(dids))
    }

    /// Create a new `RequestDownload` request
    ///     encryption_method: vehicle manufacturer specific (0x0 for no encryption)
    ///     compression_method: vehicle manufacturer specific (0x0 for no compression)
    ///     memory_address: the address in memory to start downloading from (Maximum 40 bits - 1024GB)
    ///     memory_size: the size of the memory to download (Max 4GB)
    pub fn request_download(
        encryption_method: u8,
        compression_method: u8,
        memory_address: u64,
        memory_size: u32,
    ) -> Result<Self, Error> {
        let data_format_identifier =
            DataFormatIdentifier::new(compression_method, encryption_method).unwrap();

        Ok(Request::RequestDownload(RequestDownloadRequest::new(
            data_format_identifier,
            memory_address,
            memory_size,
        )?))
    }

    pub fn request_transfer_exit() -> Self {
        Self::RequestTransferExit
    }

    pub fn routine_control(
        sub_function: RoutineControlSubFunction,
        routine_id: u16,
        data: Vec<u8>,
    ) -> Self {
        Request::RoutineControl(RoutineControlRequest::new(sub_function, routine_id, data))
    }

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

    pub fn tester_present(suppress_positive_response: bool) -> Self {
        Request::TesterPresent(TesterPresentRequest::new(suppress_positive_response))
    }

    pub fn transfer_data(sequence: u8, data: Vec<u8>) -> Self {
        Request::TransferData(TransferDataRequest::new(sequence, data))
    }

    pub fn write_data_by_identifier(
        identifier: DiagnosticIdentifier,
        payload: DiagnosticPayload,
    ) -> Self {
        Request::WriteDataByIdentifier(WriteDataByIdentifierRequest::new(identifier, payload))
    }

    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::CommunicationControl(_) => UdsServiceType::CommunicationControl,
            Self::ControlDTCSettings(_) => UdsServiceType::ControlDTCSettings,
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::ReadDataByIdentifier(_) => UdsServiceType::ReadDataByIdentifier,
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

impl<DiagnosticIdentifier: IterableWireFormat, DiagnosticPayload: IterableWireFormat> WireFormat
    for Request<DiagnosticIdentifier, DiagnosticPayload>
{
    /// Deserialization function to read a [`Request`] from a [`Reader`](std::io::Read)
    /// This function reads the service byte and then calls the appropriate
    /// deserialization function for the service in question
    ///
    /// *Note*:
    ///
    /// Some services allow for custom byte arrays at the end of the request
    /// It is important that only the request data is passed to this function
    /// or the deserialization could read unexpected data
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let service = UdsServiceType::service_from_request_byte(reader.read_u8()?);
        Ok(Some(match service {
            UdsServiceType::CommunicationControl => {
                Self::CommunicationControl(CommunicationControlRequest::from_reader(reader)?)
            }
            UdsServiceType::ControlDTCSettings => {
                Self::ControlDTCSettings(ControlDTCSettingsRequest::from_reader(reader)?)
            }
            UdsServiceType::DiagnosticSessionControl => Self::DiagnosticSessionControl(
                DiagnosticSessionControlRequest::from_reader(reader)?,
            ),
            UdsServiceType::EcuReset => Self::EcuReset(EcuResetRequest::from_reader(reader)?),
            UdsServiceType::ReadDataByIdentifier => {
                Self::ReadDataByIdentifier(ReadDataByIdentifierRequest::from_reader(reader)?)
            }
            UdsServiceType::RequestDownload => {
                Self::RequestDownload(RequestDownloadRequest::from_reader(reader)?)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::RoutineControl => {
                Self::RoutineControl(RoutineControlRequest::from_reader(reader)?)
            }
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(SecurityAccessRequest::from_reader(reader)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(TesterPresentRequest::from_reader(reader)?)
            }
            UdsServiceType::TransferData => {
                Self::TransferData(TransferDataRequest::from_reader(reader)?)
            }
            UdsServiceType::WriteDataByIdentifier => {
                Self::WriteDataByIdentifier(WriteDataByIdentifierRequest::from_reader(reader)?)
            }
            UdsServiceType::Authentication => todo!(),
            UdsServiceType::AccessTimingParameters => todo!(),
            UdsServiceType::SecuredDataTransmission => todo!(),
            UdsServiceType::ResponseOnEvent => todo!(),
            UdsServiceType::LinkControl => todo!(),
            UdsServiceType::ReadMemoryByAddress => todo!(),
            UdsServiceType::ReadScalingDataByIdentifier => todo!(),
            UdsServiceType::ReadDataByIdentifierPeriodic => todo!(),
            UdsServiceType::DynamicallyDefinedDataIdentifier => todo!(),
            UdsServiceType::WriteMemoryByAddress => todo!(),
            UdsServiceType::ClearDiagnosticInfo => todo!(),
            UdsServiceType::ReadDTCInfo => todo!(),
            UdsServiceType::InputOutputControlByIdentifier => todo!(),
            UdsServiceType::RequestUpload => todo!(),
            UdsServiceType::RequestFileTransfer => todo!(),
            UdsServiceType::NegativeResponse => todo!(),
            UdsServiceType::UnsupportedDiagnosticService => todo!(),
        }))
    }

    fn required_size(&self) -> usize {
        1 + match self {
            Self::CommunicationControl(cc) => cc.required_size(),
            Self::ControlDTCSettings(ct) => ct.required_size(),
            Self::DiagnosticSessionControl(ds) => ds.required_size(),
            Self::EcuReset(er) => er.required_size(),
            Self::ReadDataByIdentifier(rd) => rd.required_size(),
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
    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Write the service byte
        writer.write_u8(self.service().request_service_to_byte())?;
        // Write the payload
        Ok(1 + match self {
            Self::CommunicationControl(cc) => cc.to_writer(writer),
            Self::ControlDTCSettings(ct) => ct.to_writer(writer),
            Self::DiagnosticSessionControl(ds) => ds.to_writer(writer),
            Self::EcuReset(er) => er.to_writer(writer),
            Self::ReadDataByIdentifier(rd) => rd.to_writer(writer),
            Self::RequestDownload(rd) => rd.to_writer(writer),
            Self::RequestTransferExit => Ok(0),
            Self::RoutineControl(rc) => rc.to_writer(writer),
            Self::SecurityAccess(sa) => sa.to_writer(writer),
            Self::TesterPresent(tp) => tp.to_writer(writer),
            Self::TransferData(td) => td.to_writer(writer),
            Self::WriteDataByIdentifier(wd) => wd.to_writer(writer),
        }?)
    }
}

impl<DiagnosticIdentifier: IterableWireFormat, DiagnosticPayload: IterableWireFormat>
    IterableWireFormat for Request<DiagnosticIdentifier, DiagnosticPayload>
{
}
