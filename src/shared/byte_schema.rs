//! `OpenAPI` schemas for the types that serialize as a single protocol byte.
//!
//! Several public types are one wire byte with a range invariant. Their `serde` impls go
//! through `u8` (via `serde(try_from = "u8", into = "u8")`) so that deserializing cannot build a
//! value their constructor would reject. A derived `ToSchema` would still describe the Rust
//! shape — an object with nibble properties, or a `oneOf` over variant names — and so disagree
//! with what `serde` actually reads and writes. These impls keep the two in step.

/// Implement `PartialSchema`/`ToSchema` for types whose serialized form is a `u8`.
macro_rules! byte_schema {
    ($($ty:ident),* $(,)?) => {
        $(
            impl utoipa::PartialSchema for crate::$ty {
                fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                    <u8 as utoipa::PartialSchema>::schema()
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
    CommunicationControlType,
    CommunicationType,
    DataFormatIdentifier,
    DtcSettingType,
    SecurityAccessType,
    SubnetNumber,
);
