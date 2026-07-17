//! `CommunicationControl` (0x28) service implementation
use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, Incomplete, NegativeResponseCode};

/// `CommunicationControlType` is used to specify the type of communication behavior to be modified
///
/// *Note*:
///
/// Conversions from `u8` to `CommunicationControlType` are fallible and will return an [`Error`](crate::Error) if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommunicationControlType {
    /// This value indicates that the reception and transmission of messages
    /// shall be enabled for the specified [`CommunicationType`]
    EnableRxAndTx,
    /// This value indicates that the reception of messages shall be enabled
    /// and the transmission of messages shall be disabled for the specified [`CommunicationType`]
    EnableRxAndDisableTx,
    /// This value indicates that the reception of messages shall be disabled
    /// and the transmission of messages shall be enabled for the specified [`CommunicationType`]
    DisableRxAndEnableTx,
    /// This value indicates that the reception and transmission of messages
    /// shall be disabled for the specified [`CommunicationType`]
    DisableRxAndTx,
    /// This value indicates that the reception of messages shall be enabled
    /// and the transmission of messages shall be disabled for the specified [`CommunicationType`]
    /// Additionally, enhanced address information shall be included in the request
    EnableRxAndDisableTxWithEnhancedAddressInfo,
    /// This value indicates that the reception and transmission of messages
    /// shall be enabled for the specified [`CommunicationType`]
    /// Additionally, enhanced address information shall be included in the request
    EnableRxAndTxWithEnhancedAddressInfo,
    /// These values are reserved by the ISO 14229-1 Specification.
    ///
    /// Construct through [`CommunicationControlType::try_from`] so the raw byte is
    /// range-checked and can never collide with the SPRMIB bit.
    #[cfg_attr(feature = "clap", clap(skip))]
    #[non_exhaustive]
    ISOSAEReserved(u8),
    /// Values reserved for use by vehicle manufacturers.
    ///
    /// Construct through [`CommunicationControlType::try_from`] so the raw byte is
    /// range-checked and can never collide with the SPRMIB bit.
    #[cfg_attr(feature = "clap", clap(skip))]
    #[non_exhaustive]
    VehicleManufacturerSpecific(u8),
    /// Values reserved for use by system suppliers.
    ///
    /// Construct through [`CommunicationControlType::try_from`] so the raw byte is
    /// range-checked and can never collide with the SPRMIB bit.
    #[cfg_attr(feature = "clap", clap(skip))]
    #[non_exhaustive]
    SystemSupplierSpecific(u8),
}

impl CommunicationControlType {
    /// Returns `true` if this control type requires an enhanced-address node identifier.
    #[must_use]
    pub const fn is_extended_address_variant(&self) -> bool {
        matches!(
            self,
            CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo
                | CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo
        )
    }
}

impl From<CommunicationControlType> for u8 {
    #[allow(clippy::match_same_arms)]
    fn from(value: CommunicationControlType) -> Self {
        match value {
            CommunicationControlType::EnableRxAndTx => 0x00,
            CommunicationControlType::EnableRxAndDisableTx => 0x01,
            CommunicationControlType::DisableRxAndEnableTx => 0x02,
            CommunicationControlType::DisableRxAndTx => 0x03,
            CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo => 0x04,
            CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo => 0x05,
            CommunicationControlType::ISOSAEReserved(val) => val,
            CommunicationControlType::VehicleManufacturerSpecific(val) => val,
            CommunicationControlType::SystemSupplierSpecific(val) => val,
        }
    }
}

impl TryFrom<u8> for CommunicationControlType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(Self::EnableRxAndTx),
            0x01 => Ok(Self::EnableRxAndDisableTx),
            0x02 => Ok(Self::DisableRxAndEnableTx),
            0x03 => Ok(Self::DisableRxAndTx),
            0x04 => Ok(Self::EnableRxAndDisableTxWithEnhancedAddressInfo),
            0x05 => Ok(Self::EnableRxAndTxWithEnhancedAddressInfo),
            0x06..=0x3F | 0x7F => Ok(Self::ISOSAEReserved(value)),
            0x40..=0x5F => Ok(Self::VehicleManufacturerSpecific(value)),
            0x60..=0x7E => Ok(Self::SystemSupplierSpecific(value)),
            _ => Err(Error::InvalidCommunicationControlType(value)),
        }
    }
}

#[cfg(test)]
mod communication_control_type_tests {
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn from_all_u8_values() {
        for i in 0..=u8::MAX {
            let msg_type = CommunicationControlType::try_from(i);
            match i {
                0x00 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::EnableRxAndTx)
                )),
                0x01 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::EnableRxAndDisableTx)
                )),
                0x02 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::DisableRxAndEnableTx)
                )),
                0x03 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::DisableRxAndTx)
                )),
                0x04 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo)
                )),
                0x05 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo)
                )),
                0x06..=0x3F | 0x7F => {
                    assert!(matches!(
                        msg_type,
                        Ok(CommunicationControlType::ISOSAEReserved(_))
                    ));
                }
                0x40..=0x5F => {
                    assert!(matches!(
                        msg_type,
                        Ok(CommunicationControlType::VehicleManufacturerSpecific(_))
                    ));
                }
                0x60..=0x7E => {
                    assert!(matches!(
                        msg_type,
                        Ok(CommunicationControlType::SystemSupplierSpecific(_))
                    ));
                }
                _ => assert!(matches!(
                    msg_type,
                    Err(Error::InvalidCommunicationControlType(_))
                )),
            }
        }
    }

    #[test]
    fn communication_control_type_round_trip_all_values() {
        for i in 0..=u8::MAX {
            let value = CommunicationControlType::try_from(i);
            match value {
                Ok(value) => assert_eq!(u8::from(value), i),
                Err(Error::InvalidCommunicationControlType(value)) => assert_eq!(value, i),
                _ => panic!("Invalid error type"),
            }
        }
    }
}

/// `CommunicationType` is used to specify the type of communication behavior to be modified.
///
/// TODO: Note that this implementation is incomplete and does not properly handle the behavior of the upper 4 bits of the field.
/// This implementation is a placeholder and will be updated in the future, which will also be a breaking API change.
///
/// Note:
///
/// Conversions from `u8` to `CommunicationType` are fallible and will return an [`Error`](crate::Error) if the value is not a valid `CommunicationType`
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommunicationType {
    /// This value is reserved by the ISO 14229-1 Specification
    ISOSAEReserved,
    /// This value represents all application related communication.
    Normal,
    /// This value represents all network management related communication.
    NetworkManagement,
    /// This value represents all application and network management related communication.
    NormalAndNetworkManagement,
}

impl From<CommunicationType> for u8 {
    fn from(value: CommunicationType) -> Self {
        match value {
            CommunicationType::ISOSAEReserved => 0x00,
            CommunicationType::Normal => 0x01,
            CommunicationType::NetworkManagement => 0x02,
            CommunicationType::NormalAndNetworkManagement => 0x03,
        }
    }
}

impl TryFrom<u8> for CommunicationType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x00 => Ok(Self::ISOSAEReserved),
            0x01 => Ok(Self::Normal),
            0x02 => Ok(CommunicationType::NetworkManagement),
            0x03 => Ok(CommunicationType::NormalAndNetworkManagement),
            val => Err(Error::InvalidCommunicationType(val)),
        }
    }
}

#[cfg(test)]
mod communication_type_tests {
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn communication_type_from_all_u8_values() {
        for i in 0..=u8::MAX {
            let msg_type = CommunicationType::try_from(i);
            match i {
                0x00 => assert!(matches!(msg_type, Ok(CommunicationType::ISOSAEReserved))),
                0x01 => assert!(matches!(msg_type, Ok(CommunicationType::Normal))),
                0x02 => assert!(matches!(msg_type, Ok(CommunicationType::NetworkManagement))),
                0x03 => assert!(matches!(
                    msg_type,
                    Ok(CommunicationType::NormalAndNetworkManagement)
                )),
                _ => assert!(matches!(msg_type, Err(Error::InvalidCommunicationType(_)))),
            }
        }
    }

    #[test]
    fn communication_type_round_trip_all_values() {
        for i in 0..=u8::MAX {
            let value = CommunicationType::try_from(i);
            match value {
                Ok(value) => assert_eq!(u8::from(value), i),
                Err(Error::InvalidCommunicationType(value)) => assert_eq!(value, i),
                _ => panic!("Invalid error type"),
            }
        }
    }
}

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
    communication_type: CommunicationType,
    node_id: Option<u16>,
}

impl CommunicationControlRequest {
    /// Create a `CommunicationControlRequest` with standard address information.
    ///
    /// # Errors
    /// Returns [`Error::InvalidCommunicationControlType`] if `control_type` is an
    /// enhanced-address variant — those require a node identifier and must be built
    /// with [`new_with_node_id`](Self::new_with_node_id).
    pub fn new(
        suppress_positive_response: bool,
        control_type: CommunicationControlType,
        communication_type: CommunicationType,
    ) -> Result<Self, Error> {
        if control_type.is_extended_address_variant() {
            return Err(Error::InvalidCommunicationControlType(u8::from(
                control_type,
            )));
        }
        Ok(Self {
            control_type: SuppressablePositiveResponse::new(
                suppress_positive_response,
                control_type,
            ),
            communication_type,
            node_id: None,
        })
    }

    /// Create a `CommunicationControlRequest` with enhanced address information.
    ///
    /// # Errors
    /// Returns [`Error::InvalidCommunicationControlType`] if `control_type` is not an
    /// enhanced-address variant — a node identifier is only carried by the
    /// `*WithEnhancedAddressInfo` variants.
    pub fn new_with_node_id(
        suppress_positive_response: bool,
        control_type: CommunicationControlType,
        communication_type: CommunicationType,
        node_id: u16,
    ) -> Result<Self, Error> {
        if !control_type.is_extended_address_variant() {
            return Err(Error::InvalidCommunicationControlType(u8::from(
                control_type,
            )));
        }
        Ok(Self {
            control_type: SuppressablePositiveResponse::new(
                suppress_positive_response,
                control_type,
            ),
            communication_type,
            node_id: Some(node_id),
        })
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

    /// The [`CommunicationType`] the control applies to.
    #[must_use]
    pub const fn communication_type(&self) -> CommunicationType {
        self.communication_type
    }

    /// The node identifier, present only for enhanced-address control types.
    #[must_use]
    pub const fn node_id(&self) -> Option<u16> {
        self.node_id
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &COMMUNICATION_CONTROL_NEGATIVE_RESPONSE_CODES
    }
}
impl Encode for CommunicationControlRequest {
    type Error = crate::Error;

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
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(Incomplete {
                needed: 2,
                available: buf.len(),
            }));
        }
        let communication_enable = SuppressablePositiveResponse::try_from(buf[0])?;
        let communication_type = CommunicationType::try_from(buf[1])?;
        match communication_enable.value() {
            CommunicationControlType::EnableRxAndDisableTxWithEnhancedAddressInfo
            | CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo => {
                if buf.len() < 4 {
                    return Err(Error::InsufficientData(Incomplete {
                        needed: 4,
                        available: buf.len(),
                    }));
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
    pub const fn new(control_type: CommunicationControlType) -> Self {
        Self { control_type }
    }
}

impl Encode for CommunicationControlResponse {
    type Error = crate::Error;

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.control_type)])
            .map_err(Error::io)?;
        Ok(1)
    }
}

impl<'a> Decode<'a> for CommunicationControlResponse {
    type Error = crate::Error;

    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(Incomplete {
                needed: 1,
                available: buf.len(),
            }));
        }
        let control_type = CommunicationControlType::try_from(buf[0])?;
        Ok((Self::new(control_type), &buf[1..]))
    }
}

#[cfg(test)]
mod request {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[cfg(feature = "alloc")]
    #[test]
    fn simple_request() {
        let bytes: [u8; 3] = [0x01, 0x02, 0x03];
        let (req, _) = <CommunicationControlRequest as Decode>::decode(&bytes).unwrap();
        assert_eq!(
            req.control_type(),
            CommunicationControlType::EnableRxAndDisableTx
        );
        assert_eq!(
            req.communication_type(),
            CommunicationType::NetworkManagement
        );
        assert_eq!(req.node_id(), None);

        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(written, req.encoded_size().unwrap());
        assert_eq!(buffer.len(), req.encoded_size().unwrap());
        assert_encode_size_agrees(&req);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn node_id() {
        let bytes: [u8; 4] = [0x05, 0x02, 0x01, 0x02];
        let (req, _) = <CommunicationControlRequest as Decode>::decode(&bytes).unwrap();
        assert_eq!(
            req.control_type(),
            CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo
        );
        assert_eq!(
            req.communication_type(),
            CommunicationType::NetworkManagement
        );
        assert_eq!(req.node_id(), Some(258));

        let mut buffer = Vec::new();
        let written = Encode::encode(&req, &mut buffer).unwrap();
        assert_eq!(written, req.encoded_size().unwrap());
        assert_eq!(buffer.len(), req.encoded_size().unwrap());
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn new_with_node_id() {
        let req = CommunicationControlRequest::new_with_node_id(
            true,
            CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo,
            CommunicationType::NetworkManagement,
            258,
        )
        .unwrap();
        assert_eq!(req.node_id(), Some(258));
        assert!(req.suppress_positive_response());
    }

    #[test]
    fn new_extra() {
        let req = CommunicationControlRequest::new(
            false,
            CommunicationControlType::EnableRxAndDisableTx,
            CommunicationType::NetworkManagement,
        )
        .unwrap();
        assert!(!req.suppress_positive_response());

        assert_eq!(CommunicationControlRequest::allowed_nack_codes().len(), 4);
    }

    #[test]
    fn new_rejects_enhanced_address_variant() {
        // An enhanced-address control type has no node id via `new`; it must error
        // rather than silently encode a frame missing the mandatory node identifier.
        let result = CommunicationControlRequest::new(
            false,
            CommunicationControlType::EnableRxAndTxWithEnhancedAddressInfo,
            CommunicationType::NetworkManagement,
        );
        assert!(matches!(
            result,
            Err(Error::InvalidCommunicationControlType(0x05))
        ));
    }

    #[test]
    fn new_with_node_id_rejects_standard_variant() {
        let result = CommunicationControlRequest::new_with_node_id(
            false,
            CommunicationControlType::EnableRxAndDisableTx,
            CommunicationType::NetworkManagement,
            258,
        );
        assert!(matches!(
            result,
            Err(Error::InvalidCommunicationControlType(0x01))
        ));
    }
}

#[cfg(test)]
mod response {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};
    #[cfg(feature = "alloc")]
    use alloc::vec::Vec;

    #[cfg(feature = "alloc")]
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
