use crate::{
    services::DiagnosticSessionControlResponse, DiagnosticSessionType, Error, UdsServiceType,
};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub struct UdsResponse {
    pub service: UdsServiceType,
    pub data: Vec<u8>,
}

pub enum Response {
    DiagnosticSessionControl(DiagnosticSessionControlResponse),
    RequestTransferExit,
    TesterPresent,
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

    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::DiagnosticSessionControl(_) => UdsServiceType::DiagnosticSessionControl,
            Self::RequestTransferExit => UdsServiceType::RequestTransferExit,
            Self::TesterPresent => UdsServiceType::TesterPresent,
        }
    }

    pub fn from_reader<T: Read>(reader: &mut T) -> Result<Self, Error> {
        let service = UdsServiceType::response_from_byte(reader.read_u8()?);
        Ok(match service {
            UdsServiceType::DiagnosticSessionControl => {
                Self::DiagnosticSessionControl(DiagnosticSessionControlResponse::read(reader)?)
            }
            UdsServiceType::RequestTransferExit => Self::RequestTransferExit,
            UdsServiceType::TesterPresent => Self::TesterPresent,
            _ => todo!(),
        })
    }

    pub fn to_writer<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
        // Write the service byte
        writer.write_u8(self.service().response_to_byte())?;
        // Write the payload
        match self {
            Self::DiagnosticSessionControl(ds) => ds.write(writer),
            Self::RequestTransferExit => Ok(()),
            Self::TesterPresent => Ok(()),
        }
    }
}
