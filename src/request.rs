//! Module for making and handling UDS Requests
use crate::{
    Decode, Encode, Error,
    services::{
        ClearDiagnosticInfoRequest, CommunicationControlRequest, ControlDTCSettingsRequest,
        DiagnosticSessionControlRequest, EcuResetRequest, RequestDownloadRequest,
        RequestFileTransferRequestTx, SecurityAccessRequestTx, TesterPresentRequest,
        TransferDataRequestTx,
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
    /// Request file transfer.
    RequestFileTransfer(RequestFileTransferRequestTx<'a>),
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
            UdsServiceType::RequestFileTransfer => {
                let (req, _) = <RequestFileTransferRequestTx as Decode>::decode(payload)?;
                Self::RequestFileTransfer(req)
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

impl Encode for Request<'_> {
    fn encoded_size(&self) -> usize {
        let payload = match self {
            Self::ClearDiagnosticInfo(req) => req.encoded_size(),
            Self::CommunicationControl(req) => req.encoded_size(),
            Self::ControlDTCSettings(req) => req.encoded_size(),
            Self::DiagnosticSessionControl(req) => req.encoded_size(),
            Self::EcuReset(req) => req.encoded_size(),
            Self::ReadDataByIdentifier(bytes)
            | Self::WriteDataByIdentifier(bytes)
            | Self::ReadDTCInfo(bytes) => bytes.len(),
            Self::RequestDownload(req) => req.encoded_size(),
            Self::RequestFileTransfer(req) => req.encoded_size(),
            Self::RequestTransferExit => 0,
            Self::RoutineControl { raw_payload, .. } => 1 + raw_payload.len(),
            Self::SecurityAccess(req) => req.encoded_size(),
            Self::TesterPresent(req) => req.encoded_size(),
            Self::TransferData(req) => req.encoded_size(),
        };
        1 + payload
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[self.service().request_service_to_byte()])
            .map_err(Error::io)?;
        let payload = match self {
            Self::ClearDiagnosticInfo(req) => req.encode(writer)?,
            Self::CommunicationControl(req) => req.encode(writer)?,
            Self::ControlDTCSettings(req) => req.encode(writer)?,
            Self::DiagnosticSessionControl(req) => req.encode(writer)?,
            Self::EcuReset(req) => req.encode(writer)?,
            Self::ReadDataByIdentifier(bytes)
            | Self::WriteDataByIdentifier(bytes)
            | Self::ReadDTCInfo(bytes) => {
                writer.write_all(bytes).map_err(Error::io)?;
                bytes.len()
            }
            Self::RequestDownload(req) => req.encode(writer)?,
            Self::RequestFileTransfer(req) => req.encode(writer)?,
            Self::RequestTransferExit => 0,
            Self::RoutineControl {
                sub_function,
                raw_payload,
            } => {
                writer.write_all(&[*sub_function]).map_err(Error::io)?;
                writer.write_all(raw_payload).map_err(Error::io)?;
                1 + raw_payload.len()
            }
            Self::SecurityAccess(req) => req.encode(writer)?,
            Self::TesterPresent(req) => req.encode(writer)?,
            Self::TransferData(req) => req.encode(writer)?,
        };
        Ok(1 + payload)
    }

    fn is_positive_response_suppressed(&self) -> bool {
        match self {
            Self::ControlDTCSettings(req) => req.is_positive_response_suppressed(),
            Self::DiagnosticSessionControl(req) => req.is_positive_response_suppressed(),
            Self::EcuReset(req) => req.is_positive_response_suppressed(),
            Self::SecurityAccess(req) => req.is_positive_response_suppressed(),
            Self::TesterPresent(req) => req.is_positive_response_suppressed(),
            _ => false,
        }
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
            Self::RequestFileTransfer(_) => UdsServiceType::RequestFileTransfer,
            Self::RequestTransferExit => UdsServiceType::RequestTransferExit,
            Self::RoutineControl { .. } => UdsServiceType::RoutineControl,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent(_) => UdsServiceType::TesterPresent,
            Self::TransferData(_) => UdsServiceType::TransferData,
            Self::WriteDataByIdentifier(_) => UdsServiceType::WriteDataByIdentifier,
        }
    }
}
