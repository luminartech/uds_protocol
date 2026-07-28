# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/) (treating
`0.x` breaking changes as minor bumps, per the Cargo/SemVer convention for
pre-1.0 crates).

## [Unreleased]

These changes require at least a 0.1.0 -> 0.2.0 bump before the next release.

### Added

- `RequestUpload` (0x35 / 0x75) is now modeled, via `RequestUploadRequest` and
  `RequestUploadResponse` plus `Request::RequestUpload` / `Response::RequestUpload`. It was the
  conspicuous gap in the transfer story: `RequestDownload`, `TransferData` and
  `RequestTransferExit` were all modeled, so the download flow was complete while the
  structurally identical upload flow decoded only to `Request::Other`. ISO 14229-1 gives the
  two services the same message layout, so both pairs are now generated from one macro in
  `services/upload_download.rs` — a fix to the address/size width derivation cannot land on one
  service and miss the other. The two NRC tables are kept separate so they can diverge later.
- `Error::negative_response_code()` maps any decode error to the `NegativeResponseCode` a server
  should answer with, following ISO 14229-1: `0x13` for a malformed frame, `0x12` for an
  unsupported sub-function byte, `0x31` for an out-of-range parameter, and `0x10` only for
  `IoError`. Three `Error` variants documented their NRC in prose and eighteen said nothing, so
  every server had to re-derive the mapping against a `#[non_exhaustive]` enum.
- `Request::allowed_nack_codes()` dispatches to the per-service tables from a decoded
  `Request`, alongside the existing `service()` and `is_positive_response_suppressed()`. All 15
  request types already exposed the associated function, but reaching it from a `Request`
  required matching every variant. Returns an empty slice for `Request::Other`, meaning "NRC set
  unknown" rather than "no codes apply".
- `RequestDownloadRequest::data_format_identifier()`, plus
  `DataFormatIdentifier::compression_method()` and `DataFormatIdentifier::encryption_method()`.
  The DFI was previously write-only: a server could decode a download request but had no way to
  read the compression or encryption method it had been asked to use.
- `TesterPresentRequest::sub_function()` and `TesterPresentResponse::sub_function()`.

### Fixed (behaviour)

- **Breaking:** `TesterPresentRequest` no longer rewrites reserved sub-function bytes.
  `[0x3E, 0x01]` decoded and re-encoded as `[0x3E, 0x00]`: the reserved value was parsed and
  then discarded. The normalization was deliberate, but it left the service inconsistent with
  its own response type, which retains the same values, and with every other service
  (`ResetType::IsoSaeReserved`, `DiagnosticSessionType::IsoSaeReserved`, ...). The value is now
  retained in a private field — callers still cannot mint a reserved value, `new(suppress)`
  keeps its signature and its `0x00` encoding — so a server can report
  `subFunctionNotSupported` naming the byte it actually received.
- `TesterPresentResponse::new()` is now `const`, the last `new()` in the crate that was not.
- The README service table was missing rows for `DynamicallyDefinedDataIdentifier` (0x2C) and
  `AccessTimingParameter` (0x83), both enumerated in `UdsServiceType`, and named two services
  differently from the code (`ECUReset`, `ControlDTCSetting`). The table's 27 rows now match the
  27 request SIDs in `UdsServiceType` exactly.

### Changed (API consistency pass)

- **Breaking:** Acronyms in type and variant names now follow the Rust API guideline
  ([C-CASE](https://rust-lang.github.io/api-guidelines/naming.html)): `Dtc`, `Uds`, `Ecu`,
  `Obd`, `Edr`, `Iso`, `Sae`, `Vin`, `Rpm`, `WwhObd`, `IsoSae`. The crate previously used
  both conventions at once — `UdsServiceType` and `DtcAndStatusIter` beside `UDSIdentifier`
  and `DTCStatusMask`. Type renames:

  | Old                              | New                              |
  | -------------------------------- | -------------------------------- |
  | `UDSIdentifier`                  | `UdsIdentifier`                  |
  | `UDSRoutineIdentifier`           | `UdsRoutineIdentifier`           |
  | `DTCRecord`                      | `DtcRecord`                      |
  | `DTCStatusMask`                  | `DtcStatusMask`                  |
  | `DTCSeverityMask`                | `DtcSeverityMask`                |
  | `DTCFormatIdentifier`            | `DtcFormatIdentifier`            |
  | `DTCExtDataRecordNumber`         | `DtcExtDataRecordNumber`         |
  | `DTCSnapshotRecordNumber`        | `DtcSnapshotRecordNumber`        |
  | `DTCStoredDataRecordNumber`      | `DtcStoredDataRecordNumber`      |
  | `DTCFaultDetectionCounterRecord` | `DtcFaultDetectionCounterRecord` |
  | `ControlDTCSettingRequest`       | `ControlDtcSettingRequest`       |
  | `ControlDTCSettingResponse`      | `ControlDtcSettingResponse`      |
  | `ReadDTCInfoRequest`             | `ReadDtcInfoRequest`             |
  | `ReadDTCInfoResponse`            | `ReadDtcInfoResponse`            |
  | `ReadDTCInfoSubFunction`         | `ReadDtcInfoSubFunction`         |

  Enum variants follow the same rule mechanically (`UdsServiceType::ReadDTCInfo` ->
  `ReadDtcInfo`, `ISOSAEReserved` -> `IsoSaeReserved` across all nine enums that
  declare it, and so on). Four variant groups did **not** rename mechanically:

  - `ReadDtcInfoSubFunction` variants lose their underscores along with the enum's
    `#[allow(non_camel_case_types)]`: `ReportDTC_ByStatusMask` -> `ReportDtcByStatusMask`,
    `ReportWWHOBDDTC_ByMaskRecord` -> `ReportWwhObdDtcByMaskRecord`.
  - `DtcSeverityMask::DTCClass_0..4` -> `DtcClass0..4`.
  - `DtcFormatIdentifier` and `SecurityAccessType` variants **keep** the underscores that
    separate digit groups inside a standard's document number, because those carry meaning:
    `ISO_11992_4_DTCFormat` -> `Iso11992_4DtcFormat`, `ISO26021_2Values` ->
    `Iso26021_2Values`. (`non_camel_case_types` permits `_` between digits, so no `allow`
    is needed.) Trailing sequence numbers do lose theirs:
    `SAE_J2012_DA_DTCFormat_00` -> `SaeJ2012DaDtcFormat00`.
  - `FunctionalGroupIdentifier::VODBSystem` -> `VobdSystem`, which also corrects a typo —
    see *Fixed* below.

  All three `#[allow(non_camel_case_types)]` attributes in the crate are now gone.

- **Breaking:** `UdsServiceType`'s four SID conversions had two naming conventions for one
  concept and are now symmetric. Standardised on `sid` rather than `byte`, matching both ISO
  and the crate's existing `Request::Other { sid }` / `NegativeResponse::request_service_sid()`:

  | Old                         | New                 |
  | --------------------------- | ------------------- |
  | `service_from_request_byte` | `from_request_sid`  |
  | `request_service_to_byte`   | `to_request_sid`    |
  | `response_from_byte`        | `from_response_sid` |
  | `response_to_byte`          | `to_response_sid`   |

  Both `to_*` methods now document the lossy `0x7F` fallback for
  `NegativeResponse`/`UnsupportedDiagnosticService`, and point at `Request::Other` /
  `Response::Other` for lossless pass-through of unmodeled services.

- **Breaking:** `#[non_exhaustive]` added to eleven public types that ISO will grow into, so
  that later additions are not breaking changes: `ReadDtcInfoSubFunction`, `FileOperationMode`,
  `DtcExtDataRecordNumber`, `DtcSnapshotRecordNumber`, `DtcFaultDetectionCounterRecord`,
  `SizePayload`, `NamePayload`, `SentDataPayload`, `FileSizePayload`, `DirSizePayload`,
  `PositionPayload`. Downstream `match` statements over these need a wildcard arm, and the
  seven structs must be built through `new()` rather than a struct literal.
  `ReadDtcInfoSubFunction` is the important one: it has sub-functions the crate does not model
  yet, so adding one post-tag would otherwise have been breaking.

### Fixed

- **Breaking:** The `utoipa` and `clap` features now imply `std`. Neither compiled without it:
  their derive macros expand to `std::`, `String` and `Vec` paths inside this crate, so
  `cargo build --no-default-features --features utoipa` failed with 318 resolution errors, as
  did the `clap` equivalent. Only the `--all-features` / `--no-default-features` /
  `--no-default-features --features alloc` combinations were ever built, so the optional
  integrations were never exercised in isolation.
- The `serde` feature now works on a bare-metal target. The dependency was declared with
  serde's default features on, which pulls `serde/std`, so
  `cargo build --no-default-features --features serde --target thumbv6m-none-eabi` failed even
  though every host-side build passed — a host build proves nothing here, because the host has
  `std` available for serde to compile against regardless of this crate being `#![no_std]`.
  serde is now wired `default-features = false`, picking up its `alloc` and `std` layers
  through weak `serde?/alloc` and `serde?/std` features only when this crate's own `alloc`/`std`
  features are enabled.
- `FunctionalGroupIdentifier::VODBSystem` is now `VobdSystem`. Beyond the casing change, the
  old name transposed the letters: ISO 14229-1 Table D.1 names functional group `0xFE`
  `VOBDSystem` (vehicle OBD system).
- `DtcFaultDetectionCounterRecord` is now exported from the crate root. It is the `Item` of
  the public `DtcFaultDetectionIter`, but had no public path, so callers could iterate it and
  read its fields yet could not name the type — no `Vec<T>`, no struct field, no function
  signature. Its two `pub` fields are also documented now; `missing_docs` had never fired on
  an unreachable type.
- Four private type aliases no longer appear in public signatures, where rustdoc rendered them
  as unlinkable names: `DTCFaultDetectionCounter`, `MemorySelection` and
  `DTCReadinessGroupIdentifier` (all `= u8`) are spelled `u8` with the meaning moved into the
  field and variant docs, and `DTCStatusAvailabilityMask` (`= DtcStatusMask`) is spelled
  `DtcStatusMask` with its "bits on = supported by server" semantics moved onto the four
  `status_availability_mask` fields. This closes the `TODO` above sub-function `0x18`.
- `RequestTransferExitRequest` and `RequestTransferExitResponse` now derive `serde` and
  `utoipa` support like every other public request/response type. Enabling the `serde` feature
  previously left these two types unserializable.

### Removed

- The `serde_bytes` optional dependency. The `serde` feature activated it, but the crate never
  referenced it.

### Changed

- **Breaking:** `Error::InsufficientData` now carries an `automotive_wire_codec::Incomplete`
  (with `needed` and `available` byte counts) instead of a bare `usize`.
- `automotive-wire-codec` is now a public dependency: its `Incomplete` and `TrailingBytes`
  types are re-exported at the crate root (`uds_protocol::{Incomplete, TrailingBytes}`) and
  are considered part of `uds_protocol`'s public API. A semver-major release of
  `automotive-wire-codec` is therefore a breaking change for `uds_protocol`.

### Added

- **Breaking:** New `Error::TrailingBytes` variant, produced when a decode leaves unconsumed
  bytes in the input. Both `Error::InsufficientData` and `Error::TrailingBytes` map to
  NRC `0x13` (`IncorrectMessageLengthOrInvalidFormat`).

### Removed

- The `byteorder-embedded-io` dependency, superseded by `automotive-wire-codec`.
- **Breaking:** `param_length_u16`/`param_length_u32`/`param_length_u64`/`param_length_u128` have
  been removed. Use `automotive_wire_codec::minimal_be_len` instead.
- **Breaking:** `uds_protocol`'s `Encode`/`Decode` implementations for the primitive numeric types
  (`u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64`) have been removed. Now that
  `Encode`/`Decode` are re-exports of `automotive-wire-codec`'s traits, implementing them on
  foreign primitive types would violate Rust's orphan rule — the codec intentionally ships no
  blanket impls for primitives. Callers who relied on `Encode`/`Decode` for a bare primitive
  should switch to the leaf helpers (`read_u16_be`/`write_u16_be`/etc., or
  `read_be_uint`/`write_be_uint`/`read_be_uint_into` for variable-width fields) re-exported from
  `automotive_wire_codec`.

### Changed (migration to `automotive-wire-codec` 0.3)

- **Breaking:** `uds_protocol::{Decode, DecodeIter, Encode}` are now re-exports of the
  `automotive-wire-codec` 0.3 traits (previously crate-local traits). This is the underlying
  cause of most of the other breaking changes in this release.
- **Breaking:** `Encode::encoded_size` is now `Result<usize, Self::Error>` (previously infallible
  `usize`), via the codec trait's correct-by-construction counting-sink default. Crate-local
  `encoded_size` overrides have been removed; callers must handle/unwrap the `Result`.
- **Breaking:** Added `Error::InvalidWidth`, produced when a wire-declared variable-width field
  requests a byte width the target type cannot hold. The underlying `automotive_wire_codec::InvalidWidth`
  fragment is also re-exported at the crate root (alongside `Incomplete` and `TrailingBytes`).
- **Breaking:** `decode_exact` trailing-bytes now surface as `Error::TrailingBytes` instead of
  `Error::IncorrectMessageLengthOrInvalidFormat` (both still map to NRC 0x13).
- This release remains a semver-major bump.
