use crate::{
    CommunicationControlType, CommunicationType, Error, NegativeResponseCode,
    SuppressablePositiveResponse,
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const COMMUNICATION_CONTROL_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
];

/// Request for the server to change communication behavior
///
/// # TODO
///
/// Communication Control is not fully implemented.
/// CommunicationType has more complex behavior than is currently implemented.
/// Issue is tracked [here](https://github.com/luminartech/dft/issues/196)
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CommunicationControlRequest {
    control_type: SuppressablePositiveResponse<CommunicationControlType>,
    pub communication_type: CommunicationType,
    pub node_id: Option<u16>,
}

impl CommunicationControlRequest {
    pub(crate) fn new(
        suppress_positive_response: bool,
        control_type: CommunicationControlType,
        communication_type: CommunicationType,
    ) -> Self {
        assert!(!control_type.is_extended_address_variant());
        Self {
            control_type: SuppressablePositiveResponse::new(
                suppress_positive_response,
                control_type,
            ),
            communication_type,
            node_id: None,
        }
    }

    pub(crate) fn new_with_node_id(
        suppress_positive_response: bool,
        control_type: CommunicationControlType,
        communication_type: CommunicationType,
        node_id: u16,
    ) -> Self {
        assert!(control_type.is_extended_address_variant());
        Self {
            control_type: SuppressablePositiveResponse::new(
                suppress_positive_response,
                control_type,
            ),
            communication_type,
            node_id: Some(node_id),
        }
    }

    /// Getter for whether a positive response should be suppressed
    pub fn suppress_positive_response(&self) -> bool {
        self.control_type.suppress_positive_response()
    }

    /// Getter for the requested [`CommunicationControlType`]
    pub fn control_type(&self) -> CommunicationControlType {
        self.control_type.value()
    }

    /// Get the allowed Nack codes for this request
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &COMMUNICATION_CONTROL_NEGATIVE_RESPONSE_CODES
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let enable_byte = buffer.read_u8()?;
        let communication_enable = SuppressablePositiveResponse::try_from(enable_byte)?;
        let communication_type = CommunicationType::try_from(buffer.read_u8()?)?;
        match communication_enable.value() {
            CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo
            | CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo => {
                let node_id = Some(buffer.read_u16::<BigEndian>()?);
                Ok(Self {
                    control_type: communication_enable,
                    communication_type,
                    node_id,
                })
            }
            _ => Ok(Self {
                control_type: communication_enable,
                communication_type,
                node_id: None,
            }),
        }
    }

    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.control_type))?;
        buffer.write_u8(u8::from(self.communication_type))?;
        if let Some(id) = self.node_id {
            buffer.write_u16::<BigEndian>(id)?;
        }
        Ok(())
    }
}

/// Positive response from the server to change communication behavior
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive] // Prevent direct construction externally
pub struct CommunicationControlResponse {
    pub control_type: CommunicationControlType,
}

impl CommunicationControlResponse {
    pub(crate) fn new(control_type: CommunicationControlType) -> Self {
        Self { control_type }
    }

    pub(crate) fn read<T: Read>(buffer: &mut T) -> Result<Self, Error> {
        let control_type = CommunicationControlType::try_from(buffer.read_u8()?)?;
        Ok(Self::new(control_type))
    }

    pub(crate) fn write<T: Write>(&self, buffer: &mut T) -> Result<(), Error> {
        buffer.write_u8(u8::from(self.control_type))?;
        Ok(())
    }
}
