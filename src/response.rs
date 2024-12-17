use crate::{
    services::{DiagnosticSessionControlResponse, SecurityAccessResponse, TesterPresentResponse},
    DiagnosticSessionType, EcuResetResponse, Error, ResetType, SecurityAccessType, UdsServiceType,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub struct UdsResponse {
    pub service: UdsServiceType,
    pub data: Vec<u8>,
}

pub enum Response {
    DiagnosticSessionControl(DiagnosticSessionControlResponse),
    EcuReset(EcuResetResponse),
    RequestTransferExit,
    SecurityAccess(SecurityAccessResponse),
    TesterPresent(TesterPresentResponse),
}

impl Response {
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
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::EcuReset(_) => UdsServiceType::EcuReset,
            Self::RequestTransferExit => UdsServiceType::RequestTransferExit,
            Self::SecurityAccess(_) => UdsServiceType::SecurityAccess,
            Self::TesterPresent(_) => UdsServiceType::TesterPresent,
        }
    }

    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self, Error> {
        let service = UdsServiceType::response_from_byte(reader.read_u8()?);
        Ok(match service {
            UdsServiceType::DiagnosticSessionControl => {
                Self::DiagnosticSessionControl(DiagnosticSessionControlResponse::read(reader)?)
            }
            UdsServiceType::EcuReset => Self::EcuReset(EcuResetResponse::read(reader)?),
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::SecurityAccess => {
                Self::SecurityAccess(SecurityAccessResponse::read(reader)?)
            }
            UdsServiceType::TesterPresent => {
                Self::TesterPresent(TesterPresentResponse::read(reader)?)
            }
            _ => todo!(),
        })
    }

    pub fn to_writer<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
        // Write the service byte
        writer.write_u8(self.service().response_to_byte())?;
        // Write the payload
        match self {
            Self::DiagnosticSessionControl(ds) => ds.write(writer),
            Self::EcuReset(reset) => reset.write(writer),
            Self::RequestTransferExit => Ok(()),
            Self::SecurityAccess(sa) => sa.write(writer),
            Self::TesterPresent(tp) => tp.write(writer),
        }
    }
}
