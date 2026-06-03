# API Exposure & Consistency — Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliberately expose every intended-public type from the crate root and complete the two codec-incomplete descriptors (`ReadDTCInfoRequest` gains `Encode`; `WriteDataByIdentifierResponse` gains `Decode`), so the `no_std` breaking change ships as one cohesive chunk.

**Architecture:** Five independent commits. (1) Replace the `pub use {common,services}::*` globs in `lib.rs` with explicit named re-exports. (2) Add 1-byte `Encode` impls to the five DTC parameter types `ReadDTCInfoSubFunction` needs, fixing a latent `todo!()` panic in `FunctionalGroupIdentifier::value()`. (3) Give `ReadDTCInfoSubFunction` (and a delegating `ReadDTCInfoRequest`) a faithful per-variant `Encode`. (4) Give `WriteDataByIdentifierResponse` a 2-byte `Decode`. (5) Add crate-root integration tests proving the completed types round-trip through the public API. No `Request`/`Response` enum shapes change — that is Phase 2.

**Tech Stack:** Rust, `no_std` + `no_alloc` (`alloc`/`std` additive), `embedded_io::Write` for encoding, borrowed `&[u8]` for decoding. Test helper `assert_encode_size_agrees` in `src/test_util.rs`.

**Spec:** `docs/superpowers/specs/2026-06-03-api-exposure-and-consistency-design.md`

---

## Conventions for every task

- Local per-task verification: `cargo test --all-features` (fast host run).
- Commit message format (matches repo history):
  ```
  <imperative summary>

  Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
  ```
- Do **not** touch `Request`/`Response` enum variant shapes anywhere in this plan.

---

## Task 1: De-glob the crate-root re-exports

Replace the two wildcard re-exports in `src/lib.rs` with explicit named lists so the public surface is intentional and individually documented. The compiler is the safety net: every internal module imports common/service types via `crate::…`, so a missing name fails the build.

**Files:**
- Modify: `src/lib.rs:18` (`pub use common::*;`) and `src/lib.rs:30` (`pub use services::*;`)

- [ ] **Step 1: Replace `pub use common::*;`**

In `src/lib.rs`, replace the single line `pub use common::*;` with:

```rust
pub use common::{
    CLEAR_ALL_DTCS, CommunicationControlType, CommunicationType, DTCExtDataRecordNumber,
    DTCFormatIdentifier, DTCRecord, DTCSeverityMask, DTCSeverityRecord, DTCSnapshotRecordNumber,
    DTCStatusMask, DTCStoredDataRecordNumber, DiagnosticSessionType, FunctionalGroupIdentifier,
    NegativeResponseCode, ResetType, SecurityAccessType, UDSIdentifier, UDSRoutineIdentifier,
    param_length_u128, param_length_u16, param_length_u32, param_length_u64,
};
```

- [ ] **Step 2: Replace `pub use services::*;`**

In `src/lib.rs`, replace the single line `pub use services::*;` with:

```rust
pub use services::{
    ClearDiagnosticInfoRequest, CommunicationControlRequest, CommunicationControlResponse,
    ControlDTCSettingsRequest, ControlDTCSettingsResponse, DiagnosticSessionControlRequest,
    DiagnosticSessionControlResponse, DirSizePayload, DtcAndStatusIter, DtcFaultDetectionIter,
    DtcSeverityAndStatusIter, EcuResetRequest, EcuResetResponse, FileOperationMode,
    FileSizePayload, NamePayloadTx, NegativeResponse, PositionPayload,
    ReadDTCInfoRequest, ReadDTCInfoResponseRx, ReadDTCInfoSubFunction,
    ReadDataByIdentifierRequestTx, RequestDownloadRequest, RequestDownloadResponseTx,
    RequestFileTransferRequestTx, RequestFileTransferResponseTx, RoutineControlRequestTx,
    RoutineControlResponseTx, SecurityAccessRequestTx, SecurityAccessResponseTx, SentDataPayloadTx,
    SizePayload, TesterPresentRequest, TesterPresentResponse, TransferDataRequestTx,
    TransferDataResponseTx, WriteDataByIdentifierRequestTx, WriteDataByIdentifierResponse,
};
```

- [ ] **Step 3: Build to verify no public name was dropped**

Run: `cargo build --all-features`
Expected: PASS. A missing re-export would fail here (internal modules resolve these via `crate::…`). If a name is reported unresolved or unused, add/remove it from the lists above to match.

- [ ] **Step 4: Confirm nothing newly hidden / formatting**

Run: `cargo test --all-features && cargo fmt -- --check`
Expected: PASS (tests reference these names through the crate root). If `cargo fmt` reports diffs, run `cargo fmt` and re-check.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "$(printf 'make crate-root re-exports explicit (drop glob re-exports)\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

---

## Task 2: `Encode` for the five DTC parameter types

`ReadDTCInfoSubFunction` (Task 3) encodes its parameters by delegating to each parameter type's own `Encode`. `DTCStatusMask` and `DTCRecord` already implement `Encode`; these five do not yet. Each is a 1-byte value. This task also fixes a latent `todo!()` panic in `FunctionalGroupIdentifier::value()`.

**Files:**
- Modify: `src/common/dtc_snapshot.rs` (add `use` + `Encode` for `DTCSnapshotRecordNumber`)
- Modify: `src/common/dtc_ext_data.rs` (add `use` + `Encode` for `DTCExtDataRecordNumber`)
- Modify: `src/common/dtc_status.rs` (`Encode` for `DTCStoredDataRecordNumber`, `DTCSeverityMask`, `FunctionalGroupIdentifier`; fix `FunctionalGroupIdentifier::value()`)

- [ ] **Step 1: Write failing tests for all five `Encode` impls**

In `src/common/dtc_snapshot.rs`, inside `mod snapshot`, add:

```rust
    #[test]
    fn encode_snapshot_record_number() {
        use crate::test_util::assert_encode_size_agrees;
        let n = DTCSnapshotRecordNumber::new(0x02);
        let mut buf = [0u8; 4];
        let written = crate::Encode::encode(&n, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x02);
        assert_encode_size_agrees(&n);
    }
```

In `src/common/dtc_ext_data.rs`, inside `mod tests`, add:

```rust
    #[test]
    fn encode_ext_data_record_number() {
        use crate::test_util::assert_encode_size_agrees;
        let n = DTCExtDataRecordNumber::new(0x90);
        let mut buf = [0u8; 4];
        let written = crate::Encode::encode(&n, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x90);
        assert_encode_size_agrees(&n);
    }
```

In `src/common/dtc_status.rs`, add a test module at the end of the file (if one does not exist) or append these tests to the existing `#[cfg(test)] mod` block:

```rust
#[cfg(test)]
mod encode_param_tests {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn encode_stored_data_record_number() {
        let n = DTCStoredDataRecordNumber::new(0x05).unwrap();
        let mut buf = [0u8; 4];
        let written = Encode::encode(&n, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x05);
        assert_encode_size_agrees(&n);
    }

    #[test]
    fn encode_severity_mask() {
        let m = DTCSeverityMask::CheckImmediately;
        let mut buf = [0u8; 4];
        let written = Encode::encode(&m, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0b1000_0000);
        assert_encode_size_agrees(&m);
    }

    #[test]
    fn encode_functional_group_identifier_named() {
        let g = FunctionalGroupIdentifier::EmissionsSystemGroup;
        let mut buf = [0u8; 4];
        let written = Encode::encode(&g, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 1);
        assert_eq!(buf[0], 0x33);
        assert_encode_size_agrees(&g);
    }

    #[test]
    fn functional_group_identifier_value_does_not_panic_on_reserved() {
        // Regression: value() previously called todo!() for carried-byte variants.
        let g = FunctionalGroupIdentifier::from(0x10); // -> ISOSAEReserved(0x10)
        assert_eq!(g.value(), 0x10);
        let g2 = FunctionalGroupIdentifier::from(0xD5); // -> LegislativeSystemGroup(0xD5)
        assert_eq!(g2.value(), 0xD5);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test --all-features encode_snapshot_record_number encode_ext_data_record_number encode_stored_data_record_number encode_severity_mask encode_functional_group_identifier_named functional_group_identifier_value_does_not_panic_on_reserved`
Expected: FAIL — `Encode` not implemented for these types; the reserved-value test panics with `todo!`.

- [ ] **Step 3: Fix `FunctionalGroupIdentifier::value()`**

In `src/common/dtc_status.rs`, replace the body of `FunctionalGroupIdentifier::value()` (currently the `match` with two `todo!()` arms) with:

```rust
    /// Return the raw `u8` value of this functional group identifier.
    #[must_use]
    pub fn value(&self) -> u8 {
        match self {
            FunctionalGroupIdentifier::EmissionsSystemGroup => 0x33,
            FunctionalGroupIdentifier::SafetySystemGroup => 0xD0,
            FunctionalGroupIdentifier::VODBSystem => 0xFE,
            FunctionalGroupIdentifier::LegislativeSystemGroup(value)
            | FunctionalGroupIdentifier::ISOSAEReserved(value) => *value,
        }
    }
```

- [ ] **Step 4: Add the `Encode` impls**

In `src/common/dtc_snapshot.rs`, add at the top of the file (the file currently has no imports):

```rust
use crate::{Encode, Error};
```

and after the `impl PartialEq<u8> for DTCSnapshotRecordNumber` block add:

```rust
impl Encode for DTCSnapshotRecordNumber {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[self.value()]).map_err(Error::io)?;
        Ok(1)
    }
}
```

In `src/common/dtc_ext_data.rs`, add at the top of the file:

```rust
use crate::{Encode, Error};
```

and after the `impl PartialEq<u8> for DTCExtDataRecordNumber` block add:

```rust
impl Encode for DTCExtDataRecordNumber {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[self.value()]).map_err(Error::io)?;
        Ok(1)
    }
}
```

In `src/common/dtc_status.rs` (which already imports `Encode`/`Error` for the existing `DTCStatusMask`/`DTCRecord` impls), add these three impls (place each near its type definition):

```rust
impl Encode for DTCStoredDataRecordNumber {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[self.0]).map_err(Error::io)?;
        Ok(1)
    }
}

impl Encode for DTCSeverityMask {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[self.bits()]).map_err(Error::io)?;
        Ok(1)
    }
}

impl Encode for FunctionalGroupIdentifier {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[self.value()]).map_err(Error::io)?;
        Ok(1)
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --all-features encode_snapshot_record_number encode_ext_data_record_number encode_stored_data_record_number encode_severity_mask encode_functional_group_identifier_named functional_group_identifier_value_does_not_panic_on_reserved`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/common/dtc_snapshot.rs src/common/dtc_ext_data.rs src/common/dtc_status.rs
git commit -m "$(printf 'add Encode to DTC parameter types; fix FunctionalGroupIdentifier::value panic\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

---

## Task 3: `Encode` for `ReadDTCInfoSubFunction` and `ReadDTCInfoRequest`

Give the 25-variant `ReadDTCInfoSubFunction` a faithful `Encode` (sub-function byte + each variant's typed parameters, delegating to the parameter `Encode` impls from Task 2), and have `ReadDTCInfoRequest` delegate to it. The two match arms (in `encode` and `encoded_size`) use identical variant grouping and reference `encoded_size()` rather than hard-coded widths; `assert_encode_size_agrees` guards drift.

**Files:**
- Modify: `src/services/read_dtc_information.rs` (already imports `Decode, Encode, Error`)

- [ ] **Step 1: Write failing tests**

In `src/services/read_dtc_information.rs`, add (or extend) a `#[cfg(test)]` module with:

```rust
#[cfg(test)]
mod read_dtc_info_request_encode_tests {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn encode_no_param_subfunction() {
        // 0x0A ReportSupportedDTC, no parameters.
        let req = ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportSupportedDTC);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x0A]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_single_param_subfunction() {
        // 0x02 ReportDTC_ByStatusMask(mask). DTCStatusMask is 1 byte.
        let mask = DTCStatusMask::from(0xFF);
        let req =
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportDTC_ByStatusMask(mask));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x02, 0xFF]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_multi_param_subfunction() {
        // 0x42 ReportWWHOBDDTC_ByMaskRecord(group, status, severity).
        let req = ReadDTCInfoRequest::new(
            ReadDTCInfoSubFunction::ReportWWHOBDDTC_ByMaskRecord(
                FunctionalGroupIdentifier::EmissionsSystemGroup,
                DTCStatusMask::from(0x08),
                DTCSeverityMask::CheckImmediately,
            ),
        );
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x42, 0x33, 0x08, 0b1000_0000]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_reserved_subfunction() {
        // ISOSAEReserved carries the sub-function byte itself, no params.
        let req = ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ISOSAEReserved(0x57));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x57]);
        assert_encode_size_agrees(&req);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --all-features read_dtc_info_request_encode_tests`
Expected: FAIL — `Encode` not implemented for `ReadDTCInfoRequest`.

- [ ] **Step 3: Implement `Encode` for `ReadDTCInfoSubFunction`**

In `src/services/read_dtc_information.rs`, after the `impl ReadDTCInfoSubFunction { … value() … }` block, add:

```rust
impl Encode for ReadDTCInfoSubFunction {
    fn encoded_size(&self) -> usize {
        use ReadDTCInfoSubFunction as S;
        1 + match self {
            S::ReportNumberOfDTC_ByStatusMask(m)
            | S::ReportDTC_ByStatusMask(m)
            | S::ReportUserDefMemoryDTC_ByStatusMask(m) => m.encoded_size(),
            S::ReportDTCSnapshotRecord_ByDTCNumber(r, n) => r.encoded_size() + n.encoded_size(),
            S::ReportDTCStoredData_ByRecordNumber(n) => n.encoded_size(),
            S::ReportDTCExtDataRecord_ByDTCNumber(r, n) => r.encoded_size() + n.encoded_size(),
            S::ReportNumberOfDTC_BySeverityMaskRecord(s, m)
            | S::ReportDTC_BySeverityMaskRecord(s, m) => s.encoded_size() + m.encoded_size(),
            S::ReportSeverityInfoOfDTC(r) => r.encoded_size(),
            S::ReportDTCExtDataRecord_ByRecordNumber(n)
            | S::ReportSupportedDTCExtDataRecord(n) => n.encoded_size(),
            S::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(r, n, mem) => {
                r.encoded_size() + n.encoded_size() + mem.encoded_size()
            }
            S::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(r, n, mem) => {
                r.encoded_size() + n.encoded_size() + mem.encoded_size()
            }
            S::ReportWWHOBDDTC_ByMaskRecord(g, m, s) => {
                g.encoded_size() + m.encoded_size() + s.encoded_size()
            }
            S::ReportWWHOBDDTC_WithPermanentStatus(g) => g.encoded_size(),
            S::ReportDTCInformation_ByDTCReadinessGroupIdentifier(g, rg) => {
                g.encoded_size() + rg.encoded_size()
            }
            S::ReportDTCSnapshotIdentification
            | S::ReportSupportedDTC
            | S::ReportFirstTestFailedDTC
            | S::ReportFirstConfirmedDTC
            | S::ReportMostRecentTestFailedDTC
            | S::ReportMostRecentConfirmedDTC
            | S::ReportDTCFaultDetectionCounter
            | S::ReportDTCWithPermanentStatus
            | S::ISOSAEReserved(_) => 0,
        }
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        use ReadDTCInfoSubFunction as S;
        writer.write_all(&[self.value()]).map_err(Error::io)?;
        match self {
            S::ReportNumberOfDTC_ByStatusMask(m)
            | S::ReportDTC_ByStatusMask(m)
            | S::ReportUserDefMemoryDTC_ByStatusMask(m) => {
                m.encode(writer)?;
            }
            S::ReportDTCSnapshotRecord_ByDTCNumber(r, n) => {
                r.encode(writer)?;
                n.encode(writer)?;
            }
            S::ReportDTCStoredData_ByRecordNumber(n) => {
                n.encode(writer)?;
            }
            S::ReportDTCExtDataRecord_ByDTCNumber(r, n) => {
                r.encode(writer)?;
                n.encode(writer)?;
            }
            S::ReportNumberOfDTC_BySeverityMaskRecord(s, m)
            | S::ReportDTC_BySeverityMaskRecord(s, m) => {
                s.encode(writer)?;
                m.encode(writer)?;
            }
            S::ReportSeverityInfoOfDTC(r) => {
                r.encode(writer)?;
            }
            S::ReportDTCExtDataRecord_ByRecordNumber(n)
            | S::ReportSupportedDTCExtDataRecord(n) => {
                n.encode(writer)?;
            }
            S::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(r, n, mem) => {
                r.encode(writer)?;
                n.encode(writer)?;
                mem.encode(writer)?;
            }
            S::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(r, n, mem) => {
                r.encode(writer)?;
                n.encode(writer)?;
                mem.encode(writer)?;
            }
            S::ReportWWHOBDDTC_ByMaskRecord(g, m, s) => {
                g.encode(writer)?;
                m.encode(writer)?;
                s.encode(writer)?;
            }
            S::ReportWWHOBDDTC_WithPermanentStatus(g) => {
                g.encode(writer)?;
            }
            S::ReportDTCInformation_ByDTCReadinessGroupIdentifier(g, rg) => {
                g.encode(writer)?;
                rg.encode(writer)?;
            }
            S::ReportDTCSnapshotIdentification
            | S::ReportSupportedDTC
            | S::ReportFirstTestFailedDTC
            | S::ReportFirstConfirmedDTC
            | S::ReportMostRecentTestFailedDTC
            | S::ReportMostRecentConfirmedDTC
            | S::ReportDTCFaultDetectionCounter
            | S::ReportDTCWithPermanentStatus
            | S::ISOSAEReserved(_) => {}
        }
        Ok(self.encoded_size())
    }
}
```

(Note: `mem` is `MemorySelection` and `rg` is `DTCReadinessGroupIdentifier`; both are `type … = u8;` aliases, and `u8` already implements `Encode` with a 1-byte width.)

- [ ] **Step 4: Implement `Encode` for `ReadDTCInfoRequest` (delegates)**

In `src/services/read_dtc_information.rs`, after the `impl ReadDTCInfoRequest { … new() … }` block, add:

```rust
impl Encode for ReadDTCInfoRequest {
    fn encoded_size(&self) -> usize {
        self.dtc_subfunction.encoded_size()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        self.dtc_subfunction.encode(writer)
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --all-features read_dtc_info_request_encode_tests`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/services/read_dtc_information.rs
git commit -m "$(printf 'implement Encode for ReadDTCInfoSubFunction and ReadDTCInfoRequest\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

---

## Task 4: `Decode` for `WriteDataByIdentifierResponse`

The fixed 2-byte echo response currently has `Encode` but no `Decode`, so it cannot round-trip. Add a 2-byte big-endian `Decode`.

**Files:**
- Modify: `src/services/write_data_by_identifier.rs:2` (imports) and add a `Decode` impl

- [ ] **Step 1: Write a failing round-trip test**

In `src/services/write_data_by_identifier.rs`, inside `mod test`, add:

```rust
    #[test]
    fn write_response_roundtrip() {
        let response = WriteDataByIdentifierResponse::new(0xF186);
        let mut buf = [0u8; 4];
        let written = Encode::encode(&response, &mut buf.as_mut_slice()).unwrap();
        let (decoded, rest) =
            <WriteDataByIdentifierResponse as crate::Decode>::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, response);
        assert!(rest.is_empty());
    }

    #[test]
    fn write_response_decode_rejects_short_buffer() {
        let err = <WriteDataByIdentifierResponse as crate::Decode>::decode(&[0x01]);
        assert!(err.is_err());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --all-features write_response_roundtrip write_response_decode_rejects_short_buffer`
Expected: FAIL — `Decode` not implemented for `WriteDataByIdentifierResponse`.

- [ ] **Step 3: Add `Decode` to imports and implement it**

In `src/services/write_data_by_identifier.rs`, change the import line:

```rust
use crate::{Encode, Error, NegativeResponseCode};
```

to:

```rust
use crate::{Decode, Encode, Error, NegativeResponseCode};
```

Then add, after the `impl Encode for WriteDataByIdentifierResponse` block:

```rust
impl<'a> Decode<'a> for WriteDataByIdentifierResponse {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(2));
        }
        let identifier = u16::from_be_bytes([buf[0], buf[1]]);
        Ok((Self { identifier }, &buf[2..]))
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --all-features write_response_roundtrip write_response_decode_rejects_short_buffer`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/services/write_data_by_identifier.rs
git commit -m "$(printf 'add Decode for WriteDataByIdentifierResponse (2-byte round-trip)\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

---

## Task 5: Crate-root integration tests for the completed types

Prove the two newly-completed types are reachable through the explicit crate-root re-exports (validating Task 1) and behave correctly end-to-end (validating Tasks 3–4). Tests import only via `crate::…` (the public surface), not via internal module paths.

**Files:**
- Modify: `src/lib.rs` (`#[cfg(test)] mod no_std_api_tests`)

- [ ] **Step 1: Write the integration tests**

In `src/lib.rs`, inside `mod no_std_api_tests`, add:

```rust
    #[test]
    fn read_dtc_info_request_encodes_through_public_api() {
        // Public-surface construction: types reached via crate root, not common::/services::.
        let req = ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportDTC_ByStatusMask(
            DTCStatusMask::from(0xFF),
        ));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x02, 0xFF]);
        assert_eq!(written, req.encoded_size());
    }

    #[test]
    fn write_data_by_identifier_response_roundtrips_through_public_api() {
        let resp = WriteDataByIdentifierResponse::new(0xBEEF);
        let mut buf = [0u8; 4];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        let (decoded, rest) =
            <WriteDataByIdentifierResponse as Decode>::decode(&buf[..written]).unwrap();
        assert_eq!(decoded, resp);
        assert!(rest.is_empty());
    }
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test --all-features read_dtc_info_request_encodes_through_public_api write_data_by_identifier_response_roundtrips_through_public_api`
Expected: PASS. (If a name does not resolve, Task 1's re-export list is missing it — add it.)

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "$(printf 'add crate-root integration tests for completed descriptor types\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

---

## Task 6: Full-matrix verification

Confirm the whole CI matrix is green before considering Phase 1 done. No code changes; if any command fails, fix the offending task and re-run.

- [ ] **Step 1: Ensure the bare-metal target is installed**

Run: `rustup target add thumbv6m-none-eabi`
Expected: installed (or "up to date").

- [ ] **Step 2: Build + test (host, all features)**

Run:
```bash
cargo build --all-features --release
cargo test --all-features
```
Expected: PASS.

- [ ] **Step 3: no_std / no_alloc builds**

Run:
```bash
cargo build --no-default-features --target thumbv6m-none-eabi
cargo build --no-default-features --features alloc --target thumbv6m-none-eabi
```
Expected: PASS.

- [ ] **Step 4: Clippy on all host feature combos**

Run:
```bash
cargo clippy --all-features
cargo clippy --no-default-features
cargo clippy --no-default-features --features alloc
```
Expected: no warnings. (The crate sets `#![warn(clippy::pedantic, missing_docs)]`; resolve any new lints — all added items are documented and `#[must_use]` where applicable.)

- [ ] **Step 5: Formatting + docs**

Run:
```bash
cargo fmt -- --check
cargo doc --release --all-features --no-deps
```
Expected: PASS (no diffs, no doc warnings).

- [ ] **Step 6: (No commit)** — verification only. Phase 1 complete.

---

## Self-review notes

- **Spec coverage:** Phase-1 commit 1 → Task 1; commit 2 → Task 2; commit 3 → Task 3; commit 4 → Task 4; commit 5 (repurposed) → Task 5; matrix testing → Task 6. Phase 2 items are intentionally out of scope.
- **No enum restructuring:** confirmed — no task edits `Request`/`Response` variant shapes.
- **Type consistency:** `Encode`/`Decode` signatures match `src/traits.rs`; `assert_encode_size_agrees` matches `src/test_util.rs`; parameter accessor methods (`value()`, `bits()`, field `.0`) match the definitions in `src/common/`.
- **Latent bug:** `FunctionalGroupIdentifier::value()` `todo!()` panic is fixed in Task 2 (required for its `Encode`).
