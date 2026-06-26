# Pre-Merge API Interrogation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the public-API and implementation-detail resolutions (Decisions A–H, I1–I4) from `docs/superpowers/specs/2026-06-08-api-interrogation-resolutions-design.md` as one cohesive breaking change on `feature/no_std`.

**Architecture:** Move-first. Three mechanical, behavior-neutral module-reorg commits (`common/`→`shared/`, extract `dtc/`, relocate single-service enums) land before any semantic change, so the heavily-reviewed reshaping happens once, in the final structure. Then: the test-helper fix and the standalone `UDSIdentifier` rebuild, then the dispatch-enum/descriptor reshaping, then constant removal. No commit mixes a mechanical move with a semantic change.

**Tech Stack:** Rust 2024, `no_std` + `no_alloc` baseline (`std`/`alloc` additive), `embedded-io` traits, `byteorder-embedded-io`. Codec via crate-local `Encode`/`Decode`/`DecodeIter` traits. Test matrix: default (`std`), `--no-default-features --features alloc`, `--no-default-features`, `thumbv6m-none-eabi`.

**Standing verification (run after every task that changes code) — matches CI exactly:**

```bash
cargo test
cargo clippy --all-features
cargo clippy --no-default-features
cargo clippy --no-default-features --features alloc
cargo fmt --check
```

CI does **not** use `-D warnings` or `--all-targets`. Do not add them: the base branch
already carries ~11 `clippy::pedantic` warnings in *test* code (`similar_names`,
`unreadable_literal`, `cast_possible_truncation`, etc.) that CI does not gate and that are
**out of scope** for this work. The library compiles clean even under `-D warnings`; only
test code trips pedantic lints. "Clippy clean" here means the three CI invocations above
exit 0. Do not introduce *new* clippy warnings.

The full breaking matrix (`--no-default-features [--features alloc]` + `cargo build --target thumbv6m-none-eabi ...`) is validated at the end (Task 15), and in CI.

______________________________________________________________________

## File Structure

**Phase 0 (mechanical moves):**

- `src/common/` → `src/shared/` (rename module; `mod.rs` + every `crate::common` path).
- `src/dtc/` (new): `mod.rs`, `status.rs`, `snapshot.rs`, `ext_data.rs` — the DTC vocabulary (`DTCRecord`, masks, severity, format, record-numbers, `FunctionalGroupIdentifier`, `CLEAR_ALL_DTCS`) moved out of `shared/`. Read-DTC iterators stay in `src/services/read_dtc_information.rs`.
- Single-service enums move from `shared/` (or `lib.rs`) into their service module: `ResetType`→`ecu_reset.rs`, `DiagnosticSessionType`→`diagnostic_session_control.rs`, `SecurityAccessType`→`security_access.rs`, `CommunicationControlType`+`CommunicationType`→`communication_control.rs`, `DtcSettings`→`control_dtc_settings.rs`, `RoutineControlSubFunction`→`routine_control.rs`.

**Phase 1+ (semantic):**

- `src/test_util.rs` — strengthen `assert_encode_size_agrees`.
- `src/shared/diagnostic_identifier.rs` — `UDSIdentifier` faithful total `From<u16>`; drop `TryFrom`; remove `Encode`/`Decode` on both identifier enums.
- `src/error.rs` — remove `InvalidDiagnosticIdentifier`.
- `src/request.rs` / `src/response.rs` — `Other { sid, data }`; `Response::service()`; rewire reshaped variants.
- `src/services/{read_data_by_identifier,write_data_by_identifier,routine_control,control_dtc_settings}.rs` — descriptor reshaping.
- `src/services/request_transfer_exit.rs` (new) — the `RequestTransferExit{Request,Response}` descriptors.
- `src/lib.rs` — re-export updates; delete `SUCCESS`/`PENDING`.

______________________________________________________________________

# PHASE 0 — Mechanical module reorg (move-first)

### Task 1: Rename `common/` → `shared/`

**Files:**

- Rename: `src/common/` → `src/shared/` (all files within)

- Modify: `src/lib.rs` (the `mod common;` declaration and its `pub use common::{…}`)

- Modify: every file importing `crate::common` — `src/shared/util.rs`, `src/services/{communication_control,diagnostic_session_control,ecu_reset,request_download,request_file_transfer,routine_control,security_access,tester_present}.rs`

- [ ] **Step 1: Move the directory with git**

Run:

```bash
git mv src/common src/shared
```

- [ ] **Step 2: Repoint every `common` path to `shared`**

Replace `crate::common` → `crate::shared`, `use crate::common` → `use crate::shared`, and `super::common` → `super::shared` across `src/`. In `src/lib.rs` change `mod common;` → `mod shared;` and `pub use common::{` → `pub use shared::{`. Verify none remain:

```bash
rg -n "\bcommon\b" src   # expect: no module-path hits (CommunicationControl etc. are unrelated)
```

- [ ] **Step 3: Build + test**

Run: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: PASS, unchanged test count. This is a pure rename — git should show the files as renames.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "rename common/ module to shared/ (no behavior change)"
```

### Task 2: Extract the `dtc/` domain module

**Files:**

- Create: `src/dtc/mod.rs`

- Move: `src/shared/dtc_status.rs`→`src/dtc/status.rs`, `src/shared/dtc_snapshot.rs`→`src/dtc/snapshot.rs`, `src/shared/dtc_ext_data.rs`→`src/dtc/ext_data.rs`

- Modify: `src/lib.rs` (add `mod dtc;`, move the DTC re-exports off the `shared` list onto a `dtc` list), `src/shared/mod.rs` (drop the three `mod dtc_*` + their `pub use`)

- Modify: importers of DTC types — `src/services/clear_dtc_information.rs`, `src/services/read_dtc_information.rs`

- [ ] **Step 1: Move the three DTC files**

Run:

```bash
git mv src/shared/dtc_status.rs src/dtc/status.rs
git mv src/shared/dtc_snapshot.rs src/dtc/snapshot.rs
git mv src/shared/dtc_ext_data.rs src/dtc/ext_data.rs
```

(`git mv` to a non-existent dir: create `src/dtc/` first if needed — `mkdir src/dtc` then move.)

- [ ] **Step 2: Create `src/dtc/mod.rs`**

```rust
//! DTC (Diagnostic Trouble Code) vocabulary shared across the DTC services
//! (ReadDTCInformation, ClearDiagnosticInformation).
mod status;
pub use status::*;

mod snapshot;
pub use snapshot::*;

mod ext_data;
pub use ext_data::*;
```

- [ ] **Step 3: Update `src/shared/mod.rs`**

Remove the `mod dtc_ext_data; pub use dtc_ext_data::*;`, `mod dtc_status; pub use dtc_status::*;`, and `mod dtc_snapshot; pub use dtc_snapshot::*;` blocks (now owned by `dtc/`).

- [ ] **Step 4: Add `mod dtc;` and move re-exports in `src/lib.rs`**

Add `mod dtc;` next to `mod shared;`. Move the DTC names (`CLEAR_ALL_DTCS`, `DTCExtDataRecordNumber`, `DTCFormatIdentifier`, `DTCRecord`, `DTCSeverityMask`, `DTCSeverityRecord`, `DTCSnapshotRecordNumber`, `DTCStatusMask`, `DTCStoredDataRecordNumber`, `FunctionalGroupIdentifier`) out of the `pub use shared::{…}` list into a new `pub use dtc::{…}` list. Fix intra-crate importers to `crate::dtc::…` (or rely on the crate-root re-export — `crate::DTCRecord` still works).

- [ ] **Step 5: Build + test**

Run: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: PASS, unchanged test count. Verify acyclic: `rg -n "services::" src/dtc` must return nothing (dtc/ depends only on shared/ + crate-root traits).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "extract dtc/ domain module from shared/"
```

### Task 3: Relocate single-service enums into their service modules

**Files (one enum per move; each carries its impls, tests, and derives):**

- `ResetType` (`src/shared/reset_type.rs`) → into `src/services/ecu_reset.rs`; delete the file

- `DiagnosticSessionType` (`src/shared/diagnostic_session_type.rs`) → `src/services/diagnostic_session_control.rs`

- `SecurityAccessType` (`src/shared/security_access_type.rs`) → `src/services/security_access.rs`

- `CommunicationControlType` (`src/shared/communication_control_type.rs`) + `CommunicationType` (`src/shared/communication_type.rs`) → `src/services/communication_control.rs`

- `DtcSettings` (defined in `src/lib.rs`) → `src/services/control_dtc_settings.rs`

- `RoutineControlSubFunction` (defined in `src/lib.rs`) → `src/services/routine_control.rs`

- Modify: `src/shared/mod.rs` (drop the moved `mod`/`pub use`), `src/lib.rs` (the moved enums now re-export from `services::`, and `DtcSettings`/`RoutineControlSubFunction` definitions leave `lib.rs`)

- [ ] **Step 1: Move each enum's source into its service module**

For each enum: cut its full definition + `impl` blocks + `#[cfg(test)]` module from the source and paste into the target service module (top of file, after the existing `use`s). For `DtcSettings` and `RoutineControlSubFunction`, cut from `src/lib.rs` (lines defining `pub enum DtcSettings` / `pub enum RoutineControlSubFunction` and their `From`/`TryFrom` impls). Delete the now-empty `src/shared/*_type.rs` files via `git rm`.

- [ ] **Step 2: Fix `mod`/`pub use` wiring**

In `src/shared/mod.rs`, remove the `mod reset_type; pub use reset_type::ResetType;` (and the four siblings). In `src/lib.rs`, the moved enums must still re-export flat from the crate root: add them to the `pub use services::{…}` list (`ResetType`, `DiagnosticSessionType`, `SecurityAccessType`, `CommunicationControlType`, `CommunicationType`, `DtcSettings`, `RoutineControlSubFunction`). Inside service modules, drop now-redundant `use crate::ResetType;`-style self-imports.

- [ ] **Step 3: Build + test**

Run: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`
Expected: PASS, unchanged test count.

- [ ] **Step 4: Verify no public-path drift**

Compare the set of re-exported names against the pre-Phase-0 `lib.rs`. Capture the baseline once at the start of Phase 0 (`git show <pre-phase-0-rev>:src/lib.rs`), then:

```bash
rg -o "pub use \w+::\{[^}]*\}" src/lib.rs
```

Expected: the union of re-exported identifiers is identical to before Phase 0 (only their source module changed). No name added or dropped.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "relocate single-service enums into their service modules"
```

______________________________________________________________________

# PHASE 1 — Test helper + standalone identifier rebuild

### Task 4: Strengthen `assert_encode_size_agrees`

**Files:**

- Modify: `src/test_util.rs`

- [ ] **Step 1: Replace the helper body to also check bytes consumed**

```rust
//! Test-only helpers shared across the crate.

use crate::Encode;

/// Assert that an [`Encode`] value writes exactly `encoded_size()` bytes — both the
/// returned count AND the number of bytes actually consumed from the writer.
///
/// Guards against `encode` and `encoded_size` drifting, and against `encode` returning
/// a count that disagrees with how many bytes it actually wrote — either corrupts callers
/// that pre-size a buffer from `encoded_size()`.
pub(crate) fn assert_encode_size_agrees<T: Encode>(value: &T) {
    let mut buf = [0u8; 512];
    let cap = buf.len();
    let mut writer: &mut [u8] = &mut buf;
    let written = value.encode(&mut writer).unwrap();
    let consumed = cap - writer.len();
    let size = value.encoded_size();
    assert_eq!(written, size, "encode returned {written}, encoded_size() is {size}");
    assert_eq!(consumed, size, "encode consumed {consumed} bytes, encoded_size() is {size}");
}
```

- [ ] **Step 2: Build + test**

Run: `cargo test`
Expected: PASS — every existing `assert_encode_size_agrees` call site still passes (consumed == returned == size for all current codecs).

- [ ] **Step 3: Commit**

```bash
git add src/test_util.rs
git commit -m "assert_encode_size_agrees: also check actual bytes consumed"
```

### Task 5: `UDSIdentifier` faithful rebuild + drop enum codecs (Decision B-followup, I1)

**Files:**

- Modify: `src/shared/diagnostic_identifier.rs`
- Modify: `src/error.rs` (remove `InvalidDiagnosticIdentifier`)
- Test: in `src/shared/diagnostic_identifier.rs`

Reference: the authoritative partition is the Appendix of the design spec. Add the missing range variants and relabel `0xF100–0xF17F`.

- [ ] **Step 1: Write the failing round-trip + classification test**

Add to the test module in `src/shared/diagnostic_identifier.rs`:

```rust
#[test]
fn uds_identifier_from_is_total_and_round_trips() {
    // Every u16 maps to a variant and round-trips back to itself.
    for raw in 0u16..=u16::MAX {
        let id = UDSIdentifier::from(raw);
        assert_eq!(u16::from(id), raw, "round-trip failed for {raw:#06X}");
    }
}

#[test]
fn uds_identifier_classifies_representative_ranges() {
    use UDSIdentifier::*;
    assert!(matches!(UDSIdentifier::from(0x0042), ISOSAEReserved(0x0042)));
    assert!(matches!(UDSIdentifier::from(0x2000), VehicleManufacturerSpecific(0x2000)));
    assert!(matches!(UDSIdentifier::from(0xA600), ReservedForLegislativeUse(0xA600)));
    assert!(matches!(UDSIdentifier::from(0xF100), IdentificationOptionVehicleManufacturerSpecific(0xF100)));
    assert!(matches!(UDSIdentifier::from(0xF190), VIN));
    assert!(matches!(UDSIdentifier::from(0xF200), PeriodicDataIdentifier(0xF200)));
    assert!(matches!(UDSIdentifier::from(0xF400), OBDDataIdentifier(0xF400)));
    assert!(matches!(UDSIdentifier::from(0xFA10), NumberOfEDRDevices));
    assert!(matches!(UDSIdentifier::from(0xFF00), UDSVersionData));
    assert!(matches!(UDSIdentifier::from(0xFF01), ReservedForISO15765_5));
    assert!(matches!(UDSIdentifier::from(0xFFFE), ISOSAEReserved(0xFFFE)));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test uds_identifier_from -v`
Expected: FAIL — `From<u16>` does not exist for `UDSIdentifier` yet (only `TryFrom`), and the new variants are undefined.

- [ ] **Step 3: Add the new variants to the enum**

In `pub enum UDSIdentifier`, keep the existing `0xF180–0xF19F` named singletons, `UDSVersionData = 0xFF00`, `ReservedForISO15765_5 = 0xFF01`, and `ISOSAEReserved(u16)` / `SystemSupplierSpecific(u16)`. **Rename** the existing `VehicleManufacturerSpecific(u16)` semantics so it covers the true broad ranges, and **add** these range/singleton variants (drop the now-wrong `#[clap(skip)]`-only doc ranges; keep `#[clap(skip)]` on the `(u16)` variants):

```rust
    ReservedForLegislativeUse(u16),
    NetworkConfigDataForTractorTrailer(u16),          // 0xF000–0xF00F
    IdentificationOptionVehicleManufacturerSpecific(u16), // 0xF100–0xF17F, 0xF1A0–0xF1EF
    IdentificationOptionSystemSupplierSpecific(u16),  // 0xF1F0–0xF1FF
    PeriodicDataIdentifier(u16),                      // 0xF200–0xF2FF
    DynamicallyDefinedDataIdentifier(u16),            // 0xF300–0xF3FF
    OBDDataIdentifier(u16),                           // 0xF400–0xF5FF, 0xF700–0xF7FF
    OBDMonitorDataIdentifier(u16),                    // 0xF600–0xF6FF
    OBDInfoTypeDataIdentifier(u16),                   // 0xF800–0xF8FF
    TachographDataIdentifier(u16),                    // 0xF900–0xF9FF
    AirbagDeploymentDataIdentifier(u16),              // 0xFA00–0xFA0F
    NumberOfEDRDevices,                               // 0xFA10
    EDRIdentification,                                // 0xFA11
    EDRDeviceAddressInformation,                      // 0xFA12
    EDREntries(u16),                                  // 0xFA13–0xFA18
    SafetySystemDataIdentifier(u16),                  // 0xFA19–0xFAFF
```

(Singletons `NumberOfEDRDevices`/`EDRIdentification`/`EDRDeviceAddressInformation` carry no value; their fixed byte is set in `From<UDSIdentifier> for u16`.)

- [ ] **Step 4: Replace `impl TryFrom<u16>` with a total `impl From<u16>`**

```rust
impl From<u16> for UDSIdentifier {
    #[allow(clippy::match_same_arms)]
    fn from(value: u16) -> Self {
        match value {
            0x0000..=0x00FF => Self::ISOSAEReserved(value),
            0x0100..=0xA5FF => Self::VehicleManufacturerSpecific(value),
            0xA600..=0xA7FF => Self::ReservedForLegislativeUse(value),
            0xA800..=0xACFF => Self::VehicleManufacturerSpecific(value),
            0xAD00..=0xAFFF => Self::ReservedForLegislativeUse(value),
            0xB000..=0xB1FF => Self::VehicleManufacturerSpecific(value),
            0xB200..=0xBFFF => Self::ReservedForLegislativeUse(value),
            0xC000..=0xC2FF => Self::VehicleManufacturerSpecific(value),
            0xC300..=0xCEFF => Self::ReservedForLegislativeUse(value),
            0xCF00..=0xEFFF => Self::VehicleManufacturerSpecific(value),
            0xF000..=0xF00F => Self::NetworkConfigDataForTractorTrailer(value),
            0xF010..=0xF0FF => Self::VehicleManufacturerSpecific(value),
            0xF100..=0xF17F => Self::IdentificationOptionVehicleManufacturerSpecific(value),
            0xF180 => Self::BootSoftwareIdentification,
            0xF181 => Self::ApplicationSoftwareIdentification,
            0xF182 => Self::ApplicationDataIdentification,
            0xF183 => Self::BootSoftwareFingerprint,
            0xF184 => Self::ApplicationSoftwareFingerprint,
            0xF185 => Self::ApplicationDataFingerprint,
            0xF186 => Self::ActiveDiagnosticSession,
            0xF187 => Self::VehicleManufacturerSparePartNumber,
            0xF188 => Self::VehicleManufacturerECUSoftwareNumber,
            0xF189 => Self::VehicleManufacturerECUSoftwareVersionNumber,
            0xF18A => Self::SystemSupplierIdentifier,
            0xF18B => Self::ECUManufacturingData,
            0xF18C => Self::ECUSerialNumber,
            0xF18D => Self::SupportedFunctionalUnits,
            0xF18E => Self::VehicleManufacturerKitAssemblyPartNumber,
            0xF18F => Self::RegulationXSoftwareIdentificationNumbers,
            0xF190 => Self::VIN,
            0xF191 => Self::VehicleManufacturerECUHardwareNumber,
            0xF192 => Self::SystemSupplierECUHardwareNumber,
            0xF193 => Self::SystemSupplierECUHardwareVersionNumber,
            0xF194 => Self::SystemSupplierECUSoftwareNumber,
            0xF195 => Self::SystemSupplierECUSoftwareVersionNumber,
            0xF196 => Self::ExhaustRegulationOrTypeApprovalNumber,
            0xF197 => Self::SystemNameOrEngineType,
            0xF198 => Self::RepairShopOrTesterSerialNumber,
            0xF199 => Self::ProgrammingDate,
            0xF19A => Self::CalibrationRepairShopCodeOrCalibrationEquipmentSerialNumber,
            0xF19B => Self::CalibrationDate,
            0xF19C => Self::CalibrationEquipmentSoftwareNumber,
            0xF19D => Self::ECUInstallationDate,
            0xF19E => Self::ODXFile,
            0xF19F => Self::Entity,
            0xF1A0..=0xF1EF => Self::IdentificationOptionVehicleManufacturerSpecific(value),
            0xF1F0..=0xF1FF => Self::IdentificationOptionSystemSupplierSpecific(value),
            0xF200..=0xF2FF => Self::PeriodicDataIdentifier(value),
            0xF300..=0xF3FF => Self::DynamicallyDefinedDataIdentifier(value),
            0xF400..=0xF5FF => Self::OBDDataIdentifier(value),
            0xF600..=0xF6FF => Self::OBDMonitorDataIdentifier(value),
            0xF700..=0xF7FF => Self::OBDDataIdentifier(value),
            0xF800..=0xF8FF => Self::OBDInfoTypeDataIdentifier(value),
            0xF900..=0xF9FF => Self::TachographDataIdentifier(value),
            0xFA00..=0xFA0F => Self::AirbagDeploymentDataIdentifier(value),
            0xFA10 => Self::NumberOfEDRDevices,
            0xFA11 => Self::EDRIdentification,
            0xFA12 => Self::EDRDeviceAddressInformation,
            0xFA13..=0xFA18 => Self::EDREntries(value),
            0xFA19..=0xFAFF => Self::SafetySystemDataIdentifier(value),
            0xFB00..=0xFCFF => Self::ReservedForLegislativeUse(value),
            0xFD00..=0xFEFF => Self::SystemSupplierSpecific(value),
            0xFF00 => Self::UDSVersionData,
            0xFF01 => Self::ReservedForISO15765_5,
            0xFF02..=0xFFFF => Self::ISOSAEReserved(value),
        }
    }
}
```

- [ ] **Step 5: Extend `impl From<UDSIdentifier> for u16` for the new variants**

Add arms returning the carried value for the `(u16)` variants, and the fixed bytes for the EDR singletons:

```rust
            UDSIdentifier::ReservedForLegislativeUse(v) => v,
            UDSIdentifier::NetworkConfigDataForTractorTrailer(v) => v,
            UDSIdentifier::IdentificationOptionVehicleManufacturerSpecific(v) => v,
            UDSIdentifier::IdentificationOptionSystemSupplierSpecific(v) => v,
            UDSIdentifier::PeriodicDataIdentifier(v) => v,
            UDSIdentifier::DynamicallyDefinedDataIdentifier(v) => v,
            UDSIdentifier::OBDDataIdentifier(v) => v,
            UDSIdentifier::OBDMonitorDataIdentifier(v) => v,
            UDSIdentifier::OBDInfoTypeDataIdentifier(v) => v,
            UDSIdentifier::TachographDataIdentifier(v) => v,
            UDSIdentifier::AirbagDeploymentDataIdentifier(v) => v,
            UDSIdentifier::NumberOfEDRDevices => 0xFA10,
            UDSIdentifier::EDRIdentification => 0xFA11,
            UDSIdentifier::EDRDeviceAddressInformation => 0xFA12,
            UDSIdentifier::EDREntries(v) => v,
            UDSIdentifier::SafetySystemDataIdentifier(v) => v,
```

(Keep existing arms; the old `VehicleManufacturerSpecific(v) => v` already returns `v`.)

- [ ] **Step 6: Remove `Encode`/`Decode` for both identifier enums**

Delete `impl Encode for UDSIdentifier`, `impl Decode<'a> for UDSIdentifier`, `impl Encode for UDSRoutineIdentifier`, `impl Decode<'a> for UDSRoutineIdentifier`, and the `codec_tests` module that exercised them. Remove the now-unused `use crate::{Decode, Encode, …}` (keep `Error`? — no longer needed after `TryFrom` removal; drop it).

- [ ] **Step 7: Remove the dead error variant**

In `src/error.rs`, delete the `InvalidDiagnosticIdentifier(u16)` variant (and its `thiserror` attribute line). Confirm no other reference:

```bash
rg -n "InvalidDiagnosticIdentifier" src   # expect: no matches
```

- [ ] **Step 8: Run tests**

Run: `cargo test uds_identifier && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: PASS — the totality loop and classification tests pass; nothing references the removed impls/variant.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "rebuild UDSIdentifier as faithful total From<u16>; drop identifier-enum codecs"
```

______________________________________________________________________

# PHASE 2 — Dispatch enum + descriptor reshaping

### Task 6: `Other { sid: u8, data }` + `Response::service()` (Decisions C, F)

**Files:**

- Modify: `src/request.rs`, `src/response.rs`

- [ ] **Step 1: Write failing lossless + service() tests**

In `src/request.rs` tests, replace the body of `unmodeled_service_decodes_to_other` and add a truly-unknown-byte case:

```rust
#[test]
fn unknown_request_byte_round_trips_losslessly() {
    // 0x40 is not in the ISO request table; it must survive a decode→encode round-trip.
    let frame = [0x40, 0xAA, 0xBB];
    let (req, rest) = Request::decode(&frame).unwrap();
    assert!(rest.is_empty());
    match req {
        Request::Other { sid, data } => {
            assert_eq!(sid, 0x40);
            assert_eq!(data, &[0xAA, 0xBB]);
        }
        other => panic!("expected Other, got {other:?}"),
    }
    let mut buf = [0u8; 8];
    let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
    assert_eq!(&buf[..written], &frame); // previously re-encoded as 0x7F
}
```

In `src/response.rs` tests:

```rust
#[test]
fn unknown_response_byte_round_trips_losslessly() {
    let frame = [0x99, 0x01, 0x02];
    let (resp, _) = Response::decode(&frame).unwrap();
    assert!(matches!(resp, Response::Other { sid: 0x99, .. }));
    assert_eq!(resp.service(), UdsServiceType::response_from_byte(0x99));
    let mut buf = [0u8; 8];
    let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
    assert_eq!(&buf[..written], &frame); // previously became 0x7F (NegativeResponse)
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test round_trips_losslessly -v`
Expected: FAIL — `Other` still holds `service`, and re-encode normalizes to `0x7F`; `Response::service()` doesn't exist.

- [ ] **Step 3: Reshape `Request::Other`**

In `src/request.rs`: change the variant to `Other { sid: u8, data: &'a [u8] }`. In `decode`, the `_ =>` arm becomes `_ => Self::Other { sid: buf[0], data: payload }`. In `service()`, `Self::Other { sid, .. } => UdsServiceType::service_from_request_byte(*sid)`. In `encode`, special-case the SID write so `Other` emits its raw byte: change the leading `writer.write_all(&[self.service().request_service_to_byte()])` to branch — `let sid = match self { Self::Other { sid, .. } => *sid, other => other.service().request_service_to_byte() };` then `writer.write_all(&[sid])`. The `Self::Other { data, .. }` arms in `encode`/`encoded_size` already use `data`.

- [ ] **Step 4: Reshape `Response::Other` + add `Response::service()`**

In `src/response.rs`: change the variant to `Other { sid: u8, data: &'a [u8] }`; `decode`'s `_ =>` arm → `Self::Other { sid: buf[0], data: payload }`. In `response_sid()`, `Self::Other { sid, .. } => *sid`. Add a public method:

```rust
impl Response<'_> {
    /// The [`UdsServiceType`] this response frame addresses.
    ///
    /// For `NegativeResponse` this returns [`UdsServiceType::NegativeResponse`] (the frame's
    /// own type); the *failed* request service is `NegativeResponse.request_service`.
    #[must_use]
    pub fn service(&self) -> UdsServiceType {
        match self {
            Self::Other { sid, .. } => UdsServiceType::response_from_byte(*sid),
            other => UdsServiceType::response_from_byte(other.response_sid()),
        }
    }
}
```

(`response_sid()` stays private; `service()` derives from it for non-`Other`, and from the raw `sid` for `Other`.)

- [ ] **Step 5: Update the doc comments**

On both `Other` variants, replace the "lossless for any service byte… a byte that maps to `UnsupportedDiagnosticService` re-encodes as `0x7F`" caveat with: "Re-encoding is lossless for every service byte: the raw `sid` is echoed verbatim."

- [ ] **Step 6: Run tests**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/request.rs src/response.rs
git commit -m "Other carries raw sid for lossless pass-through; add Response::service()"
```

### Task 7: `WriteDataByIdentifierRequest { identifier: u16, data }` (Decision B)

**Files:**

- Modify: `src/services/write_data_by_identifier.rs`, `src/request.rs` (match arms unchanged in shape — variant still wraps the descriptor)

- [ ] **Step 1: Write the failing round-trip + short-buffer tests**

Replace the request portion of the test module in `write_data_by_identifier.rs`:

```rust
#[test]
fn wdbi_request_round_trips() {
    let req = WriteDataByIdentifierRequest::new(0xF190, &[0x01, 0x02, 0x03]);
    let mut buf = [0u8; 8];
    let n = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
    assert_eq!(&buf[..n], &[0xF1, 0x90, 0x01, 0x02, 0x03]);
    let (decoded, rest) = <WriteDataByIdentifierRequest as Decode>::decode(&buf[..n]).unwrap();
    assert!(rest.is_empty());
    assert_eq!(decoded.identifier(), 0xF190);
    assert_eq!(decoded.data(), &[0x01, 0x02, 0x03]);
    assert_encode_size_agrees(&req);
}

#[test]
fn wdbi_request_allows_empty_data() {
    let (decoded, _) = <WriteDataByIdentifierRequest as Decode>::decode(&[0xF1, 0x90]).unwrap();
    assert_eq!(decoded.identifier(), 0xF190);
    assert!(decoded.data().is_empty());
}

#[test]
fn wdbi_request_rejects_short_buffer() {
    assert!(matches!(
        <WriteDataByIdentifierRequest as Decode>::decode(&[0xF1]),
        Err(Error::InsufficientData(2))
    ));
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test wdbi_request -v`
Expected: FAIL — `new` takes one arg, `identifier()`/`data()` don't exist.

- [ ] **Step 3: Reshape the struct + impls**

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct WriteDataByIdentifierRequest<'d> {
    identifier: u16,
    data: &'d [u8],
}

impl<'d> WriteDataByIdentifierRequest<'d> {
    /// Create a request to write `data` to the given Data Identifier.
    #[must_use]
    pub const fn new(identifier: u16, data: &'d [u8]) -> Self {
        Self { identifier, data }
    }

    /// The Data Identifier being written.
    #[must_use]
    pub const fn identifier(&self) -> u16 {
        self.identifier
    }

    /// The opaque data record to write.
    #[must_use]
    pub const fn data(&self) -> &[u8] {
        self.data
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &WRITE_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for WriteDataByIdentifierRequest<'_> {
    fn encoded_size(&self) -> usize {
        2 + self.data.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&self.identifier.to_be_bytes()).map_err(Error::io)?;
        writer.write_all(self.data).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for WriteDataByIdentifierRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::InsufficientData(2));
        }
        let identifier = u16::from_be_bytes([buf[0], buf[1]]);
        Ok((Self { identifier, data: &buf[2..] }, &[]))
    }
}
```

(Add `use crate::Error;` imports as needed; `assert_encode_size_agrees` import in the test module.)

- [ ] **Step 4: Run tests**

Run: `cargo test wdbi_request && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: PASS — `Request::WriteDataByIdentifier(WriteDataByIdentifierRequest)` arms still compile (shape unchanged).

- [ ] **Step 5: Commit**

```bash
git add src/services/write_data_by_identifier.rs
git commit -m "WriteDataByIdentifierRequest: typed u16 identifier + opaque data"
```

### Task 8: RoutineControl typed `routine_id` (Decision B)

**Files:**

- Modify: `src/services/routine_control.rs`

- [ ] **Step 1: Write failing tests (request round-trip incl. SPRMIB, response, short buffer)**

```rust
#[test]
fn rc_request_round_trips_with_suppress() {
    let req = RoutineControlRequest::new(true, RoutineControlSubFunction::StartRoutine, 0xFF00, &[0xAA]);
    let mut buf = [0u8; 8];
    let n = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
    assert_eq!(&buf[..n], &[0x81, 0xFF, 0x00, 0xAA]); // 0x81 = StartRoutine | SPRMIB
    let (d, rest) = <RoutineControlRequest as Decode>::decode(&buf[..n]).unwrap();
    assert!(rest.is_empty());
    assert!(d.suppress_positive_response());
    assert_eq!(d.sub_function(), RoutineControlSubFunction::StartRoutine);
    assert_eq!(d.routine_id(), 0xFF00);
    assert_eq!(d.option_record(), &[0xAA]);
    assert_encode_size_agrees(&req);
}

#[test]
fn rc_request_rejects_short_buffer() {
    assert!(<RoutineControlRequest as Decode>::decode(&[0x01, 0xFF]).is_err());
}

#[test]
fn rc_response_round_trips_and_rejects_sprmib_bit() {
    let resp = RoutineControlResponse::new(RoutineControlSubFunction::StartRoutine, 0xFF00, &[0x10]);
    let mut buf = [0u8; 8];
    let n = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
    assert_eq!(&buf[..n], &[0x01, 0xFF, 0x00, 0x10]);
    let (d, _) = <RoutineControlResponse as Decode>::decode(&buf[..n]).unwrap();
    assert_eq!(d.routine_control_type(), RoutineControlSubFunction::StartRoutine);
    assert_eq!(d.routine_id(), 0xFF00);
    // A response with the SPRMIB bit set (0x81) is malformed and rejected.
    assert!(<RoutineControlResponse as Decode>::decode(&[0x81, 0xFF, 0x00]).is_err());
    assert_encode_size_agrees(&resp);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test rc_request rc_response -v`
Expected: FAIL — new constructor/getter signatures don't exist.

- [ ] **Step 3: Reshape the request**

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequest<'d> {
    sub_function: SuppressablePositiveResponse<RoutineControlSubFunction>,
    routine_id: u16,
    option_record: &'d [u8],
}

impl<'d> RoutineControlRequest<'d> {
    #[must_use]
    pub const fn new(
        suppress_positive_response: bool,
        sub_function: RoutineControlSubFunction,
        routine_id: u16,
        option_record: &'d [u8],
    ) -> Self {
        Self {
            sub_function: SuppressablePositiveResponse::new(suppress_positive_response, sub_function),
            routine_id,
            option_record,
        }
    }

    #[must_use]
    pub fn suppress_positive_response(&self) -> bool { self.sub_function.suppress_positive_response() }
    #[must_use]
    pub fn sub_function(&self) -> RoutineControlSubFunction { self.sub_function.value() }
    #[must_use]
    pub const fn routine_id(&self) -> u16 { self.routine_id }
    #[must_use]
    pub const fn option_record(&self) -> &[u8] { self.option_record }
}

impl Encode for RoutineControlRequest<'_> {
    fn encoded_size(&self) -> usize { 1 + 2 + self.option_record.len() }
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[u8::from(self.sub_function)]).map_err(Error::io)?;
        writer.write_all(&self.routine_id.to_be_bytes()).map_err(Error::io)?;
        writer.write_all(self.option_record).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for RoutineControlRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 3 {
            return Err(Error::InsufficientData(3));
        }
        let sub_function = SuppressablePositiveResponse::try_from(buf[0])?;
        let routine_id = u16::from_be_bytes([buf[1], buf[2]]);
        Ok((Self { sub_function, routine_id, option_record: &buf[3..] }, &[]))
    }
}
```

- [ ] **Step 4: Reshape the response**

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponse<'d> {
    routine_control_type: RoutineControlSubFunction,
    routine_id: u16,
    status_record: &'d [u8],
}

impl<'d> RoutineControlResponse<'d> {
    #[must_use]
    pub const fn new(routine_control_type: RoutineControlSubFunction, routine_id: u16, status_record: &'d [u8]) -> Self {
        Self { routine_control_type, routine_id, status_record }
    }
    #[must_use]
    pub const fn routine_control_type(&self) -> RoutineControlSubFunction { self.routine_control_type }
    #[must_use]
    pub const fn routine_id(&self) -> u16 { self.routine_id }
    #[must_use]
    pub const fn status_record(&self) -> &[u8] { self.status_record }
}

impl Encode for RoutineControlResponse<'_> {
    fn encoded_size(&self) -> usize { 1 + 2 + self.status_record.len() }
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[u8::from(self.routine_control_type)]).map_err(Error::io)?;
        writer.write_all(&self.routine_id.to_be_bytes()).map_err(Error::io)?;
        writer.write_all(self.status_record).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for RoutineControlResponse<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 3 {
            return Err(Error::InsufficientData(3));
        }
        // Plain try_from (no SPRMIB mask): a set 0x80 bit on a response is malformed.
        let routine_control_type = RoutineControlSubFunction::try_from(buf[0])?;
        let routine_id = u16::from_be_bytes([buf[1], buf[2]]);
        Ok((Self { routine_control_type, routine_id, status_record: &buf[3..] }, &[]))
    }
}
```

(`u8::from(RoutineControlSubFunction)` already exists in `lib.rs`/moved module.)

- [ ] **Step 5: Run tests**

Run: `cargo test rc_request rc_response && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: PASS — the `Request`/`Response` `RoutineControl(...)` arms wrap these descriptors unchanged.

- [ ] **Step 6: Commit**

```bash
git add src/services/routine_control.rs
git commit -m "RoutineControl req/resp: typed u16 routine_id + opaque record"
```

### Task 9: `ReadDataByIdentifierRequest` bidirectional `Dids` backing (Decisions A, G)

**Files:**

- Modify: `src/services/read_data_by_identifier.rs`, `src/request.rs` (variant becomes `ReadDataByIdentifier(ReadDataByIdentifierRequest<'a>)`), `src/lib.rs` (rename re-export), `src/services/mod.rs` (rename re-export)

- [ ] **Step 1: Write failing tests (native build, wire decode, cross-backing, odd/empty rejects)**

```rust
#[test]
fn rdbi_native_encodes_be() {
    let req = ReadDataByIdentifierRequest::new(&[0xF190, 0xF186]);
    let mut buf = [0u8; 8];
    let n = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
    assert_eq!(&buf[..n], &[0xF1, 0x90, 0xF1, 0x86]);
    assert_encode_size_agrees(&req);
}

#[test]
fn rdbi_wire_decodes_and_dids_iterates() {
    let (req, rest) = <ReadDataByIdentifierRequest as Decode>::decode(&[0xF1, 0x90, 0xF1, 0x86]).unwrap();
    assert!(rest.is_empty());
    // Iterate without alloc (no_std-friendly): pull items directly.
    let mut it = req.dids();
    assert_eq!(it.next(), Some(0xF190));
    assert_eq!(it.next(), Some(0xF186));
    assert_eq!(it.next(), None);
    assert_encode_size_agrees(&req);
}

#[test]
fn rdbi_cross_backing_encodes_identically() {
    let native = ReadDataByIdentifierRequest::new(&[0xF190]);
    let mut a = [0u8; 4];
    let na = Encode::encode(&native, &mut a.as_mut_slice()).unwrap();
    let (wire, _) = <ReadDataByIdentifierRequest as Decode>::decode(&a[..na]).unwrap();
    let mut b = [0u8; 4];
    let nb = Encode::encode(&wire, &mut b.as_mut_slice()).unwrap();
    assert_eq!(a[..na], b[..nb]);
}

#[test]
fn rdbi_rejects_empty_and_odd() {
    assert!(<ReadDataByIdentifierRequest as Decode>::decode(&[]).is_err());
    assert!(<ReadDataByIdentifierRequest as Decode>::decode(&[0xF1]).is_err());
    assert!(<ReadDataByIdentifierRequest as Decode>::decode(&[0xF1, 0x90, 0xF1]).is_err());
}
```

Note for the iterate test: collect into a fixed array via a manual loop to stay `no_std`/no-alloc — replace with:

```rust
    let mut it = req.dids();
    assert_eq!(it.next(), Some(0xF190));
    assert_eq!(it.next(), Some(0xF186));
    assert_eq!(it.next(), None);
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test rdbi_ -v`
Expected: FAIL — type is still `ReadDataByIdentifierRequestTx` with `dids: &[u16]` and no `Decode`/`dids()`.

- [ ] **Step 3: Implement the bidirectional type**

```rust
//! `ReadDataByIdentifier` (0x22) service implementation
use crate::{Decode, Encode, Error, NegativeResponseCode};

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

/// Read-DID request: a list of 16-bit Data Identifiers. Built from native `&[u16]`
/// or borrowed from the wire as big-endian bytes; `dids()` yields `u16` either way.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadDataByIdentifierRequest<'d> {
    dids: Dids<'d>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Dids<'d> {
    Native(&'d [u16]),
    Wire(&'d [u8]),
}

impl<'d> ReadDataByIdentifierRequest<'d> {
    /// Build a request from a list of Data Identifiers.
    #[must_use]
    pub const fn new(dids: &'d [u16]) -> Self {
        Self { dids: Dids::Native(dids) }
    }

    /// Iterate the requested DIDs as `u16` (big-endian swap hidden for wire-backed values).
    pub fn dids(&self) -> impl Iterator<Item = u16> + '_ {
        DidIter { dids: self.dids, pos: 0 }
    }

    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DID_NEGATIVE_RESPONSE_CODES
    }
}

struct DidIter<'d> {
    dids: Dids<'d>,
    pos: usize,
}

impl Iterator for DidIter<'_> {
    type Item = u16;
    fn next(&mut self) -> Option<u16> {
        match self.dids {
            Dids::Native(s) => {
                let v = *s.get(self.pos)?;
                self.pos += 1;
                Some(v)
            }
            Dids::Wire(b) => {
                let hi = *b.get(self.pos)?;
                let lo = *b.get(self.pos + 1)?;
                self.pos += 2;
                Some(u16::from_be_bytes([hi, lo]))
            }
        }
    }
}

impl Encode for ReadDataByIdentifierRequest<'_> {
    fn encoded_size(&self) -> usize {
        match self.dids {
            Dids::Native(s) => s.len() * 2,
            Dids::Wire(b) => b.len(),
        }
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        match self.dids {
            Dids::Native(s) => {
                for did in s {
                    writer.write_all(&did.to_be_bytes()).map_err(Error::io)?;
                }
            }
            Dids::Wire(b) => writer.write_all(b).map_err(Error::io)?,
        }
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for ReadDataByIdentifierRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() || buf.len() % 2 != 0 {
            return Err(Error::IncorrectMessageLengthOrInvalidFormat);
        }
        Ok((Self { dids: Dids::Wire(buf) }, &[]))
    }
}
```

- [ ] **Step 4: Rewire the dispatch enum + re-exports**

In `src/request.rs`: change the variant to `ReadDataByIdentifier(ReadDataByIdentifierRequest<'a>)`. In `decode`, replace `Self::ReadDataByIdentifier(payload)` with `Self::ReadDataByIdentifier(<ReadDataByIdentifierRequest as Decode>::decode_exact(payload)?)`. In `encoded_size`/`encode`, replace the bare-`bytes` arms with `req.encoded_size()` / `req.encode(writer)?`. Update the `use` list (`ReadDataByIdentifierRequest`).
In `src/services/mod.rs` and `src/lib.rs`: rename the re-export `ReadDataByIdentifierRequestTx` → `ReadDataByIdentifierRequest`.

- [ ] **Step 5: Run tests**

Run: `cargo test rdbi_ && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: PASS. Confirm the suffix is gone:

```bash
rg -n "RequestTx|ResponseTx|\bRx\b" src   # expect: only CommunicationControl Rx/Tx domain terms
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "ReadDataByIdentifierRequest: bidirectional Dids backing; drop last Tx suffix"
```

### Task 10: `RequestTransferExit` descriptors (Decision H)

**Files:**

- Create: `src/services/request_transfer_exit.rs`

- Modify: `src/services/mod.rs` (`mod request_transfer_exit; pub use …`), `src/lib.rs` (re-export the two types), `src/request.rs` + `src/response.rs` (wrap the descriptors)

- [ ] **Step 1: Write failing round-trip tests (in the new file)**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decode, Encode, test_util::assert_encode_size_agrees};

    #[test]
    fn rte_request_round_trips_with_and_without_record() {
        for rec in [&[][..], &[0xAA, 0xBB][..]] {
            let req = RequestTransferExitRequest::new(rec);
            let mut buf = [0u8; 8];
            let n = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
            assert_eq!(&buf[..n], rec);
            let (d, rest) = <RequestTransferExitRequest as Decode>::decode(&buf[..n]).unwrap();
            assert!(rest.is_empty());
            assert_eq!(d.parameter_record(), rec);
            assert_encode_size_agrees(&req);
        }
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test rte_request -v`
Expected: FAIL — module/type does not exist.

- [ ] **Step 3: Implement the descriptors**

```rust
//! `RequestTransferExit` (0x37 / 0x77) service implementation.
use crate::{Decode, Encode, Error};

macro_rules! transfer_exit_descriptor {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub struct $name<'d> {
            parameter_record: &'d [u8],
        }
        impl<'d> $name<'d> {
            /// Create from the optional parameter record (empty slice if absent).
            #[must_use]
            pub const fn new(parameter_record: &'d [u8]) -> Self {
                Self { parameter_record }
            }
            /// The optional, opaque parameter record.
            #[must_use]
            pub const fn parameter_record(&self) -> &[u8] {
                self.parameter_record
            }
        }
        impl Encode for $name<'_> {
            fn encoded_size(&self) -> usize { self.parameter_record.len() }
            fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
                writer.write_all(self.parameter_record).map_err(Error::io)?;
                Ok(self.parameter_record.len())
            }
        }
        impl<'a> Decode<'a> for $name<'a> {
            fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
                Ok((Self { parameter_record: buf }, &[]))
            }
        }
    };
}

transfer_exit_descriptor!(RequestTransferExitRequest, "Request to exit a transfer, carrying an optional parameter record.");
transfer_exit_descriptor!(RequestTransferExitResponse, "Positive response to RequestTransferExit, carrying an optional parameter record.");
```

- [ ] **Step 4: Wrap in the dispatch enums**

In `src/request.rs`: `RequestTransferExit(RequestTransferExitRequest<'a>)`; decode arm `UdsServiceType::RequestTransferExit => Self::RequestTransferExit(<RequestTransferExitRequest as Decode>::decode_exact(payload)?)`; `encoded_size`/`encode` arms delegate to the inner descriptor. In `src/response.rs`: same with `RequestTransferExitResponse`. Add the types to the `use` lists and the crate-root + `services::mod` re-exports.

- [ ] **Step 5: Run tests**

Run: `cargo test rte_ && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "RequestTransferExit: descriptors carry the optional parameter record"
```

### Task 11: `ControlDTCSettings` → `SuppressablePositiveResponse<DtcSettings>` (Decisions D, I2)

**Files:**

- Modify: `src/services/control_dtc_settings.rs` (now also home to `DtcSettings` after Task 3)

- Modify: `src/lib.rs` (delete `SUCCESS`)

- [ ] **Step 1: Update the failing test for the new ctor order + getters**

Replace `simple_request`:

```rust
#[cfg(feature = "alloc")]
#[test]
fn simple_request() {
    // suppress-first ctor, matching the sibling sub-function services.
    let req = ControlDTCSettingsRequest::new(true, DtcSettings::On);
    let mut buffer = Vec::new();
    let written = Encode::encode(&req, &mut buffer).unwrap();
    assert_eq!(buffer, vec![0x81]); // On (0x01) | SPRMIB (0x80)
    assert_eq!(written, buffer.len());
    let (parsed, _) = <ControlDTCSettingsRequest as Decode>::decode(&buffer).unwrap();
    assert_eq!(parsed.setting(), DtcSettings::On);
    assert!(parsed.suppress_positive_response());
    assert_encode_size_agrees(&req);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test simple_request -v`
Expected: FAIL — `new(bool, DtcSettings)` and `setting()` don't exist.

- [ ] **Step 3: Reshape the request onto the wrapper**

```rust
use crate::common::SuppressablePositiveResponse; // path is crate::shared after Phase 0
use crate::{Decode, DtcSettings, Encode, Error};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ControlDTCSettingsRequest {
    setting: SuppressablePositiveResponse<DtcSettings>,
}

impl ControlDTCSettingsRequest {
    #[must_use]
    pub fn new(suppress_positive_response: bool, setting: DtcSettings) -> Self {
        Self { setting: SuppressablePositiveResponse::new(suppress_positive_response, setting) }
    }
    #[must_use]
    pub fn setting(&self) -> DtcSettings { self.setting.value() }
    #[must_use]
    pub fn suppress_positive_response(&self) -> bool { self.setting.suppress_positive_response() }
}

impl Encode for ControlDTCSettingsRequest {
    fn encoded_size(&self) -> usize { 1 }
    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(&[u8::from(self.setting)]).map_err(Error::io)?;
        Ok(1)
    }
}

impl<'a> Decode<'a> for ControlDTCSettingsRequest {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let setting = SuppressablePositiveResponse::try_from(buf[0])?;
        Ok((Self { setting }, &buf[1..]))
    }
}
```

(Use the correct post-Phase-0 path `crate::shared::SuppressablePositiveResponse` — it is `pub(crate)`. The response type is unchanged.)

- [ ] **Step 4: Delete the `SUCCESS` constant**

In `src/lib.rs`, remove `pub const SUCCESS: u8 = 0x80;` and its doc comment. Confirm no references:

```bash
rg -n "\bSUCCESS\b" src   # expect: no matches
```

- [ ] **Step 5: Run tests**

Run: `cargo test control_dtc simple_request && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: PASS — `Request::is_positive_response_suppressed` still forwards via `suppress_positive_response()`.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "ControlDTCSettings onto SuppressablePositiveResponse; remove SUCCESS const"
```

### Task 12: Delete the `PENDING` constant (Decision D)

**Files:**

- Modify: `src/lib.rs`

- [ ] **Step 1: Remove the constant**

Delete `pub const PENDING: u8 = 0x78;` and its doc comment from `src/lib.rs`. Confirm nothing used it:

```bash
rg -n "\bPENDING\b" src   # expect: no matches
```

- [ ] **Step 2: Build + test**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "remove unused PENDING constant (use NegativeResponseCode instead)"
```

______________________________________________________________________

# PHASE 3 — Final verification

### Task 13: `NegativeResponse` echoed-service edge doc (Decision F)

**Files:**

- Modify: `src/services/negative_response.rs`

- [ ] **Step 1: Document the accepted edge on `request_service`**

Update the doc comment on `NegativeResponse.request_service` to state: "For a service this library does not model, an unrecognized echoed byte decodes to `UdsServiceType::UnsupportedDiagnosticService` and re-encodes as `0x7F` — an accepted, documented edge (a one-byte normalization on an already-unsupported service)."

- [ ] **Step 2: Build + commit**

Run: `cargo test && cargo fmt --check`

```bash
git add src/services/negative_response.rs
git commit -m "document NegativeResponse echoed-service normalization edge"
```

### Task 14: README + crate-root coverage docs sweep

**Files:**

- Modify: `README.md`, `src/lib.rs` (crate-root docs if they name renamed types)

- [ ] **Step 1: Grep for stale names in docs**

```bash
rg -n "RequestTransferExit|ReadDataByIdentifierRequestTx|SUCCESS|PENDING|common::" README.md src/lib.rs
```

Fix any reference to the removed `*Tx` name, `SUCCESS`/`PENDING`, or `common::` paths in prose/snippets. Verify the modeled-vs-`Other` service-coverage list still reads correctly.

- [ ] **Step 2: rustdoc + commit**

Run: `cargo doc --no-deps` (expect no broken intra-doc links) then:

```bash
git add README.md src/lib.rs
git commit -m "docs: update for reshaped API surface"
```

### Task 15: Full breaking-matrix validation

- [ ] **Step 1: Run every feature combo + bare-metal target**

```bash
cargo test
cargo test --no-default-features --features alloc
cargo test --no-default-features
cargo build --no-default-features --target thumbv6m-none-eabi
cargo build --no-default-features --features alloc --target thumbv6m-none-eabi
cargo clippy --all-features
cargo clippy --no-default-features --features alloc
cargo clippy --no-default-features
cargo fmt --check
cargo doc --no-deps
```

Expected: all green.

- [ ] **Step 2: Final surface grep**

```bash
rg -n "\bSUCCESS\b|\bPENDING\b|RequestTx|ReadDataByIdentifierRequestTx|crate::common|InvalidDiagnosticIdentifier" src
```

Expected: no matches (CommunicationControl `Rx`/`Tx` domain names are fine).

- [ ] **Step 3: No commit** — this is a validation gate before merge.

______________________________________________________________________

## Notes for the executor

- After Phase 0, the path to the SPRMIB wrapper is `crate::shared::SuppressablePositiveResponse` (Task 11's snippet shows the pre-rename path in a comment — use the `shared` path).
- The `Request`/`Response` `Encode`/`encoded_size`/`service`/`response_sid` match arms must stay exhaustive; the compiler enforces this after each variant reshape.
- Keep the `decode_exact` calls in the dispatch enums for the wrapped descriptors — they enforce the no-trailing-bytes contract that previously didn't apply to the bare-slice/unit variants.
