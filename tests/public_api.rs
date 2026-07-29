//! Checks the public API from outside the crate.
//!
//! An integration test is a separate crate, so `#[non_exhaustive]` applies here exactly as it
//! does for a downstream user. Inline `#[cfg(test)]` modules cannot check this: inside the
//! defining crate a `#[non_exhaustive]` struct literal compiles fine, so a type that is
//! impossible for anyone else to build still looks constructible from there.

use uds_protocol::{
    DirSizePayload, DtcFaultDetectionCounterRecord, DtcRecord, FileOperationMode, FileSizePayload,
    NamePayload, PositionPayload, SentDataPayload, SizePayload,
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
    let _ = NamePayload::new(FileOperationMode::AddFile, "/a");
    let _ = SentDataPayload::new(&[0x02, 0x00]);
    let _ = FileSizePayload::new(0xC350, 0x7530);
    let _ = DirSizePayload::new(0x20);
    let _ = PositionPayload::new(0x10);
    let _ = DtcFaultDetectionCounterRecord::new(DtcRecord::new(0, 0, 0), 0);
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
