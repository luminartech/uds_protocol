use crate::{Error, NegativeResponseCode, SingleValueWireFormat, UdsServiceType, WireFormat};
use byteorder::{ReadBytesExt, WriteBytesExt};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, utoipa::ToSchema)]
#[non_exhaustive]
pub struct NegativeResponse {
    pub request_service: UdsServiceType,
    pub nrc: NegativeResponseCode,
}

impl NegativeResponse {
    /// Create a new `TesterPresentResponse`
    pub(crate) fn new(request_service: UdsServiceType, nrc: NegativeResponseCode) -> Self {
        Self {
            request_service,
            nrc,
        }
    }
}

impl WireFormat for NegativeResponse {
    /// Create a `TesterPresentResponse` from a sequence of bytes
    fn option_from_reader<T: std::io::Read>(reader: &mut T) -> Result<Option<Self>, Error> {
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

    /// Write the response as a sequence of bytes to a buffer
    fn to_writer<T: std::io::Write>(&self, writer: &mut T) -> Result<usize, Error> {
        writer.write_u8(self.request_service.request_service_to_byte())?;
        writer.write_u8(u8::from(self.nrc))?;
        Ok(2)
    }
}

impl SingleValueWireFormat for NegativeResponse {}
