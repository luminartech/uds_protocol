# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/) (treating
`0.x` breaking changes as minor bumps, per the Cargo/SemVer convention for
pre-1.0 crates).

## [Unreleased]

## [0.1.0](https://github.com/luminartech/uds_protocol/compare/v0.0.2...v0.1.0) - 2026-07-31

### Added

- absorb codec width-helper errors into Error
- [**breaking**] restructure Error around automotive-wire-codec Incomplete/TrailingBytes
- named ReadDataByIdentifierResponse type for Response symmetry
- named ClearDiagnosticInfoResponse type for Response symmetry
- allowed_nack_codes for RequestFileTransfer, RequestTransferExit, TransferData
- allowed_nack_codes for ControlDTCSettings, ReadDTCInfo, RoutineControl
- Eq + const new on ReadDTCInfo types; Eq on DTCFormatIdentifier
- derive Eq across RequestFileTransfer types
- derive Eq on ControlDTCSettingsResponse
- serde/utoipa on WriteDataByIdentifier request/response
- serde/utoipa on SecurityAccess request/response
- serde/utoipa on RoutineControl request/response
- serde + Eq on RequestDownload request/response
- non_exhaustive on ReadDataByIdentifierRequest; document serde+utoipa carve-out
- full derive block on TransferData request/response
- *(ci)* Initial CI impl.

### Changed

- [**breaking**] apply C-CASE acronym convention to all type and variant names
- [**breaking**] give UdsServiceType symmetric SID conversion names
- derive encode byte counts from codec write helpers
- [**breaking**] migrate to automotive-wire-codec 0.3 traits
- [**breaking**] use codec width helpers, remove param_length_*
- encode returns counted bytes, not self.encoded_size()
- standardize newtype accessor on const value()

### Documentation

- changelog + test polish for wire-codec 0.3 migration
- update for reshaped API surface
- document NegativeResponse echoed-service normalization edge
- document the modeled vs pass-through service coverage boundary
- document runtime-agnostic integration model and borrow semantics
- document the Decode remainder / borrow contract

### Fixed

- make DTCFaultDetectionCounterRecord reachable from the crate root
- *(ci)* let the coverage job run the property tests it was excluding
- rename test bindings to satisfy clippy::similar_names
- address final-review findings (strict ClearDiagnosticInfo decode test, docs, fmt)
- fix plan clippy gate to match CI (no -D warnings / --all-targets)
- fix clippy doc_markdown warnings introduced by no_std refactor
- fix rustdoc unresolved intra-doc link warnings
- fix serde Deserialize on borrowed file-transfer enums and a dead import
- fix bug with payload size of ProtocolRoutinePayload

### Other

- .gitignore omniscient files
- ignore semantic search config
- cargo fmt
- Make SecurityAccess levels SPRMIB-safe by construction; fix decode comment
- cargo fmt... --all
- add tooling to .gitignore
- Flatten remaining non-invariant request/response data bags
- Flatten suppressable request structs into public data bags
- Address PR review doc/test nits
- Preserve the raw echoed SID in NegativeResponse for lossless round-trips
- Apply mdformat to markdown (README + planning/spec docs)
- Apply rustfmt to review-fix edits
- Harmonize len()/is_empty() across the DTC iterators
- Add value() to DTCStoredDataRecordNumber for RX inspection
- Document ReadDTCInfo partial coverage and the severity-list record format
- Drop misleading repr(u16) from UDSRoutineIdentifier; prove totality
- Align RoutineControl response accessor name with the request
- Unify field visibility: encapsulate iff invariant-bearing
- Carry the offending byte on an invalid DtcSettings value
- Tidy UdsServiceType: dedupe doc, align byte-converter receivers
- Remove dead DTC code
- Make the simple service constructors const fn
- Add #[non_exhaustive] to the four outlier request/response structs
- Document why the ReadDataByIdentifier response stays raw
- Derive file-transfer length prefixes from the data
- Expose DataFormatIdentifier from the crate root
- Make CommunicationControl constructors fallible
- Fix clippy pedantic lints in tests under stricter rebased CI gate
- remove dead InvalidDiagnosticIdentifierPayload error variant
- remove unused PENDING constant (use NegativeResponseCode instead)
- ControlDTCSettings onto SuppressablePositiveResponse; remove SUCCESS const
- descriptors carry the optional parameter record
- bidirectional Dids backing; drop last Tx suffix
- RoutineControl req/resp: typed u16 routine_id + opaque record
- typed u16 identifier + opaque data
- Other carries raw sid for lossless pass-through; add Response::service()
- rebuild UDSIdentifier as faithful total From<u16>; drop identifier-enum codecs
- also check actual bytes consumed
- relocate single-service enums into their service modules
- extract dtc/ domain module from shared/
- rename common/ module to shared/ (no behavior change)
- add implementation plan for the pre-merge API interrogation
- expand interrogation design doc with implementation-detail resolutions
- add pre-merge public-API interrogation resolutions design doc
- add Decode for DTC params and ReadDTCInfoRequest; wrap Request::ReadDTCInfo
- wrap RoutineControl in descriptors; round-trip SPRMIB via SuppressablePositiveResponse
- wrap WriteDataByIdentifier request/response in their descriptor types
- merge identical signed/unsigned primitive codec macros
- dedup variable-length big-endian integer codec into util helpers
- rename bidirectional descriptor types: drop misleading Tx/Rx suffixes
- add Phase 2 implementation plan (API consistency)
- add API consistency Phase 2 design doc
- add crate-root integration tests for completed descriptor types
- add Decode for WriteDataByIdentifierResponse (2-byte round-trip)
- implement Encode for ReadDTCInfoSubFunction and ReadDTCInfoRequest
- add Encode to DTC parameter types; fix FunctionalGroupIdentifier::value panic
- make crate-root re-exports explicit (drop glob re-exports)
- add Phase 1 implementation plan (API exposure & consistency)
- add API exposure & consistency design doc
- drop unused Vec import in request_download tests
- gate Vec-using tests behind alloc so the no_std test matrix compiles
- correct README scope: embedded-first no_std codec
- move service-coverage docs into published README docs
- assert encode/encoded_size agreement across all services
- add symmetric Other escape hatch; drop UdsResponse + ServiceNotImplemented
- move is_positive_response_suppressed off the Encode trait
- remove Identifier machinery; add direct codec to UDS identifiers
- delete protocol_definitions module (ProtocolIdentifier/PayloadTx)
- remove orphaned DiagnosticDefinition trait and UdsSpec
- de-genericize RoutineControl to raw-bytes RoutineControl*Tx types
- de-genericize WriteDataByIdentifier to raw bytes + u16 response
- de-genericize ReadDataByIdentifierRequestTx to &[u16]
- add encode/encoded_size agreement test helper
- add no_std API alignment implementation plan
- elevate C-developer simplicity to a first-class principle in spec
- add no_std API alignment design spec
- reject trailing bytes when decoding Request/Response
- forward is_positive_response_suppressed in Request
- make DiagnosticDefinition lifetime-generic
- restore allowed_nack_codes on WriteDataByIdentifierRequest
- clamp RequestDownload memory lengths to at least one byte
- preserve error kind in From<std::io::Error>
- simplify Identifier encode conversion
- make crate buildable on bare-metal no_std targets
- clean up dead code and add missing docs after trait removal
- update all tests to use Encode/Decode traits
- remove deprecated WireFormat traits and old Vec-based types
- deprecate old WireFormat traits and add no_std API tests
- add alloc-gated convenience methods on no_std types
- add RequestRx/ResponseRx enums and DiagnosticDefinitionTx trait
- add zero-copy RX types and lazy iterators for DTC responses
- add zero-alloc TX types for variable-length services
- implement Encode + Decode for fixed-size services and primitives
- add Encode/Decode/DecodeIter traits and fix deps for no_std
- Switch to using byteorder-embedded-io
- prepare error type and dependencies for no_std
- Fix clippy::cast_possible_truncation in file size validation
- Run cargo fmt
- More thinking about the error.
- Fix errors.
- Revert file.
- Re-add TODO.
- Revert inclusion of 0 as valid.
- Revert bit masking changes.
- Revert file.
- Fix token.
- Correct comment. Address review comments.
- Pre-commit.
- Address review comments.
- Fix fuzz test errors.
- Fix fuzzing. Fix semver checks.
- Adjust release process.
- Add publish workflow.
- Fix fuzz test.
- Fix CI errors.
- cargo fmt
- replace uds_protocol_derive proc-macro crate with macro_rules
- make enums non_exhaustive to prevent breaking api changes
- check msrv on CI
- Updates for first crates.io release
- Remove reference to internal issue
- unify depencencies
- Fix bug parsing optional value in EcuResetResponse
- Add tests to verify sprmib functionality
- Routine payloads are separate from DIDs. Add support
- Document public functions
- Fix trait impl sites
- Address issues with base traits
- standardize doc comments
- Standardize doc comments in services module
- Remove redundant dtc type, internal protocol type
- Add exception for clippy line count
- cargo fmt
- Add not implemented error for diagnostic definition request & response

These changes require at least a 0.1.0 -> 0.2.0 bump before the next release.

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
  - `FunctionalGroupIdentifier::VODBSystem` -> `VobdSystem`. The old name was a
    transposition typo; ISO 14229-1 Table D.1 names 0xFE `VOBDSystem`.

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
