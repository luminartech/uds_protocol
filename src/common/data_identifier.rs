use crate::{Error, IterableWireFormat, Request, SingleValueWireFormat, WireFormat};
use byteorder::WriteBytesExt;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug)]
#[non_exhaustive]
pub struct NoCustomDataIdentifiers;

impl WireFormat for NoCustomDataIdentifiers {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        };
        let identifier = u16::from_be_bytes(identifier_data);
        Err(Error::InvalidDiagnosticIdentifier(identifier))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        unreachable!(
            "NoCustomDataIdentifiers cannot be created, and should never be written to a stream"
        );
    }
}

#[derive(Debug)]
pub enum LuminarDataIdentifier {
    LaserTemp,
    LaserPower,
    ReceiverTemp,
}

/*
impl WireFormat<Error> for LuminarDataIdentifier {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        };
        let identifier = u16::from_be_bytes(identifier_data);
        Ok(Some(match identifier {
            0x0001 => Self::LaserTemp,
            0x0002 => Self::LaserPower,
            0x0003 => Self::ReceiverTemp,
            _ => return Err(Error::InvalidDiagnosticIdentifier(identifier)),
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        match self {
            Self::LaserTemp => writer.write_u16::<byteorder::BigEndian>(0x0001)?,
            Self::LaserPower => writer.write_u16::<byteorder::BigEndian>(0x0002)?,
            Self::ReceiverTemp => writer.write_u16::<byteorder::BigEndian>(0x0003)?,
        };
        Ok(2)
    }
}

impl SingleValueWireFormat<Error> for LuminarDataIdentifier {}
*/

impl SingleValueWireFormat for NoCustomDataIdentifiers {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DataIdentifier<U> {
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
    //... A whole bunch more
    // TODO: ISO Spec C.1 DID parameter definitions
    UserDefined(U),
}

impl<U: SingleValueWireFormat> WireFormat for DataIdentifier<U> {
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let mut identifier_data: [u8; 2] = [0; 2];
        match reader.read(&mut identifier_data)? {
            0 => return Ok(None),
            1 => return Err(Error::IncorrectMessageLengthOrInvalidFormat),
            2 => (),
            _ => unreachable!("Impossible to read more than 2 bytes into 2 byte array"),
        };
        // At this point, we have read 2 bytes into the identifier_data array, and can safely treat it as a u16
        let identifier = u16::from_be_bytes(identifier_data);
        Ok(Some(match identifier {
            0x0000..=0x00FF => Self::ISOSAEReserved(identifier),
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
            //
            _ => Self::UserDefined(U::from_reader(&mut identifier_data.as_ref())?),
        }))
    }

    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        match self {
            Self::ISOSAEReserved(identifier) => {
                writer.write_u16::<byteorder::BigEndian>(*identifier)?
            }
            Self::BootSoftwareIdentification => writer.write_u16::<byteorder::BigEndian>(0x0183)?,
            Self::ApplicationSoftware => writer.write_u16::<byteorder::BigEndian>(0x0184)?,
            Self::ApplicationDataIdentification => {
                writer.write_u16::<byteorder::BigEndian>(0x0185)?
            }
            Self::BootSoftwareFingerprint => writer.write_u16::<byteorder::BigEndian>(0x0186)?,
            Self::ApplicationSoftwareFingerprint => {
                writer.write_u16::<byteorder::BigEndian>(0x0187)?
            }
            Self::ApplicationDataFingerprint => writer.write_u16::<byteorder::BigEndian>(0x0188)?,
            Self::ActiveDiagnosticSession => writer.write_u16::<byteorder::BigEndian>(0x0189)?,
            Self::VehicleManufacturerSparePartNumber => {
                writer.write_u16::<byteorder::BigEndian>(0x018A)?
            }
            Self::VehicleManufacturerECUSoftwareNumber => {
                writer.write_u16::<byteorder::BigEndian>(0x018B)?
            }
            Self::VehicleManufacturerECUSoftwareVersionNumber => {
                writer.write_u16::<byteorder::BigEndian>(0x018C)?
            }
            Self::UserDefined(u) => return u.to_writer(writer),
        };
        Ok(2)
    }
}

impl<U: SingleValueWireFormat> IterableWireFormat for DataIdentifier<U> {}
