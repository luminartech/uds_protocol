//! `OpenAPI` schemas for the types that serialize as a single protocol byte.
//!
//! Several public types are one wire byte with a range invariant. Their `serde` impls go
//! through `u8` (via `serde(try_from = "u8", into = "u8")` or `serde(from = "u8", into = "u8")`)
//! so that deserializing cannot build a value their constructor would reject. A derived
//! `ToSchema` would still describe the Rust shape — an object with nibble properties, or a
//! `oneOf` over variant names — and so disagree with what `serde` actually reads and writes.
//! These impls keep the two in step.
//!
//! Each schema carries the actual accepted range rather than `u8`'s bare
//! `{"type": "integer", "minimum": 0}`. Delegating to `<u8 as PartialSchema>` advertised
//! `0..=2^31`, so a generated client emitted an `i32` and discovered the real bound as a runtime
//! rejection. The whole reason these types exist is that their bytes have ranges; the schema is
//! the one place that has to say so.

/// Implement `PartialSchema`/`ToSchema` for a type whose serialized form is a single `u8`.
///
/// `max` is the largest byte the type's own `serde` entry point accepts, so the schema and the
/// deserializer cannot drift; `description` is the published schema description and must spell out
/// any gaps, because `minimum`/`maximum` alone cannot express a non-contiguous set.
macro_rules! byte_schema {
    ($($ty:ident { max: $max:expr, description: $desc:literal }),* $(,)?) => {
        $(
            impl utoipa::PartialSchema for crate::$ty {
                fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                    utoipa::openapi::ObjectBuilder::new()
                        .schema_type(utoipa::openapi::schema::SchemaType::Type(
                            utoipa::openapi::Type::Integer,
                        ))
                        .minimum(Some(0.0))
                        .maximum(Some(f64::from($max)))
                        .description(Some($desc))
                        .into()
                }
            }

            impl utoipa::ToSchema for crate::$ty {
                fn name() -> std::borrow::Cow<'static, str> {
                    std::borrow::Cow::Borrowed(stringify!($ty))
                }
            }
        )*
    };
}

byte_schema!(
    AddressAndLengthFormatIdentifier {
        max: 0x45u8,
        description: "An addressAndLengthFormatIdentifier byte: memorySize width in the high \
                      nibble (1 to 4 bytes), memoryAddress width in the low nibble (1 to 5). So \
                      0x44 declares four bytes each. ISO 14229-1:2020 Annex H Table H.1 runs \
                      from 0x11 to 0x45 and marks a zero nibble, or a size above 4, \
                      not applicable. The accepted set is not contiguous: 0x15, 0x25, 0x35 and \
                      0x40 to 0x43 fall inside these bounds but are rejected."
    },
    CommunicationControlType {
        max: 0x7Fu8,
        description: "A CommunicationControl sub-function byte, 0x00 to 0x7F. Bit 7 is the \
                      suppressPosRspMsgIndicationBit and is carried separately, so a value with \
                      it set is rejected."
    },
    DataFormatIdentifier {
        max: 0xFFu8,
        description: "A dataFormatIdentifier byte: compression method in the high nibble, \
                      encryption method in the low nibble. 0x00 means neither. Every byte is a \
                      valid encoding; non-zero values are vehicle-manufacturer specific."
    },
    DtcFormatIdentifier {
        max: 0xFFu8,
        description: "A DTCFormatIdentifier byte (ISO 14229-1:2020 Table 331). 0x00 to 0x04 are \
                      defined formats; every other byte classifies as ISOSAEReserved rather than \
                      being rejected."
    },
    DtcSettingType {
        max: 0x7Eu8,
        description: "A DTCSettingType byte (ISO 14229-1:2020 Table 128). Only 0x01 (on), 0x02 \
                      (off), 0x40 to 0x5F (vehicle-manufacturer specific) and 0x60 to 0x7E \
                      (system-supplier specific) are accepted; 0x00, 0x03 to 0x3F and 0x7F are \
                      reserved and rejected. The set is not contiguous, so the minimum and \
                      maximum here are looser than the real constraint."
    },
    FileOperationMode {
        max: 0xFFu8,
        description: "A RequestFileTransfer modeOfOperation byte (ISO 14229-1:2020 Table 485). \
                      0x01 to 0x06 are defined; every other byte classifies as ISOSAEReserved \
                      rather than being rejected."
    },
    FunctionalGroupIdentifier {
        max: 0xFFu8,
        description: "A functionalGroupIdentifier byte (ISO 14229-1:2020 Annex D). Every byte \
                      classifies: undefined values become ISOSAEReserved rather than being \
                      rejected."
    },
    SecurityAccessType {
        max: 0x7Fu8,
        description: "A SecurityAccess sub-function byte, 0x00 to 0x7F. Odd values request a \
                      seed and even values send a key. Bit 7 is the \
                      suppressPosRspMsgIndicationBit and is carried separately, so a value with \
                      it set is rejected."
    },
    SubnetNumber {
        max: 0x0Fu8,
        description: "A subnetNumber nibble, 0x00 to 0x0F (ISO 14229-1:2020 Annex B Table B.1). \
                      0x00 addresses the receiving node including all connected networks, 0x0F \
                      the network the request arrived on, and 0x01 to 0x0E a specific subnet. It \
                      occupies the high nibble of the communicationType byte, so anything wider \
                      than a nibble is rejected."
    },
);
