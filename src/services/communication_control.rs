//! `CommunicationControl` (0x28) service implementation
use crate::shared::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, Incomplete, NegativeResponseCode};
use automotive_wire_codec::{write_all, write_u8, write_u16_be};

/// `CommunicationControlType` is used to specify the type of communication behavior to be modified
///
/// *Note*:
///
/// Conversions from `u8` to `CommunicationControlType` are fallible and will return an [`Error`] if the
/// Suppress Positive Response bit is set.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8", into = "u8"))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
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
    IsoSaeReserved(u8),
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
    /// The raw sub-function byte, without the SPRMIB bit.
    ///
    /// `const` so [`CommunicationControlRequest::new`] and
    /// [`new_with_node_id`](CommunicationControlRequest::new_with_node_id) can be `const` too:
    /// they need the byte for their error payload, and `u8::from` is a trait method, which is
    /// not callable in a `const fn` on stable.
    #[must_use]
    pub const fn value(&self) -> u8 {
        match self {
            Self::EnableRxAndTx => 0x00,
            Self::EnableRxAndDisableTx => 0x01,
            Self::DisableRxAndEnableTx => 0x02,
            Self::DisableRxAndTx => 0x03,
            Self::EnableRxAndDisableTxWithEnhancedAddressInfo => 0x04,
            Self::EnableRxAndTxWithEnhancedAddressInfo => 0x05,
            Self::IsoSaeReserved(value)
            | Self::VehicleManufacturerSpecific(value)
            | Self::SystemSupplierSpecific(value) => *value,
        }
    }

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
    fn from(value: CommunicationControlType) -> Self {
        value.value()
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
            0x06..=0x3F | 0x7F => Ok(Self::IsoSaeReserved(value)),
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
                        Ok(CommunicationControlType::IsoSaeReserved(_))
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

/// Which network the `communicationType` byte applies to — its high nibble (bits 7-4).
///
/// See ISO 14229-1:2020 Annex B Table B.1. The low nibble of the same byte is the
/// [`CommunicationType`].
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8", into = "u8"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum SubnetNumber {
    /// `0x0` — apply to the receiving node, including communication to all connected networks.
    ///
    /// The default, and what a request that does not target a particular subnet carries.
    #[default]
    AllConnectedNetworks,
    /// `0x1`-`0xE` — apply to the specific subnet identified by this number.
    ///
    /// Construct through [`SubnetNumber::try_from`] so the value is range-checked and cannot
    /// collide with the other two variants.
    #[non_exhaustive]
    Specific(u8),
    /// `0xF` — apply to the network the request was received on.
    ReceivedOn,
}

impl SubnetNumber {
    /// The high nibble this subnet occupies, as a value in `0x0..=0xF`.
    #[must_use]
    pub const fn value(&self) -> u8 {
        match self {
            Self::AllConnectedNetworks => 0x0,
            Self::Specific(subnet) => *subnet,
            Self::ReceivedOn => 0xF,
        }
    }
}

impl From<SubnetNumber> for u8 {
    fn from(subnet: SubnetNumber) -> Self {
        subnet.value()
    }
}

impl TryFrom<u8> for SubnetNumber {
    type Error = Error;

    /// # Errors
    /// Returns [`Error::InvalidCommunicationType`] if `value` does not fit in a nibble.
    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0x0 => Ok(Self::AllConnectedNetworks),
            0x1..=0xE => Ok(Self::Specific(value)),
            0xF => Ok(Self::ReceivedOn),
            _ => Err(Error::InvalidCommunicationType(value)),
        }
    }
}

/// `CommunicationType` is used to specify the type of communication behavior to be modified.
///
/// This is the low nibble (bits 1-0) of the `communicationType` byte; the high nibble is the
/// [`SubnetNumber`]. Bits 3-2 are `ISOSAEReserved` and must be zero.
///
/// Note:
///
/// Conversions from `u8` to `CommunicationType` are fallible and will return an [`Error`] if the value is not a valid `CommunicationType`
///
/// Unlike the other single-byte types here, `serde` uses the derived variant-name form rather
/// than routing through `u8`. There is nothing to smuggle — the four variants are in bijection
/// with `0x00..=0x03` and `TryFrom<u8>` accepts all four — and the byte form would be *worse*:
/// `TryFrom<u8>` reads bits 1-0 of a whole `communicationType` byte and masks the rest, so
/// deserializing `17` would silently yield `Normal` and re-serialize as `1`.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommunicationType {
    /// This value is reserved by the ISO 14229-1 Specification
    IsoSaeReserved,
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
            CommunicationType::IsoSaeReserved => 0x00,
            CommunicationType::Normal => 0x01,
            CommunicationType::NetworkManagement => 0x02,
            CommunicationType::NormalAndNetworkManagement => 0x03,
        }
    }
}

impl TryFrom<u8> for CommunicationType {
    type Error = Error;

    /// Reads bits 1-0 of a `communicationType` byte, rejecting a byte whose reserved bits 3-2
    /// are set. Use [`SubnetNumber::try_from`] on the high nibble for the rest of the byte.
    ///
    /// # Errors
    /// Returns [`Error::InvalidCommunicationType`] if bits 3-2 are non-zero, which Annex B
    /// Table B.1 marks `ISOSAEReserved`.
    fn try_from(value: u8) -> Result<Self, Error> {
        if value & RESERVED_BITS_MASK != 0 {
            return Err(Error::InvalidCommunicationType(value));
        }
        match value & MESSAGE_TYPE_MASK {
            0x00 => Ok(Self::IsoSaeReserved),
            0x01 => Ok(Self::Normal),
            0x02 => Ok(CommunicationType::NetworkManagement),
            0x03 => Ok(CommunicationType::NormalAndNetworkManagement),
            // `MESSAGE_TYPE_MASK` keeps only two bits, so no other value can reach here.
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod communication_type_tests {
    use super::*;
    /// Check that we properly decode and encode hex bytes
    #[test]
    fn communication_type_from_all_u8_values() {
        // `CommunicationType` is bits 1-0 of the byte, so the subnet nibble is ignored here and
        // only the reserved bits 3-2 can make a byte invalid (Annex B Table B.1).
        for i in 0..=u8::MAX {
            let msg_type = CommunicationType::try_from(i);
            if i & RESERVED_BITS_MASK != 0 {
                assert!(
                    matches!(msg_type, Err(Error::InvalidCommunicationType(_))),
                    "{i:#04X} sets a reserved bit but was accepted"
                );
                continue;
            }
            match i & MESSAGE_TYPE_MASK {
                0x00 => assert!(matches!(msg_type, Ok(CommunicationType::IsoSaeReserved))),
                0x01 => assert!(matches!(msg_type, Ok(CommunicationType::Normal))),
                0x02 => assert!(matches!(msg_type, Ok(CommunicationType::NetworkManagement))),
                _ => assert!(matches!(
                    msg_type,
                    Ok(CommunicationType::NormalAndNetworkManagement)
                )),
            }
        }
    }

    #[test]
    fn communication_type_round_trip_all_values() {
        // A full byte round-trips only once the subnet nibble is put back, which is what
        // `CommunicationControlRequest`'s codec does.
        for i in 0..=u8::MAX {
            let message_type = CommunicationType::try_from(i);
            let subnet = SubnetNumber::try_from((i & SUBNET_MASK) >> 4);
            match (message_type, subnet) {
                (Ok(message_type), Ok(subnet)) => {
                    assert_eq!((subnet.value() << 4) | u8::from(message_type), i);
                }
                (Err(Error::InvalidCommunicationType(value)), _) => assert_eq!(value, i),
                other => panic!("unexpected result for {i:#04X}: {other:?}"),
            }
        }
    }

    #[test]
    fn every_subnet_nibble_round_trips() {
        for nibble in 0x0..=0xFu8 {
            let subnet = SubnetNumber::try_from(nibble).unwrap();
            assert_eq!(subnet.value(), nibble);
        }
        assert_eq!(SubnetNumber::default(), SubnetNumber::AllConnectedNetworks);
        assert!(SubnetNumber::try_from(0x10).is_err());
    }
}

/// Bits 1-0 of the `communicationType` byte: the message type.
const MESSAGE_TYPE_MASK: u8 = 0b0000_0011;
/// Bits 3-2 of the `communicationType` byte, `ISOSAEReserved` per Annex B Table B.1.
const RESERVED_BITS_MASK: u8 = 0b0000_1100;
/// Bits 7-4 of the `communicationType` byte: the subnet number.
const SUBNET_MASK: u8 = 0b1111_0000;

const COMMUNICATION_CONTROL_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 4] = [
    NegativeResponseCode::SubFunctionNotSupported,
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
];

/// Request for the server to change communication behavior
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        try_from = "CommunicationControlRepr",
        into = "CommunicationControlRepr"
    )
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CommunicationControlRequest {
    /// Whether the server should suppress a positive response (SPRMIB).
    ///
    /// Public because it carries no invariant with the other fields: it occupies bit 7 of the
    /// sub-function byte and is fused onto `control_type` only at the wire boundary. The
    /// remaining fields stay private because `node_id` must be present exactly when
    /// `control_type` is an enhanced-address variant.
    pub suppress_positive_response: bool,
    control_type: CommunicationControlType,
    communication_type: CommunicationType,
    subnet: SubnetNumber,
    node_id: Option<u16>,
}

impl CommunicationControlRequest {
    /// Create a `CommunicationControlRequest` with standard address information.
    ///
    /// # Errors
    /// Returns [`Error::InvalidCommunicationControlType`] if `control_type` is an
    /// enhanced-address variant — those require a node identifier and must be built
    /// with [`new_with_node_id`](Self::new_with_node_id).
    pub const fn new(
        suppress_positive_response: bool,
        control_type: CommunicationControlType,
        communication_type: CommunicationType,
    ) -> Result<Self, Error> {
        if control_type.is_extended_address_variant() {
            return Err(Error::InvalidCommunicationControlType(control_type.value()));
        }
        Ok(Self {
            suppress_positive_response,
            control_type,
            communication_type,
            subnet: SubnetNumber::AllConnectedNetworks,
            node_id: None,
        })
    }

    /// Create a `CommunicationControlRequest` with enhanced address information.
    ///
    /// # Errors
    /// Returns [`Error::InvalidCommunicationControlType`] if `control_type` is not an
    /// enhanced-address variant — a node identifier is only carried by the
    /// `*WithEnhancedAddressInfo` variants.
    pub const fn new_with_node_id(
        suppress_positive_response: bool,
        control_type: CommunicationControlType,
        communication_type: CommunicationType,
        node_id: u16,
    ) -> Result<Self, Error> {
        if !control_type.is_extended_address_variant() {
            return Err(Error::InvalidCommunicationControlType(control_type.value()));
        }
        Ok(Self {
            suppress_positive_response,
            control_type,
            communication_type,
            subnet: SubnetNumber::AllConnectedNetworks,
            node_id: Some(node_id),
        })
    }

    /// The requested [`CommunicationControlType`].
    ///
    /// Private field with a getter, not a public field: `node_id` must be present exactly when
    /// this is an enhanced-address variant, so the two are set together through
    /// [`new`](Self::new) / [`new_with_node_id`](Self::new_with_node_id).
    #[must_use]
    pub const fn control_type(&self) -> CommunicationControlType {
        self.control_type
    }

    /// Target a particular subnet instead of all connected networks.
    ///
    /// Offered as a builder rather than a fourth constructor: the subnet is independent of the
    /// `control_type`/`node_id` pairing, so folding it into the constructors would double them
    /// without adding a rule to enforce.
    #[must_use]
    pub const fn with_subnet(mut self, subnet: SubnetNumber) -> Self {
        self.subnet = subnet;
        self
    }

    /// The [`CommunicationType`] the control applies to.
    ///
    /// This is the low nibble of the `communicationType` byte; see [`subnet`](Self::subnet)
    /// for the high nibble.
    #[must_use]
    pub const fn communication_type(&self) -> CommunicationType {
        self.communication_type
    }

    /// Which network the control applies to — the high nibble of the `communicationType` byte.
    #[must_use]
    pub const fn subnet(&self) -> SubnetNumber {
        self.subnet
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
        // Fuse the SPRMIB bit onto the sub-function at the wire boundary.
        let sub_function =
            SuppressablePositiveResponse::new(self.suppress_positive_response, self.control_type);
        let mut written = write_all(
            writer,
            &[
                u8::from(sub_function),
                (self.subnet.value() << 4) | u8::from(self.communication_type),
            ],
        )
        .map_err(Error::io)?;
        if let Some(id) = self.node_id {
            written += write_u16_be(writer, id).map_err(Error::io)?;
        }
        Ok(written)
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
        let subnet = SubnetNumber::try_from((buf[1] & SUBNET_MASK) >> 4)?;
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
                        suppress_positive_response: communication_enable
                            .suppress_positive_response(),
                        control_type: communication_enable.value(),
                        communication_type,
                        subnet,
                        node_id,
                    },
                    &buf[4..],
                ))
            }
            _ => Ok((
                Self {
                    suppress_positive_response: communication_enable.suppress_positive_response(),
                    control_type: communication_enable.value(),
                    communication_type,
                    subnet,
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
    ///
    /// Public here although [`CommunicationControlRequest::control_type`] is a getter: the
    /// response carries no `node_id`, so there is no cross-field invariant to protect.
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
        write_u8(writer, u8::from(self.control_type)).map_err(Error::io)
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

    #[test]
    fn the_communication_type_byte_carries_a_subnet_number() {
        // ISO 14229-1:2020 Annex B Table B.1 splits this byte: bits 0-1 are the message type,
        // bits 4-7 the subnet number (0 = the specified types on all connected networks,
        // 1-E = a specific subnet, F = the network the request arrived on). The whole byte was
        // matched against 0x00..=0x03, so 0xF3 -- "network management and normal messages on
        // the network this request came in on", a common real-world value -- was rejected.
        for (byte, message_type, subnet) in [
            (
                0x03u8,
                CommunicationType::NormalAndNetworkManagement,
                SubnetNumber::AllConnectedNetworks,
            ),
            (
                0x13,
                CommunicationType::NormalAndNetworkManagement,
                SubnetNumber::Specific(0x1),
            ),
            (0xE1, CommunicationType::Normal, SubnetNumber::Specific(0xE)),
            (
                0xF3,
                CommunicationType::NormalAndNetworkManagement,
                SubnetNumber::ReceivedOn,
            ),
            (
                0xF2,
                CommunicationType::NetworkManagement,
                SubnetNumber::ReceivedOn,
            ),
        ] {
            let wire = [0x28, 0x03, byte];
            let (req, _) = crate::Request::decode(&wire).unwrap();
            let crate::Request::CommunicationControl(inner) = req else {
                panic!("expected a CommunicationControl request, got {req:?}");
            };
            assert_eq!(
                inner.communication_type(),
                message_type,
                "wrong message type for {byte:#04X}"
            );
            assert_eq!(inner.subnet(), subnet, "wrong subnet for {byte:#04X}");

            let mut buf = [0u8; 8];
            let written = req.encode_to_slice(&mut buf).unwrap();
            assert_eq!(&buf[..written], &wire, "round trip failed for {byte:#04X}");
        }
    }

    #[test]
    fn the_reserved_bits_of_the_communication_type_byte_must_be_zero() {
        // Table B.1 marks bits 2-3 ISOSAEReserved, so a conformant client leaves them clear.
        for byte in [0x07u8, 0x0B, 0x0F, 0xFF] {
            assert!(
                CommunicationType::try_from(byte).is_err(),
                "{byte:#04X} sets a reserved bit but was accepted"
            );
        }
    }

    #[test]
    fn a_subnet_can_be_attached_without_a_second_constructor() {
        let req = CommunicationControlRequest::new(
            false,
            CommunicationControlType::DisableRxAndTx,
            CommunicationType::NormalAndNetworkManagement,
        )
        .unwrap()
        .with_subnet(SubnetNumber::ReceivedOn);
        assert_eq!(req.subnet(), SubnetNumber::ReceivedOn);

        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x03, 0xF3]);
        assert_encode_size_agrees(&req);
    }

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
        assert!(req.suppress_positive_response);
    }

    #[test]
    fn new_extra() {
        let req = CommunicationControlRequest::new(
            false,
            CommunicationControlType::EnableRxAndDisableTx,
            CommunicationType::NetworkManagement,
        )
        .unwrap();
        assert!(!req.suppress_positive_response);

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

    #[test]
    fn simple_response() {
        let bytes: [u8; 1] = [0x01];
        let (res, _) = <CommunicationControlResponse as Decode>::decode(&bytes).unwrap();
        assert_eq!(
            res.control_type,
            CommunicationControlType::EnableRxAndDisableTx
        );

        let mut buffer = [0u8; 4];
        let written = Encode::encode(&res, &mut buffer.as_mut_slice()).unwrap();
        assert_eq!(&buffer[..written], &bytes);
        assert_encode_size_agrees(&res);
    }
}

/// The serde/`OpenAPI` shape of [`CommunicationControlRequest`].
///
/// Deserializing routes through the constructors, so the rule that `node_id` is present exactly
/// when `control_type` is an enhanced-address variant -- the reason those fields are private --
/// still holds for a value that arrived as JSON.
#[cfg(any(feature = "serde", feature = "utoipa"))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
// With `utoipa` but not `serde` these fields are never read: they exist to give the schema its
// shape, which the derive reads at compile time rather than at run time.
#[cfg_attr(not(feature = "serde"), allow(dead_code))]
struct CommunicationControlRepr {
    suppress_positive_response: bool,
    control_type: CommunicationControlType,
    communication_type: CommunicationType,
    subnet: SubnetNumber,
    node_id: Option<u16>,
}

#[cfg(feature = "serde")]
impl TryFrom<CommunicationControlRepr> for CommunicationControlRequest {
    type Error = Error;

    fn try_from(repr: CommunicationControlRepr) -> Result<Self, Error> {
        let request = match repr.node_id {
            Some(node_id) => CommunicationControlRequest::new_with_node_id(
                repr.suppress_positive_response,
                repr.control_type,
                repr.communication_type,
                node_id,
            )?,
            None => CommunicationControlRequest::new(
                repr.suppress_positive_response,
                repr.control_type,
                repr.communication_type,
            )?,
        };
        Ok(request.with_subnet(repr.subnet))
    }
}

#[cfg(feature = "serde")]
impl From<CommunicationControlRequest> for CommunicationControlRepr {
    fn from(request: CommunicationControlRequest) -> Self {
        Self {
            suppress_positive_response: request.suppress_positive_response,
            control_type: request.control_type(),
            communication_type: request.communication_type(),
            subnet: request.subnet(),
            node_id: request.node_id(),
        }
    }
}

#[cfg(feature = "utoipa")]
impl utoipa::PartialSchema for CommunicationControlRequest {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        <CommunicationControlRepr as utoipa::PartialSchema>::schema()
    }
}

#[cfg(feature = "utoipa")]
impl utoipa::ToSchema for CommunicationControlRequest {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("CommunicationControlRequest")
    }
}
