//! Checks the public API from outside the crate.
//!
//! An integration test is a separate crate, so `#[non_exhaustive]` applies here exactly as it
//! does for a downstream user. Inline `#[cfg(test)]` modules cannot check this: inside the
//! defining crate a `#[non_exhaustive]` struct literal compiles fine, so a type that is
//! impossible for anyone else to build still looks constructible from there.

use uds_protocol::{
    DirSizePayload, DtcFaultDetectionCounterRecord, DtcFormatIdentifier, DtcRecord,
    FileSizePayload, NamePayload, PositionPayload, SentDataPayload, SizePayload,
};

#[test]
fn dtc_fault_detection_counter_record_is_constructible_downstream() {
    // This is the `Item` type of a public iterator and is re-exported at the crate root so
    // callers can name it. Without a constructor, `#[non_exhaustive]` made the only way to
    // obtain one decoding bytes through `DtcFaultDetectionIter` — E0639 out here.
    let record = DtcFaultDetectionCounterRecord::new(DtcRecord::new(0x01, 0x02, 0x03), 0x2A);
    assert_eq!(record.dtc_record, DtcRecord::new(0x01, 0x02, 0x03));
    assert_eq!(record.dtc_fault_detection_counter, 0x2A);
}

#[test]
fn every_non_exhaustive_payload_type_has_a_reachable_constructor() {
    // One commit added `#[non_exhaustive]` to seven public structs with `pub` fields. Six had
    // a constructor and one did not, which made it unbuildable outside the crate. Pin all
    // seven so the next `#[non_exhaustive]` cannot reintroduce the gap silently.
    let _ = SizePayload::new(0xC350, 0x7530);
    let _ = NamePayload::new("/a");
    let _ = SentDataPayload::new(&[0x02, 0x00]);
    let _ = FileSizePayload::new(0xC350, 0x7530);
    let _ = DirSizePayload::new(0x20);
    let _ = PositionPayload::new(0x10);
    let _ = DtcFaultDetectionCounterRecord::new(DtcRecord::new(0, 0, 0), 0);
}

#[test]
fn a_reserved_variant_cannot_alias_a_named_one() {
    // `PartialEq` on these enums is variant equality, not wire equality, so being able to name a
    // reserved variant with a byte that a named variant also encodes creates a trap:
    // `DtcFormatIdentifier::IsoSaeReserved(0x01) != Iso14229_1DtcFormat` even though both encode
    // 0x01. `#[non_exhaustive]` on the byte-carrying variants makes that unconstructible from
    // out here, so a value can only be obtained through the classifier, which never aliases.
    //
    // Constructing one is a compile error, checked by `tests/ui` conventions in spirit; what this
    // test pins is the positive half — the classifier and the byte accessor agree, and equality
    // matches the wire for every byte.
    for byte in 0x00..=0xFFu8 {
        let format = DtcFormatIdentifier::from(byte);
        assert_eq!(format.value(), byte, "classifier lost the byte {byte:#04X}");
        assert_eq!(
            format == DtcFormatIdentifier::Iso14229_1DtcFormat,
            byte == 0x01,
            "equality disagreed with the wire for {byte:#04X}"
        );
    }
}

/// A `derive(Deserialize)` ignores field visibility and range checks, so it is a second,
/// unvalidated constructor for every type whose invariant lives in its constructor. These check
/// that deserializing cannot build a value `new`/`try_from` would have rejected.
#[cfg(feature = "serde")]
mod serde_cannot_bypass_validation {
    use uds_protocol::{
        CommunicationType, DataFormatIdentifier, DtcSettingType, SecurityAccessLevel, SubnetNumber,
    };

    #[test]
    fn security_access_level_rejects_a_value_that_collides_with_sprmib() {
        // A level must fit in 0x00..=0x7F, because bit 7 of the sub-function byte is SPRMIB.
        // Deserializing 0xFF produced a level that encoded to a byte which decoded back as a
        // *different* level with suppression set — silent semantic corruption.
        assert!(serde_json::from_str::<SecurityAccessLevel>("255").is_err());
        let level: SecurityAccessLevel = serde_json::from_str("127").unwrap();
        assert_eq!(level.value(), 0x7F);
    }

    #[test]
    fn data_format_identifier_rejects_an_over_wide_nibble() {
        // `new` returns Err when either method exceeds 0x0F; JSON must not be a way around it.
        assert!(
            serde_json::from_str::<DataFormatIdentifier>(
                r#"{"encryption_method":255,"compression_method":255}"#
            )
            .is_err()
        );
        let dfi: DataFormatIdentifier = serde_json::from_str("33").unwrap();
        assert_eq!(dfi.compression_method(), 0x02);
        assert_eq!(dfi.encryption_method(), 0x01);
    }

    #[test]
    fn range_checked_enums_reject_out_of_range_bytes() {
        // Each of these has a `try_from` that range-checks the byte; the derived impl let a
        // caller name the reserved variant directly and skip it.
        assert!(serde_json::from_str::<SubnetNumber>("16").is_err());
        assert!(serde_json::from_str::<DtcSettingType>("0").is_err());
        assert!(serde_json::from_str::<CommunicationType>("7").is_err());

        assert_eq!(
            serde_json::from_str::<SubnetNumber>("15").unwrap(),
            SubnetNumber::ReceivedOn
        );
    }

    #[test]
    fn tester_present_cannot_be_given_a_sub_function_that_collides_with_sprmib() {
        use uds_protocol::TesterPresentRequest;

        // The sub-function field is private precisely so a caller cannot mint a value with bit
        // 7 set, which is SPRMIB. The derived impl exposed the private field by name and
        // accepted any variant payload.
        assert!(
            serde_json::from_str::<TesterPresentRequest>(
                r#"{"suppress_positive_response":false,"sub_function":128}"#
            )
            .is_err()
        );
        // A reserved-but-legal byte is still accepted, because decode must round-trip it.
        let req: TesterPresentRequest =
            serde_json::from_str(r#"{"suppress_positive_response":false,"sub_function":66}"#)
                .unwrap();
        assert_eq!(req.sub_function(), 0x42);
    }

    #[test]
    fn communication_control_cannot_be_given_a_contradictory_node_id() {
        use uds_protocol::CommunicationControlRequest;

        // `node_id` must be present exactly when `control_type` is an enhanced-address
        // variant. That rule is the entire reason both fields are private, and deserializing
        // ignored it: an enhanced type with a null node_id produced a request that encoded to
        // two bytes, which this crate's own decoder then rejected.
        assert!(
            serde_json::from_str::<CommunicationControlRequest>(
                r#"{"suppress_positive_response":false,"control_type":5,"communication_type":1,"subnet":0,"node_id":null}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<CommunicationControlRequest>(
                r#"{"suppress_positive_response":false,"control_type":3,"communication_type":1,"subnet":0,"node_id":10}"#
            )
            .is_err()
        );

        let req: CommunicationControlRequest = serde_json::from_str(
            r#"{"suppress_positive_response":false,"control_type":5,"communication_type":1,"subnet":15,"node_id":10}"#
        )
        .unwrap();
        assert_eq!(req.node_id(), Some(10));
        assert_eq!(req.subnet(), SubnetNumber::ReceivedOn);
    }

    #[test]
    fn transfer_requests_cannot_be_given_a_width_that_truncates_the_value() {
        use uds_protocol::RequestDownloadRequest;

        // The width nibbles are private so they cannot fall out of step with the values. A
        // deserialized 8-byte address with a 1-byte declared width silently truncated to one
        // byte on the wire.
        assert!(
            serde_json::from_str::<RequestDownloadRequest>(
                r#"{"data_format_identifier":0,"memory_address":18446744073709551615,"memory_address_length":1,"memory_size":16,"memory_size_length":1}"#
            )
            .is_err()
        );
        // A width outside Table H.1 is rejected too.
        assert!(
            serde_json::from_str::<RequestDownloadRequest>(
                r#"{"data_format_identifier":0,"memory_address":16,"memory_address_length":9,"memory_size":16,"memory_size_length":0}"#
            )
            .is_err()
        );

        // A wider-than-minimal declaration survives the round trip, so the ALFID a server was
        // sent is the one it gets back.
        let json = r#"{"data_format_identifier":17,"memory_address":6299648,"memory_address_length":3,"memory_size":65535,"memory_size_length":3}"#;
        let req: RequestDownloadRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.memory_address(), 0x0060_2000);
        assert_eq!(serde_json::to_string(&req).unwrap(), json);
    }

    #[test]
    fn no_crate_private_type_appears_in_the_serialized_form() {
        use uds_protocol::{RequestDownloadRequest, TesterPresentRequest};

        // `ZeroSubFunction` is module-private and `MemoryFormatIdentifier` is `pub(crate)`, yet
        // both used to be named components of the serialized shape — and of the generated
        // OpenAPI schema, giving client generators a type downstream Rust cannot reference.
        let tester = serde_json::to_string(&TesterPresentRequest::new(false)).unwrap();
        assert!(
            !tester.contains("zero_sub_function"),
            "leaked a private field name: {tester}"
        );
        assert!(tester.contains("sub_function"), "got {tester}");

        let download =
            RequestDownloadRequest::new(DataFormatIdentifier::NONE, 0x1234, 0x10).unwrap();
        let json = serde_json::to_string(&download).unwrap();
        assert!(
            !json.contains("address_and_length_format_identifier"),
            "leaked a private field name: {json}"
        );
        assert!(json.contains("memory_address_length"), "got {json}");
    }

    #[test]
    fn the_serialized_form_is_the_wire_byte() {
        // Round-tripping through serde must agree with the protocol's own representation,
        // otherwise `serde(try_from)` on the way in and a struct shape on the way out would
        // disagree with each other.
        assert_eq!(
            serde_json::to_string(&SubnetNumber::ReceivedOn).unwrap(),
            "15"
        );
        assert_eq!(serde_json::to_string(&DtcSettingType::Off).unwrap(), "2");
        assert_eq!(
            serde_json::to_string(&SecurityAccessLevel::new(0x11).unwrap()).unwrap(),
            "17"
        );
    }
}
