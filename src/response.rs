use crate::{
    services::{DiagnosticSessionControlResponse, SecurityAccessResponse},
    CommunicationControlResponse, CommunicationControlType, DiagnosticSessionType,
    EcuResetResponse, Error, ResetType, SecurityAccessType, UdsServiceType,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub struct UdsResponse {
    pub service: UdsServiceType,
    pub data: Vec<u8>,
}

pub enum Response {
    /// Response to a [`CommunicationControlRequest`](crate::CommunicationControlRequest)
    CommunicationControl(CommunicationControlResponse),
    DiagnosticSessionControl(DiagnosticSessionControlResponse),
    EcuReset(EcuResetResponse),
    RequestTransferExit,
    SecurityAccess(SecurityAccessResponse),
    TesterPresent,
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

    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::CommunicationControl(_) => UdsServiceType::CommunicationControl,
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::RequestTransferExit => UdsServiceType::RequestTransferExit,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent => UdsServiceType::TesterPresent,
        }
    }

    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self, Error> {
        let service = UdsServiceType::response_from_byte(reader.read_u8()?);
        Ok(match service {
            UdsServiceType::CommunicationControl => {
                Self::CommunicationControl(CommunicationControlResponse::read(reader)?)
            }
            UdsServiceType::DiagnosticSessionControl => {
                Self::DiagnosticSessionControl(DiagnosticSessionControlResponse::read(reader)?)
            }
            UdsServiceType::EcuReset => Self::EcuReset(EcuResetResponse::read(reader)?),
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(SecurityAccessResponse::read(reader)?)
            }
            UdsServiceType::TesterPresent => Self::TesterPresent,
            _ => todo!(),
        })
    }

    pub fn to_writer<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
        // Write the service byte
        writer.write_u8(self.service().response_to_byte())?;
        // Write the payload
        match self {
            Self::CommunicationControl(cc) => cc.write(writer),
            Self::DiagnosticSessionControl(ds) => ds.write(writer),
            Self::EcuReset(reset) => reset.write(writer),
            Self::RequestTransferExit => Ok(()),
            Self::SecurityAccess(sa) => sa.write(writer),
            Self::TesterPresent => Ok(()),
        }
    }
}
