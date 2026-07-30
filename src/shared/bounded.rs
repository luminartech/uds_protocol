//! `serde` field validators for the borrowed fields that carry a length invariant.
//!
//! A few request and response fields are length-bounded because the wire field that declares
//! their length is one or two bytes wide, or because ISO marks the first byte mandatory. Their
//! constructors enforce that, but `derive(Deserialize)` ignores constructors — so without these,
//! deserializing is a second way in that produces a value `encode` rejects, or worse, one that
//! encodes to a frame this crate's own decoder refuses.
//!
//! These are field-level `deserialize_with` hooks rather than whole-type mirror structs: every
//! invariant here is a property of a single field, so there is nothing for an aggregate to
//! coordinate. Two mirror structs were used for a single-field invariant elsewhere in this crate
//! and cost about a hundred lines to say what one attribute says.

use serde::{Deserialize, Deserializer, de::Error as _};

/// Deserialize a byte slice that must not be empty.
///
/// ISO 14229-1:2020 Table 277 marks `dataRecord` byte #1 mandatory for `WriteDataByIdentifier`,
/// and Figure 26 states a four-byte minimum frame, so an empty record encodes to a message the
/// decoder answers NRC 0x13 to.
pub(crate) fn non_empty_bytes<'de, D>(deserializer: D) -> Result<&'de [u8], D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = <&'de [u8]>::deserialize(deserializer)?;
    if bytes.is_empty() {
        return Err(D::Error::custom(
            "the data record must contain at least one byte",
        ));
    }
    Ok(bytes)
}

/// Deserialize a byte slice whose length must fit the one-byte field that declares it.
pub(crate) fn bytes_within_u8_len<'de, D>(deserializer: D) -> Result<&'de [u8], D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = <&'de [u8]>::deserialize(deserializer)?;
    if bytes.len() > u8::MAX as usize {
        return Err(D::Error::custom(
            "maxNumberOfBlockLength is longer than the one-byte length field can declare",
        ));
    }
    Ok(bytes)
}

/// Deserialize a string whose length must fit the two-byte field that declares it.
pub(crate) fn str_within_u16_len<'de, D>(deserializer: D) -> Result<&'de str, D::Error>
where
    D: Deserializer<'de>,
{
    let text = <&'de str>::deserialize(deserializer)?;
    if text.len() > u16::MAX as usize {
        return Err(D::Error::custom(
            "filePathAndName is longer than the two-byte length field can declare",
        ));
    }
    Ok(text)
}
