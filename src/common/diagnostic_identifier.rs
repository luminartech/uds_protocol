use crate::{Error, IterableWireFormat, WireFormat};
use byteorder::WriteBytesExt;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum UDSIdentifier {
    ISOSAEReserved(u16),
    BootSoftwareIdentification,
    ApplicationSoftware,
    ApplicationDataIdentification,
    BootSoftwareFingerprint,
    ApplicationSoftwareFingerprint,
    ApplicationDataFingerprint,
    ActiveDiagnosticSession,
    VehicleManufacturerSparePartNumber,
    VehicleManufacturerECUSoftwareNumber,
    VehicleManufacturerECUSoftwareVersionNumber,
}

impl TryFrom<u16> for UDSIdentifier {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0000..=0x00FF => Self::ISOSAEReserved(value),
            0x0183 => Self::BootSoftwareIdentification,
            0x0184 => Self::ApplicationSoftware,
            0x0185 => Self::ApplicationDataIdentification,
            0x0186 => Self::BootSoftwareFingerprint,
            0x0187 => Self::ApplicationSoftwareFingerprint,
            0x0188 => Self::ApplicationDataFingerprint,
            0x0189 => Self::ActiveDiagnosticSession,
            0x018A => Self::VehicleManufacturerSparePartNumber,
            0x018B => Self::VehicleManufacturerECUSoftwareNumber,
            0x018C => Self::VehicleManufacturerECUSoftwareVersionNumber,
            _ => return Err(Error::InvalidDiagnosticIdentifier(value)),
        })
    }
}

impl From<UDSIdentifier> for u16 {
    fn from(value: UDSIdentifier) -> Self {
        match value {
            UDSIdentifier::ISOSAEReserved(identifier) => identifier,
            UDSIdentifier::BootSoftwareIdentification => 0x0183,
            UDSIdentifier::ApplicationSoftware => 0x0184,
            UDSIdentifier::ApplicationDataIdentification => 0x0185,
            UDSIdentifier::BootSoftwareFingerprint => 0x0186,
            UDSIdentifier::ApplicationSoftwareFingerprint => 0x0187,
            UDSIdentifier::ApplicationDataFingerprint => 0x0188,
            UDSIdentifier::ActiveDiagnosticSession => 0x0189,
            UDSIdentifier::VehicleManufacturerSparePartNumber => 0x018A,
            UDSIdentifier::VehicleManufacturerECUSoftwareNumber => 0x018B,
            UDSIdentifier::VehicleManufacturerECUSoftwareVersionNumber => 0x018C,
        }
    }
}

/// Protocol Identifier provides an implementation of Diagnostics Identifiers that only supports IDs defined by UDS
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub struct ProtocolIdentifier {
    pub identifier: UDSIdentifier,
}

impl ProtocolIdentifier {
    pub fn new(identifier: UDSIdentifier) -> Self {
        ProtocolIdentifier { identifier }
    }
}

impl WireFormat for ProtocolIdentifier {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        };

        // This seems wrong or something it doesn't account for
        let identifier = u16::from_be_bytes(identifier_data);
        Ok(Some(Self {
            identifier: UDSIdentifier::try_from(identifier)?,
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u16::<byteorder::BigEndian>(u16::from(self.identifier))?;
        Ok(2)
    }
}

impl IterableWireFormat for ProtocolIdentifier {}
