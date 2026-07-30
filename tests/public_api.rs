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
fn a_tester_can_still_request_an_unmodeled_read_dtc_information_sub_function() {
    // Sealing `IsoSaeReserved` keeps bit 7 (SPRMIB) out of the sub-function value, but it also
    // made `IsoSaeReserved(byte)` unwritable out here — so a tester could no longer originate a
    // request for a report type the crate has not implemented. `try_reserved` is the door.
    use uds_protocol::{Encode, ReadDtcInfoRequest, ReadDtcInfoSubFunction};

    let sub = ReadDtcInfoSubFunction::try_reserved(0x57).expect("0x57 has bit 7 clear");
    assert_eq!(sub.value(), 0x57);

    let mut buf = [0u8; 4];
    let req = ReadDtcInfoRequest::new(false, sub);
    let written = req.encode_to_slice(&mut buf).expect("encodes");
    assert_eq!(&buf[..written], &[0x57]);

    // ...and with the positive response suppressed, the flag is fused back in as bit 7.
    let suppressed = ReadDtcInfoRequest::new(true, sub);
    let written = suppressed.encode_to_slice(&mut buf).expect("encodes");
    assert_eq!(&buf[..written], &[0xD7]);

    // Bit 7 is SPRMIB, so it must not be smuggled in as part of the sub-function value:
    // 0x80 would otherwise encode as a suppressed 0x00.
    assert!(ReadDtcInfoSubFunction::try_reserved(0x80).is_err());
    assert!(ReadDtcInfoSubFunction::try_reserved(0xD7).is_err());
}

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
fn the_payload_data_bags_are_buildable_with_a_struct_literal() {
    // These five carry no invariant: every field is `pub`, and their encoders derive the wire
    // widths from the values, so no combination fails. The crate's rule is to encapsulate a
    // request/response struct *iff* it bears an invariant, so these are plain data bags and a
    // struct literal must work from out here. They were briefly `#[non_exhaustive]` purely for
    // symmetry with the invariant-bearing types, which cost a downstream the struct literal
    // (E0639) and bought nothing -- `DtcFaultDetectionCounterRecord` became unbuildable
    // entirely and took two follow-up commits to repair.
    let _ = SizePayload {
        file_size_uncompressed: 0xC350,
        file_size_compressed: 0x7530,
    };
    let _ = FileSizePayload {
        file_size_uncompressed: 0xC350,
        file_size_compressed: 0x7530,
    };
    let _ = DirSizePayload {
        dir_info_length: 0x20,
    };
    let _ = PositionPayload {
        file_position: 0x10,
    };
    let _ = DtcFaultDetectionCounterRecord {
        dtc_record: DtcRecord::new(0, 0, 0),
        dtc_fault_detection_counter: 0x2A,
    };

    // The convenience constructors stay, and agree with the literals.
    assert_eq!(
        SizePayload::new(0xC350, 0x7530),
        SizePayload {
            file_size_uncompressed: 0xC350,
            file_size_compressed: 0x7530,
        }
    );
}

#[test]
fn the_two_length_bounded_payloads_reject_an_unencodable_value() {
    // These two are the ones that genuinely bear an invariant, so they keep both
    // `#[non_exhaustive]` and a fallible constructor: the wire field carrying the length is one
    // byte for `SentDataPayload` and two for `NamePayload`, and `encode` used to be the first
    // thing to notice an over-long value. A struct literal here is correctly a compile error.
    assert!(SentDataPayload::new(&[0u8; 255]).is_ok());
    assert!(SentDataPayload::new(&[0u8; 256]).is_err());

    let short = NamePayload::new("/a").expect("a two-character name fits");
    assert_eq!(short.file_path_and_name, "/a");
}

#[cfg(feature = "serde")]
#[test]
fn serde_cannot_build_a_reserved_variant_that_aliases_a_named_one() {
    // This is what the variant seals were supposed to prevent and did not: with a plain derived
    // `Deserialize`, `{"IsoSaeReserved":1}` produced a value whose `value()` is 0x01 -- the same
    // byte `Iso14229_1DtcFormat` encodes -- yet compared unequal to it. `#[non_exhaustive]` on
    // the variant blocked the Rust struct literal and did nothing about serde, so it bought
    // pattern-matching friction and no guarantee. Routing serde through `u8` is what actually
    // closes it, for every byte, and it lets the variant be named and destructured again.
    let round: DtcFormatIdentifier = serde_json::from_str("1").expect("a bare byte");
    assert_eq!(round, DtcFormatIdentifier::Iso14229_1DtcFormat);
    assert_eq!(serde_json::to_string(&round).unwrap(), "1");

    // The old escape route no longer parses at all.
    assert!(serde_json::from_str::<DtcFormatIdentifier>(r#"{"IsoSaeReserved":1}"#).is_err());

    // Every byte survives the round trip through the classifier, so no aliasing state exists.
    for byte in 0x00..=0xFFu8 {
        let via_serde: DtcFormatIdentifier =
            serde_json::from_str(&byte.to_string()).expect("every byte classifies");
        assert_eq!(via_serde, DtcFormatIdentifier::from(byte));
        assert_eq!(via_serde, byte, "PartialEq<u8> must be wire equality");
    }
}

#[test]
fn a_reserved_variant_is_nameable_and_destructurable_downstream() {
    // The seal made both `IsoSaeReserved(b)` and `IsoSaeReserved(..)` E0603 out here, so a
    // downstream could not even read the byte by matching -- only `value()` worked. For an
    // audience new to Rust that is a compile error whose text says nothing about the way out.
    let reserved = DtcFormatIdentifier::from(0xAA);
    match reserved {
        DtcFormatIdentifier::IsoSaeReserved(byte) => assert_eq!(byte, 0xAA),
        other => panic!("0xAA should classify as reserved, got {other:?}"),
    }
    assert_eq!(DtcFormatIdentifier::IsoSaeReserved(0xAA), reserved);
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
    #[test]
    fn tester_present_sub_function_stays_inside_the_seven_bit_range() {
        use uds_protocol::{TesterPresentRequest, TesterPresentResponse};

        // Bit 7 of the sub-function byte is the SPRMIB and is carried in its own field, so the
        // sub-function itself is a 7-bit value. This used to be enforced by a mirror struct whose
        // `TryFrom` called the classifier; it is now enforced by the classifier being serde's
        // entry point for the field. Same guarantee, so the same bytes must be refused.
        for byte in 0x80..=0xFFu8 {
            let json = format!(r#"{{"suppress_positive_response":false,"sub_function":{byte}}}"#);
            assert!(
                serde_json::from_str::<TesterPresentRequest>(&json).is_err(),
                "sub_function {byte:#04X} has bit 7 set and must be rejected"
            );
            let json = format!(r#"{{"sub_function":{byte}}}"#);
            assert!(
                serde_json::from_str::<TesterPresentResponse>(&json).is_err(),
                "response sub_function {byte:#04X} has bit 7 set and must be rejected"
            );
        }

        // And every legal byte round-trips, reserved values included, so the range check did not
        // become a blanket rejection.
        for byte in 0x00..=0x7Fu8 {
            let json = format!(r#"{{"suppress_positive_response":true,"sub_function":{byte}}}"#);
            let req: TesterPresentRequest =
                serde_json::from_str(&json).expect("0x00..=0x7F is legal");
            assert_eq!(req.sub_function(), byte);
            assert!(req.suppress_positive_response);
            assert_eq!(serde_json::to_string(&req).unwrap(), json);
        }
    }

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
                r#"{"suppress_positive_response":false,"control_type":5,"communication_type":"Normal","subnet":0,"node_id":null}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<CommunicationControlRequest>(
                r#"{"suppress_positive_response":false,"control_type":3,"communication_type":"Normal","subnet":0,"node_id":10}"#
            )
            .is_err()
        );

        let req: CommunicationControlRequest = serde_json::from_str(
            r#"{"suppress_positive_response":false,"control_type":5,"communication_type":"Normal","subnet":15,"node_id":10}"#
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
