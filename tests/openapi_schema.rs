//! Checks the generated `OpenAPI` document against what `serde` actually does.
//!
//! The `utoipa` half of this crate is hand-written for every type whose serialized form is a
//! single protocol byte, and for the four composite requests that deserialize through a repr. Three
//! defects shipped in that code because nothing here asserted anything about it:
//!
//! - the document had four dangling `$ref`s, where the derive it replaced had none, because a
//!   hand-written `ToSchema` must forward `schemas()` and none of them did;
//! - two schema descriptions were implementation notes rather than API documentation, one of them
//!   naming a module-private type, and one had lost its verb because utoipa's derive silently
//!   drops `#[doc = concat!(..)]`;
//! - every byte schema advertised `0..=2^31` while its deserializer rejected anything above
//!   `0x7F`, or `0x0F`, so a generated client would emit an `i32` and get a runtime rejection.
//!
//! A schema that lies is worse than a verbose one, so these are checked, not eyeballed.

#![cfg(all(feature = "utoipa", feature = "serde"))]
// utoipa's `OpenApi` derive expands to `iter().for_each(..)`, so this lint fires on generated
// code that this file cannot change.
#![allow(clippy::needless_for_each)]

use std::collections::BTreeSet;

use utoipa::{OpenApi, PartialSchema};

#[derive(OpenApi)]
#[openapi(components(schemas(
    // The four composites with hand-written schemas that delegate to a repr.
    uds_protocol::CommunicationControlRequest,
    uds_protocol::TesterPresentRequest,
    uds_protocol::TesterPresentResponse,
    uds_protocol::RequestDownloadRequest,
    uds_protocol::RequestUploadRequest,
)))]
struct Api;

fn walk(value: &serde_json::Value, found: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                if key == "$ref" {
                    if let Some(name) = val
                        .as_str()
                        .and_then(|s| s.strip_prefix("#/components/schemas/"))
                    {
                        found.insert(name.to_owned());
                    }
                }
                walk(val, found);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                walk(item, found);
            }
        }
        _ => {}
    }
}

/// Every `$ref` in `doc` that points at `#/components/schemas/...`.
fn referenced_schemas(doc: &serde_json::Value) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    walk(doc, &mut found);
    found
}

#[test]
fn the_document_has_no_dangling_refs() {
    let json = Api::openapi().to_json().expect("serializes");
    let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    let schemas = doc["components"]["schemas"]
        .as_object()
        .expect("components.schemas");

    let referenced = referenced_schemas(&doc);
    let dangling: Vec<&String> = referenced
        .iter()
        .filter(|name| !schemas.contains_key(name.as_str()))
        .collect();

    assert!(
        dangling.is_empty(),
        "these schemas are referenced but never defined, so the document does not resolve: \
         {dangling:?}. A hand-written `ToSchema` must forward `schemas()` — it defaults to a \
         no-op, and only the derive overrides it."
    );

    // And the children really are pulled in, so the assertion above is not passing vacuously on
    // a document that happens to contain no `$ref`s at all.
    assert!(
        schemas.contains_key("SubnetNumber"),
        "expected the composite request to register its child schemas, got {:?}",
        schemas.keys().collect::<Vec<_>>()
    );
}

/// `maximum` on each byte schema must be the largest byte `serde` actually accepts.
///
/// This is the assertion that keeps the two from drifting: it does not hard-code a bound, it
/// discovers it by deserializing all 256 bytes and compares that to what the schema advertises.
macro_rules! assert_schema_bound_matches_serde {
    ($ty:ty, $name:literal) => {{
        let accepted: Vec<u8> = (0u8..=0xFF)
            .filter(|b| serde_json::from_str::<$ty>(&b.to_string()).is_ok())
            .collect();
        let highest_accepted = *accepted
            .last()
            .expect(concat!($name, " accepts no byte at all"));

        let schema = serde_json::to_value(<$ty as PartialSchema>::schema()).expect("serializes");
        let advertised = schema
            .get("maximum")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| {
                panic!(
                    "{} has no `maximum`, so it advertises the whole i32 range while rejecting \
                     everything above {:#04X}",
                    $name, highest_accepted
                )
            });

        assert_eq!(
            advertised,
            u64::from(highest_accepted),
            "{}: schema says maximum {}, but serde accepts up to {:#04X}",
            $name,
            advertised,
            highest_accepted
        );
        assert_eq!(
            schema.get("type").and_then(serde_json::Value::as_str),
            Some("integer"),
            "{} serializes as a bare byte, so its schema must be an integer",
            $name
        );

        // A contiguous `minimum..=maximum` cannot express a set with gaps, so any type with gaps
        // has to say so in prose or the schema is still lying.
        let description = schema
            .get("description")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{} has no schema description", $name));
        let lowest_accepted = accepted[0];
        let contiguous =
            accepted.len() == usize::from(highest_accepted) - usize::from(lowest_accepted) + 1;
        if !contiguous {
            assert!(
                description.contains("not contiguous"),
                "{} accepts a set with gaps ({} of {} bytes in range), so its description must \
                 say so: {description:?}",
                $name,
                accepted.len(),
                usize::from(highest_accepted) - usize::from(lowest_accepted) + 1
            );
        }
    }};
}

#[test]
fn every_byte_schema_advertises_the_range_serde_enforces() {
    use uds_protocol::{
        CommunicationControlType, DataFormatIdentifier, DtcFormatIdentifier, DtcSettingType,
        FileOperationMode, FunctionalGroupIdentifier, SecurityAccessType, SubnetNumber,
    };

    assert_schema_bound_matches_serde!(CommunicationControlType, "CommunicationControlType");
    assert_schema_bound_matches_serde!(DataFormatIdentifier, "DataFormatIdentifier");
    assert_schema_bound_matches_serde!(DtcFormatIdentifier, "DtcFormatIdentifier");
    assert_schema_bound_matches_serde!(DtcSettingType, "DtcSettingType");
    assert_schema_bound_matches_serde!(FileOperationMode, "FileOperationMode");
    assert_schema_bound_matches_serde!(FunctionalGroupIdentifier, "FunctionalGroupIdentifier");
    assert_schema_bound_matches_serde!(SecurityAccessType, "SecurityAccessType");
    assert_schema_bound_matches_serde!(SubnetNumber, "SubnetNumber");
}

#[test]
fn no_schema_description_leaks_a_private_type_or_is_empty() {
    // Types a downstream crate cannot name. A schema that mentions one hands a client author a
    // dead end, and these two are the whole reason the reprs exist — so leaking them into the
    // published description defeats the point.
    const PRIVATE: [&str; 2] = ["ZeroSubFunction", "MemoryFormatIdentifier"];

    let json = Api::openapi().to_json().expect("serializes");
    let doc: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    let schemas = doc["components"]["schemas"]
        .as_object()
        .expect("components.schemas");

    for (name, schema) in schemas {
        let description = schema
            .get("description")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("{name} has no schema description"));

        assert!(
            !description.trim().is_empty(),
            "{name} has an empty schema description"
        );
        for private in PRIVATE {
            assert!(
                !description.contains(private),
                "{name}'s published description names the private type `{private}`: \
                 {description:?}"
            );
        }
        // The verb-less description this test exists to catch began with a bare newline: utoipa
        // dropped the `#[doc = concat!(..)]` fragment carrying the sentence's subject and left the
        // blank line that had followed it. A leading blank line is that exact signature.
        assert!(
            !description.starts_with(char::is_whitespace),
            "{name}'s description starts with whitespace, which is what a dropped \
             `#[doc = concat!(..)]` fragment leaves behind: {description:?}"
        );
    }
}
