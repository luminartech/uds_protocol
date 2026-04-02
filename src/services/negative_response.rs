//! `NegativeResponse` (0x7F) service implementation
use crate::{
    Decode, Encode, Error, NegativeResponseCode, SingleValueWireFormat, UdsServiceType, WireFormat,
};
use byteorder_embedded_io::io::{ReadBytesExt, WriteBytesExt};

/// A negative response from the server indicating a request could not be fulfilled
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct NegativeResponse {
    /// The service that triggered this negative response.
    pub request_service: UdsServiceType,
    /// The negative response code indicating why the request failed.
    pub nrc: NegativeResponseCode,
}

impl NegativeResponse {
    /// Create a new `NegativeResponse`
    pub(crate) fn new(request_service: UdsServiceType, nrc: NegativeResponseCode) -> Self {
        Self {
            request_service,
            nrc,
        }
    }
}

impl Encode for NegativeResponse {
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[
                self.request_service.request_service_to_byte(),
                u8::from(self.nrc),
            ])
            .map_err(Error::io)?;
        Ok(2)
    }
}

impl<'a> Decode<'a> for NegativeResponse {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(2));
        }
        let request_service = UdsServiceType::service_from_request_byte(buf[0]);
        let nrc = NegativeResponseCode::from(buf[1]);
        Ok((Self { request_service, nrc }, &buf[2..]))
    }
}

impl WireFormat for NegativeResponse {
    fn required_size(&self) -> usize {
        2
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.request_service.request_service_to_byte())?;
        writer.write_u8(u8::from(self.nrc))?;
        Ok(2)
    }
}

impl SingleValueWireFormat for NegativeResponse {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Self, Error> {
        let request_service = UdsServiceType::service_from_request_byte(reader.read_u8()?);
        let nrc = NegativeResponseCode::from(reader.read_u8()?);
        Ok(Self {
            request_service,
            nrc,
        })
    }
}
