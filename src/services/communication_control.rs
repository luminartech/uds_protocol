//! `CommunicationControl` (0x28) service implementation
use crate::{
    CommunicationControlType, CommunicationType, Decode, Encode, Error, NegativeResponseCode,
    SuppressablePositiveResponse,
};

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
/// `CommunicationType` has more complex behavior than is currently implemented.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CommunicationControlRequest {
    control_type: SuppressablePositiveResponse<CommunicationControlType>,
    /// The communication type to apply the control to.
    pub communication_type: CommunicationType,
    /// Optional node identifier, present only for enhanced-address variants.
    pub node_id: Option<u16>,
}

impl CommunicationControlRequest {
    /// Create a `CommunicationControlRequest` with standard address information.
    ///
    /// # Panics
    /// Panics (debug) if an extended-address control type is passed.
    #[must_use]
    pub fn new(
        suppress_positive_response: bool,
        control_type: CommunicationControlType,
        communication_type: CommunicationType,
    ) -> Self {
        debug_assert!(!control_type.is_extended_address_variant());
        Self {
            control_type: SuppressablePositiveResponse::new(
                suppress_positive_response,
                control_type,
            ),
            communication_type,
            node_id: None,
        }
    }

    /// Create a `CommunicationControlRequest` with enhanced address information.
    ///
    /// # Panics
    /// Panics if a non-extended control type is passed.
    #[must_use]
    pub fn new_with_node_id(
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
    #[must_use]
    pub fn suppress_positive_response(&self) -> bool {
        self.control_type.suppress_positive_response()
    }

    /// Getter for the requested [`CommunicationControlType`]
    #[must_use]
    pub fn control_type(&self) -> CommunicationControlType {
        self.control_type.value()
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &COMMUNICATION_CONTROL_NEGATIVE_RESPONSE_CODES
    }
}
impl Encode for CommunicationControlRequest {
    fn encoded_size(&self) -> usize {
        if self.node_id.is_some() { 4 } else { 2 }
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[
                u8::from(self.control_type),
                u8::from(self.communication_type),
            ])
            .map_err(Error::io)?;
        if let Some(id) = self.node_id {
            writer.write_all(&id.to_be_bytes()).map_err(Error::io)?;
            Ok(4)
        } else {
            Ok(2)
        }
    }
}

impl<'a> Decode<'a> for CommunicationControlRequest {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(2));
        }
        let communication_enable = SuppressablePositiveResponse::try_from(buf[0])?;
        let communication_type = CommunicationType::try_from(buf[1])?;
        match communication_enable.value() {
            CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo
            | CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo => {
                if buf.len() < 4 {
                    return Err(Error::InsufficientData(4));
                }
                let node_id = Some(u16::from_be_bytes([buf[2], buf[3]]));
                Ok((
                    Self {
                        control_type: communication_enable,
                        communication_type,
                        node_id,
                    },
                    &buf[4..],
                ))
            }
            _ => Ok((
                Self {
                    control_type: communication_enable,
                    communication_type,
                    node_id: None,
                },
                &buf[2..],
            )),
        }
    }
}

/// Positive response from the server to change communication behavior
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive] // Prevent direct construction externally
pub struct CommunicationControlResponse {
    /// The communication control type echoed from the request.
    pub control_type: CommunicationControlType,
}

impl CommunicationControlResponse {
    /// Create a new `CommunicationControlResponse`.
    #[must_use]
    pub fn new(control_type: CommunicationControlType) -> Self {
        Self { control_type }
    }
}

impl Encode for CommunicationControlResponse {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.control_type)])
            .map_err(Error::io)?;
        Ok(1)
    }
}

impl<'a> Decode<'a> for CommunicationControlResponse {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let control_type = CommunicationControlType::try_from(buf[0])?;
        Ok((Self::new(control_type), &buf[1..]))
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};

    #[test]
    fn simple_request() {
        let bytes: [u8; 3] = [0x01, 0x02, 0x03];
        let (req, _) = <CommunicationControlRequest as Decode>::decode(&bytes).unwrap();
        assert_eq!(
            req.control_type(),
            CommunicationControlType::EnableRxAndDisableTx
        );
        assert_eq!(req.communication_type, CommunicationType::NetworkManagement);
        assert_eq!(req.node_id, None);

        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(written, req.encoded_size());
        assert_eq!(buffer.len(), req.encoded_size());
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn node_id() {
        let bytes: [u8; 4] = [0x05, 0x02, 0x01, 0x02];
        let (req, _) = <CommunicationControlRequest as Decode>::decode(&bytes).unwrap();
        assert_eq!(
            req.control_type(),
            CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo
        );
        assert_eq!(req.communication_type, CommunicationType::NetworkManagement);
        assert_eq!(req.node_id, Some(258));

        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(written, req.encoded_size());
        assert_eq!(buffer.len(), req.encoded_size());
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn new_with_node_id() {
        let req = CommunicationControlRequest::new_with_node_id(
            true,
            CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo,
            CommunicationType::NetworkManagement,
            258,
        );
        assert_eq!(req.node_id, Some(258));
        assert!(req.suppress_positive_response());
    }
    #[test]
    fn new_extra() {
        let req = CommunicationControlRequest::new(
            false,
            CommunicationControlType::EnableRxAndDisableTx,
            CommunicationType::NetworkManagement,
        );
        assert!(!req.suppress_positive_response());

        assert_eq!(CommunicationControlRequest::allowed_nack_codes().len(), 4);
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};

    #[test]
    fn simple_response() {
        let bytes: [u8; 1] = [0x01];
        let (res, _) = <CommunicationControlResponse as Decode>::decode(&bytes).unwrap();
        assert_eq!(
            res.control_type,
            CommunicationControlType::EnableRxAndDisableTx
        );

        let mut buffer = Vec::new();
        let written = Encode::encode(&res, &mut buffer).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buffer.len(), written);
        assert_encode_size_agrees(&res);
    }
}
