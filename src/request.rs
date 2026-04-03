//! Module for making and handling UDS Requests
use crate::{
    Decode, Error,
    services::{
        ClearDiagnosticInfoRequest, CommunicationControlRequest, ControlDTCSettingsRequest,
        DiagnosticSessionControlRequest, EcuResetRequest, RequestDownloadRequest,
        SecurityAccessRequestTx, TesterPresentRequest, TransferDataRequestTx,
    },
};

use super::service::UdsServiceType;

/// Zero-copy RX request. Borrows from the wire buffer.
///
/// Variable-length payloads are stored as raw `&'a [u8]` slices that can be
/// further parsed on demand.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Request<'a> {
    /// Clear diagnostic information request.
    ClearDiagnosticInfo(ClearDiagnosticInfoRequest),
    /// Communication control request.
    CommunicationControl(CommunicationControlRequest),
    /// Control DTC settings request.
    ControlDTCSettings(ControlDTCSettingsRequest),
    /// Diagnostic session control request.
    DiagnosticSessionControl(DiagnosticSessionControlRequest),
    /// ECU reset request.
    EcuReset(EcuResetRequest),
    /// Read data by identifier request. Raw DID bytes.
    ReadDataByIdentifier(&'a [u8]),
    /// Read DTC information request. Raw sub-function + parameter bytes.
    ReadDTCInfo(&'a [u8]),
    /// Request download.
    RequestDownload(RequestDownloadRequest),
    /// Request transfer exit.
    RequestTransferExit,
    /// Routine control request. Sub-function byte + raw payload.
    RoutineControl {
        /// Routine control sub-function byte.
        sub_function: u8,
        /// Raw routine ID + optional payload bytes.
        raw_payload: &'a [u8],
    },
    /// Security access request.
    SecurityAccess(SecurityAccessRequestTx<'a>),
    /// Tester present request.
    TesterPresent(TesterPresentRequest),
    /// Transfer data request.
    TransferData(TransferDataRequestTx<'a>),
    /// Write data by identifier request. Raw DID + payload bytes.
    WriteDataByIdentifier(&'a [u8]),
}

impl<'a> Decode<'a> for Request<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let service = UdsServiceType::service_from_request_byte(buf[0]);
        let payload = &buf[1..];

        let request = match service {
            UdsServiceType::ClearDiagnosticInfo => {
                let (req, _) = <ClearDiagnosticInfoRequest as Decode>::decode(payload)?;
                Self::ClearDiagnosticInfo(req)
            }
            UdsServiceType::CommunicationControl => {
                let (req, _) = <CommunicationControlRequest as Decode>::decode(payload)?;
                Self::CommunicationControl(req)
            }
            UdsServiceType::ControlDTCSettings => {
                let (req, _) = <ControlDTCSettingsRequest as Decode>::decode(payload)?;
                Self::ControlDTCSettings(req)
            }
            UdsServiceType::DiagnosticSessionControl => {
                let (req, _) = <DiagnosticSessionControlRequest as Decode>::decode(payload)?;
                Self::DiagnosticSessionControl(req)
            }
            UdsServiceType::EcuReset => {
                let (req, _) = <EcuResetRequest as Decode>::decode(payload)?;
                Self::EcuReset(req)
            }
            UdsServiceType::ReadDataByIdentifier => Self::ReadDataByIdentifier(payload),
            UdsServiceType::ReadDTCInfo => Self::ReadDTCInfo(payload),
            UdsServiceType::RequestDownload => {
                let (req, _) = <RequestDownloadRequest as Decode>::decode(payload)?;
                Self::RequestDownload(req)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::RoutineControl => {
                if payload.is_empty() {
                    return Err(Error::InsufficientData(2));
                }
                Self::RoutineControl {
                    sub_function: payload[0],
                    raw_payload: &payload[1..],
                }
            }
            UdsServiceType::SecurityAccess => {
                let (req, _) = <SecurityAccessRequestTx as Decode>::decode(payload)?;
                Self::SecurityAccess(req)
            }
            UdsServiceType::TesterPresent => {
                let (req, _) = <TesterPresentRequest as Decode>::decode(payload)?;
                Self::TesterPresent(req)
            }
            UdsServiceType::TransferData => {
                let (req, _) = <TransferDataRequestTx as Decode>::decode(payload)?;
                Self::TransferData(req)
            }
            UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(payload),
            _ => return Err(Error::ServiceNotImplemented(service)),
        };
        Ok((request, &[]))
    }
}

impl Request<'_> {
    /// Returns the [`UdsServiceType`] corresponding to this request variant.
    #[must_use]
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
            Self::RoutineControl { .. } => UdsServiceType::RoutineControl,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent(_) => UdsServiceType::TesterPresent,
            Self::TransferData(_) => UdsServiceType::TransferData,
            Self::WriteDataByIdentifier(_) => UdsServiceType::WriteDataByIdentifier,
        }
    }
}
