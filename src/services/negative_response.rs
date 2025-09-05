//! `NegativeResponse` (0x7F) service implementation
use crate::{Error, NegativeResponseCode, SingleValueWireFormat, UdsServiceType, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A negative response from the server indicating a request could not be fulfilled
#[non_exhaustive]
pub struct NegativeResponse {
    pub request_service: UdsServiceType,
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

impl WireFormat for NegativeResponse {
    fn decode<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
        let request_service = UdsServiceType::service_from_request_byte(reader.read_u8()?);
        let nrc = NegativeResponseCode::from(reader.read_u8()?);
        Ok(Some(Self {
            request_service,
            nrc,
        }))
    }

    fn required_size(&self) -> usize {
        2
    }

    fn encode<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.request_service.request_service_to_byte())?;
        writer.write_u8(u8::from(self.nrc))?;
        Ok(2)
    }
}

impl SingleValueWireFormat for NegativeResponse {}
