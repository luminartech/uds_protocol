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
