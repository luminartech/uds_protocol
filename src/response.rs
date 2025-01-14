use crate::{
    CommunicationControlResponse, CommunicationControlType, DiagnosticSessionControlResponse,
    DiagnosticSessionType, EcuResetResponse, Error, ResetType, SecurityAccessResponse,
    SecurityAccessType, SingleValueWireFormat, TesterPresentResponse, UdsServiceType, WireFormat,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

pub struct UdsResponse {
    pub service: UdsServiceType,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Response {
    /// Response to a [`CommunicationControlRequest`](crate::CommunicationControlRequest)
    CommunicationControl(CommunicationControlResponse),
    /// Response to a [`DiagnosticSessionControlRequest`](crate::DiagnosticSessionControlRequest)
    DiagnosticSessionControl(DiagnosticSessionControlResponse),
    /// Response to a [`EcuResetRequest`](crate::EcuResetRequest)
    EcuReset(EcuResetResponse),
    /// Response to a [`RequestTransferExit`](crate::RequestTransferExit)
    RequestTransferExit,
    SecurityAccess(SecurityAccessResponse),
    TesterPresent(TesterPresentResponse),
}

impl Response {
    pub fn communication_control(control_type: CommunicationControlType) -> Self {
        Response::CommunicationControl(CommunicationControlResponse::new(control_type))
    }
    pub fn diagnostic_session_control(
        session_type: DiagnosticSessionType,
        p2_max: u16,
        p2_star_max: u16,
    ) -> Self {
        Response::DiagnosticSessionControl(DiagnosticSessionControlResponse::new(
            session_type,
            p2_max,
            p2_star_max,
        ))
    }

    pub fn ecu_reset(reset_type: ResetType, power_down_time: u8) -> Self {
        Response::EcuReset(EcuResetResponse::new(reset_type, power_down_time))
    }

    pub fn security_access(access_type: SecurityAccessType, security_seed: Vec<u8>) -> Self {
        Response::SecurityAccess(SecurityAccessResponse::new(access_type, security_seed))
    }

    pub fn tester_present() -> Self {
        Response::TesterPresent(TesterPresentResponse::new())
    }

    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::CommunicationControl(_) => UdsServiceType::CommunicationControl,
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::RequestTransferExit => UdsServiceType::RequestTransferExit,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent(_) => UdsServiceType::TesterPresent,
        }
    }
}

impl WireFormat<'_, Error> for Response {
    fn option_from_reader<T: Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let service = UdsServiceType::response_from_byte(reader.read_u8()?);
        Ok(Some(match service {
            UdsServiceType::CommunicationControl => {
                Self::CommunicationControl(CommunicationControlResponse::from_reader(reader)?)
            }
            UdsServiceType::DiagnosticSessionControl => Self::DiagnosticSessionControl(
                DiagnosticSessionControlResponse::from_reader(reader)?,
            ),
            UdsServiceType::EcuReset => Self::EcuReset(EcuResetResponse::from_reader(reader)?),
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(SecurityAccessResponse::from_reader(reader)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(TesterPresentResponse::from_reader(reader)?)
            }
            _ => todo!(),
        }))
    }

    fn to_writer<T: Write>(&self, writer: &mut T) -> Result<usize, Error> {
        // Write the service byte
        writer.write_u8(self.service().response_to_byte())?;
        // Write the payload
        match self {
            Self::CommunicationControl(cc) => cc.to_writer(writer),
            Self::DiagnosticSessionControl(ds) => ds.to_writer(writer),
            Self::EcuReset(reset) => reset.to_writer(writer),
            Self::RequestTransferExit => Ok(0),
            Self::SecurityAccess(sa) => sa.to_writer(writer),
            Self::TesterPresent(tp) => tp.to_writer(writer),
        }
    }
}

impl SingleValueWireFormat<'_, Error> for Response {}
