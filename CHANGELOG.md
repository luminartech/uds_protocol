# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/) (treating
`0.x` breaking changes as minor bumps, per the Cargo/SemVer convention for
pre-1.0 crates).

## [Unreleased]

These changes require at least a 0.1.0 -> 0.2.0 bump before the next release.

### Fixed — wire-format conformance

Found by a review pass that checked the crate against the ISO 14229-1:2020 text rather than
against itself. All five predate this branch; none was caught by the existing tests, because a
round-trip written against the crate's own output passes whenever `encode` and `decode` share a
misreading. `tests/spec_conformance.rs` now round-trips 36 byte sequences quoted from the
standard's own numbered example tables. That catches a legal frame the crate rejects, and an
`encode`/`decode` pair that disagree with each other — but not a misreading the two share
symmetrically, which still round-trips. See the module doc for where the line falls.

- **Breaking:** `MemoryFormatIdentifier::try_from` used exclusive range patterns where its own
  comments said inclusive, accepting a `memorySize` width of 1-3 and a `memoryAddress` width of
  1-4. Annex H Table H.1 runs to 4 and 5, and `RequestDownloadRequest::new` already documented and
  derived those wider values — so the crate emitted frames it could not decode. Any transfer above
  16 MB, or to an address above 4 GB, was unrepresentable, and ALFID `0x44` (the usual value in a
  real programming session) was rejected outright. The property test had its generators narrowed
  to exactly the buggy accepted set.

- **Breaking:** `EcuResetResponse::power_down_time` is now `Option<u8>`. Table 35 marks
  `powerDownTime` `Cvt` = `C`, present only for `enableRapidPowerShutDown`, and Table 39's flow
  example is the two-byte `51 01`. The field was a bare `u8` that `encode` always wrote, so
  decoding `51 01` and re-encoding produced `51 01 00`: any proxy that decoded and re-encoded
  traffic rewrote every positive response except 0x04's. `None` is now distinct from `Some(0)`,
  which the old decode conflated, and the doc named `0x00` as the not-available sentinel where
  Table 36 defines that as `0xFF`.

- **Breaking:** `ReadDtcInfoRequest` gained `suppress_positive_response`. `decode` matched the raw
  sub-function byte, so bit 7 — SPRMIB — was read as part of the sub-function value. Table 11
  requires both bit values for every supported sub-function. A suppressed
  `reportDTCByStatusMask` (`19 82 FF`) was rejected as having trailing bytes, and `19 8A` decoded
  as a *reserved* sub-function, so a server answered `SubFunctionNotSupported` to a request it was
  required to execute. 0x19 was the only one of the eight sub-function services not using the
  shared helper.

- **Breaking:** `ReadDtcInfoSubFunction::ReportUserDefMemoryDtcByStatusMask` gained the mandatory
  `MemorySelection` byte (Table 310). The conformant 4-byte request was rejected and the malformed
  3-byte one accepted — exactly inverted. Sub-functions 0x18 and 0x19 already read theirs.

- **Breaking:** `ReadDtcInfoResponse::NumberOfDtcs` gained `format_identifier`. Table 319 makes
  `DTCFormatIdentifier` mandatory between the availability mask and the count, so decode was
  reading it as the count's high byte: Table 341's own example `59 01 2F 01 00 01` was rejected
  outright, and a count of 1 came back as `0x0100`. It is also the only thing that says how to
  interpret a DTC's three bytes.

### Fixed — spec gaps in modeled services

- **Breaking:** `ControlDtcSettingRequest` gained `option_record: &'d [u8]` for the
  `DTCSettingControlOptionRecord` (Table 127, `Cvt` = `U`), so the type now borrows. Table 132
  reserves NRC `0x31` for an error *in that record*, which a server can only detect if the record
  reaches it. This is the same `Cvt = U` omission as `ClearDiagnosticInformation`'s
  `memorySelection`.

- **Breaking:** `DtcSettingType` gained `VehicleManufacturerSpecific` and `SystemSupplierSpecific`
  for the `0x40`-`0x5F` and `0x60`-`0x7E` ranges Table 128 reserves. Both were rejected outright,
  so a client could not send a manufacturer-defined setting and a server never saw the byte.

- **Breaking:** `CommunicationControl` now decodes the subnet nibble. Annex B Table B.1 splits
  the `communicationType` byte into message type (bits 1-0), reserved bits 3-2 and subnet number
  (bits 7-4), but the whole byte was matched against `0x00..=0x03`, so `0xF3` — a common
  real-world value — was rejected. New `SubnetNumber` type, plus
  `CommunicationControlRequest::with_subnet` and `::subnet`. The file's `TODO` admitting this gap
  is gone.

- **Breaking:** `RoutineControlSubFunction::try_from` returns
  `Error::InvalidRoutineControlSubFunction` rather than `IncorrectMessageLengthOrInvalidFormat`,
  so a reserved `routineControlType` is answered with NRC `0x12` as Table 430 requires rather than
  `0x13` — for a request whose length was perfectly correct. That variant had never been
  constructed anywhere in the crate despite being in the NRC mapping table and its test.

- **Breaking:** `WriteDataByIdentifierRequest::new` is now fallible and `decode` requires three
  payload bytes. Table 277 marks `dataRecord` byte #1 mandatory and Figure 26 states a four-byte
  minimum, but a three-byte message decoded into an empty data record, so a server would attempt
  a zero-length write instead of answering NRC `0x13`. A test had locked the wrong behaviour in.

- **Breaking:** `DtcStoredDataRecordNumber::new` no longer rejects `0xF0`. Clause 12.3.3.2
  reserves only `0x00` for this parameter; `0x00`/`0xF0` is the *snapshot* record-number space,
  and both the check and its doc had been copied from there.

- `RequestFileTransferRequest::allowed_nack_codes` gained `RequestSequenceError` (0x24),
  `SecurityAccessDenied` (0x33) and `AuthenticationRequired` (0x34), three of the seven codes
  Table 484 mandates. 0x24 exists specifically for `ResumeFile` against an already-complete
  transfer.

- **Breaking:** `AddressAndLengthFormatIdentifier` is public, and
  `RequestDownloadRequest::new_with_alfid` / `RequestUploadRequest::new_with_alfid` take one. Table
  441 makes the `addressAndLengthFormatIdentifier` a client choice, not a function of the values,
  and `new` derives only minimal widths — so the crate could not reproduce Table 462's own example
  (three `memorySize` bytes for `0x00FFFF`), nor satisfy a bootloader that mandates a fixed ALFID.

  A bootloader states that requirement as a byte, so the type is built from one:
  `AddressAndLengthFormatIdentifier::try_from(0x44)?`. That replaces a five-parameter
  `new_with_widths(dfi, address, address_len, size, size_len)` whose four numeric arguments
  included two transposable width/value pairs, with no compiler help — the nibbles are
  asymmetric (high is size, low is address), so a swap silently truncated.

  `new` now derives its widths and **delegates** to `new_with_alfid`. The two used to be
  independent code paths that disagreed: the same over-wide `memory_address` produced
  `InvalidMemoryAddress` from one and `IncorrectMessageLengthOrInvalidFormat` from the other, so
  one input yielded NRC `0x31` or `0x13` depending on which constructor a caller reached for.
  Nothing tested it. New `Error::InvalidMemorySize` completes the pair, and both map to `0x31` as
  Tables 444 and 449 require for a `memoryAddress`/`memorySize` that "is not valid".

  This also changes the `serde` shape of both requests: the widths were two derived scalars
  (`memory_address_length`, `memory_size_length`) and are now the single ISO-named
  `address_and_length_format_identifier` byte, matching Table 441 rather than paraphrasing it.

### Fixed — encapsulation

- **Breaking:** deserializing can no longer bypass a type's validation. `derive(Deserialize)`
  ignores both field visibility and constructor checks, so it was a second, unchecked way in for
  every sealed type: `serde_json::from_str::<SecurityAccessLevel>("255")` succeeded, producing a
  level that encoded to a byte which decoded back as a *different* level with SPRMIB set. The six
  single-byte types now serialize as their wire byte via `serde(try_from = "u8", into = "u8")`,
  and `TesterPresentRequest`, `CommunicationControlRequest` and the two transfer requests
  deserialize through a repr routed via their constructors. This changes their serialized
  representation, and removes the module-private `ZeroSubFunction` and the `pub(crate)`
  `MemoryFormatIdentifier` from both the serialized form and the generated OpenAPI schema, where
  they had been giving client generators types downstream Rust cannot name.

- `DtcFaultDetectionCounterRecord::new`. The type is `#[non_exhaustive]` with `pub` fields and had
  no `impl` block, so outside the crate a struct literal is `E0639` and there was no constructor
  to fall back on — the `Item` type of a public iterator was unbuildable by any downstream crate.
  Introduced on this branch by the `#[non_exhaustive]` sweep; the test meant to guard it was a
  unit test, where the attribute does not apply.

- **Breaking:** `UdsServiceType::DynamicallyDefinedDataIdentifier` is now
  `DynamicallyDefineDataIdentifier`, the name Table 23 gives service 0x2C.

### Fixed — API shape

- **Breaking:** `From<u32> for DtcRecord` is now `TryFrom<u32>`. It masked the top byte away, so
  `from(0x01_0203)` and `from(0xFF01_0203)` compared equal — a caller with a DTC one byte too wide
  got a wrong DTC and no signal. New `Error::InvalidDtcRecord`.

- **Breaking:** `RequestDownloadResponse::new` / `RequestUploadResponse::new` are fallible and
  `max_number_of_block_length` is behind an accessor. Its length becomes a single nibble, so it
  cannot exceed 15 bytes — a check that lived in `encode`, leaving a 16-byte slice constructible
  and only then unencodable, reachable through the public field as well.

- **Breaking:** `LengthFormatIdentifier` no longer zeroes the low nibble of the
  `lengthFormatIdentifier`, which ISO 14229-1 leaves undefined: `74 25 08 00` used to re-encode as
  `74 20 08 00`. Same reasoning as the earlier `TesterPresentRequest` fix. Its property test had
  generated `high_nibble << 4`, so the low nibble was always zero and the property held trivially.

- **Breaking:** `NamePayload` no longer carries `mode_of_operation` — that is the
  `RequestFileTransferRequest` variant, and nothing kept the two in step. The field won, so
  `AddFile(NamePayload::new(DeleteFile, ..), dfi, size)` encoded a DeleteFile request and dropped
  the format identifier and both sizes. New `RequestFileTransferRequest::mode_of_operation()`.

- **Breaking:** `DtcStoredDataRecordNumber::new` is total, matching both sibling record-number
  types, and gains `is_reserved()` plus the `PartialEq<u8>` they already had. It returned `Result`
  while `From<u8>` accepted anything, so the invariant it advertised was not one the type held.

- **Breaking:** four `Error` variants that no code path could construct are gone —
  `NoDataAvailable`, `InvalidFileSizeParameterLength`, `InvalidDtcFormatIdentifier` and
  `ReservedForLegislativeUse`. All were in the NRC mapping and asserted by its test, so the suite
  was green on unreachable states.

- **Breaking:** `#[non_exhaustive]` now covers the byte-carrying reserved variants of
  `DtcFormatIdentifier`, `FunctionalGroupIdentifier`, `FileOperationMode` and
  `ReadDtcInfoSubFunction`, matching the four sub-function enums that already had it. Naming one
  directly let a caller build a value that aliases a named variant — `IsoSaeReserved(0x01)` is not
  equal to `Iso14229_1DtcFormat` but encodes the same byte, so `PartialEq` silently stopped meaning
  wire equality.

- **Breaking:** `fault_detection_iter` is `dtc_fault_detection_iter`, matching
  `DtcFaultDetectionIter`; and `clap::Parser` is gone from `UdsIdentifier`, where it had made a
  53-variant data enum into a CLI parser with a `parse()` that reads `std::env::args`.

- `Request` and `Response` derive `Eq`/`PartialEq`; every payload type already did, so `assert_eq!`
  worked on a payload but not on the frame holding it.

- Byte extraction is now `const` on the five types that had only a non-const `From`
  (`NegativeResponseCode`, `DtcFormatIdentifier`, `FileOperationMode`, `DataFormatIdentifier`,
  `CommunicationControlType`, plus `DtcRecord::to_u32`). A `const` server dispatch table could be
  built but not read. Adding `CommunicationControlType::value()` also makes
  `CommunicationControlRequest::new` and `::new_with_node_id` `const` — all 47 constructors now are.

### Fixed — test integrity

Mutation testing over the suite found 11 of 23 mutants surviving. The gaps, now closed: no test
had ever asserted a value the DTC iterators yield, or any of the four header fields of the 0x42
response; `allowed_nack_codes` dispatch asserted only `!is_empty()`, so any of 16 arms could return
another service's table; the misaligned-record-list test never covered a list shorter than one
record, the boundary it exists for; the iterator-termination test drained unbounded, so a
regression hung `cargo test` rather than failing it; and six "round-trip" tests encoded into a
buffer they never read.

Ten codec tests were gated on `alloc` only because they used `Vec` as a writer. With stack buffers
the no_std configuration — the one the crate targets first — now runs 236 tests rather than 186,
and `clippy --no-default-features --all-targets` went from 21 warnings to zero.

`assert_encode_size_agrees`' doc comment now says what it does not prove: `encoded_size` counts by
encoding into a sink, so every quantity it compares comes from `encode` itself and none of them
constrain *which* bytes are written. Reading its call sites as byte-correctness coverage is what
made the two gaps above possible.

### Fixed — documentation

- `NegativeResponseCode::RequestCorrectlyReceivedResponsePending` (0x78) carried 0x73's
  description, calling it a `BlockSequenceCounter` error. It is the response-pending code that
  extends P2\*, and per Annex A.1 obliges the server to send a final response regardless of
  SPRMIB.

- `DiagnosticSessionControlResponse::p2_star_server_max` documented its scaling backwards. Table
  29 gives the parameter a 10 ms resolution, so the stored value is milliseconds ÷ 10; reading it
  as a raw millisecond count under-waits by a factor of ten.

- `ReadDtcInfoResponse`'s `InsufficientData` shortfalls measured `needed` including the
  sub-function byte while `available` was measured after it, so `needed - available` overstated
  the gap by one.

- `RoutineControlResponse::status_record` now says that it spans the optional `routineInfo` byte
  as well (Table 428). Nothing on the wire distinguishes the two layouts, so a general-purpose
  decoder cannot split them — but the field name implied it already had.

- `ReadDtcInfoSubFunction::IsoSaeReserved` listed `0x42` — a modeled report type — as reserved,
  and omitted most of the ranges Table 317 actually reserves.

- README: `AccessTimingParameters` is annotated as ISO 14229-1:2013-only (it was removed in the
  2020 edition the crate targets), the escaped brackets that rendered as dead `[Request]` text on
  the crate's front page are now real links, and the re-export list no longer says the codec
  traits are re-exported "eventually" — they already are.

### Added

- `NegativeResponse::new_with_sid(request_service_sid, nrc)`, the construction-side counterpart
  to `Request::Other { sid }`. `new()` routes through `to_request_sid()`, which collapses every
  unmodeled service to `0x7F`, so a server that decoded `Request::Other { sid: 0x40 }` could not
  answer `serviceNotSupported` echoing `0x40` — despite this type already preserving such bytes
  losslessly on decode.

- `Request::decode` and `Response::decode` now document that the returned remainder is **always
  empty**. A UDS frame is not self-delimiting, so one buffer is one frame and every payload is
  decoded with `decode_exact`; feeding concatenated frames (or `DecodeIter`) will treat the whole
  buffer as a single frame.

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
  `Request`, alongside the existing `service()` and `is_positive_response_suppressed()`. All 16
  request types already exposed the associated function, but reaching it from a `Request`
  required matching every variant. Returns an empty slice for `Request::Other`, meaning "NRC set
  unknown" rather than "no codes apply".

- `RequestDownloadRequest::data_format_identifier()`, plus
  `DataFormatIdentifier::compression_method()` and `DataFormatIdentifier::encryption_method()`.
  The DFI was previously write-only: a server could decode a download request but had no way to
  read the compression or encryption method it had been asked to use.

- `TesterPresentRequest::sub_function()` and `TesterPresentResponse::sub_function()`.

- **Breaking:** New `Error::TrailingBytes` variant, produced when a decode leaves unconsumed
  bytes in the input. Both `Error::InsufficientData` and `Error::TrailingBytes` map to
  NRC `0x13` (`IncorrectMessageLengthOrInvalidFormat`).

- `DtcRecord::high_byte()`, `middle_byte()` and `low_byte()`. The fields are private and the type
  had no accessors at all, so a decoded DTC could only be inspected by round-tripping through
  `u32`. All three are `const fn`. Their docs deliberately do not ascribe a meaning to any
  individual byte: ISO 14229-1 clause 12.3.2.3 specifies no decoding method for the three DTC
  bytes, deferring to whichever standard the `DTCFormatIdentifier` names.

### Changed

- **Breaking:** `ClearDiagnosticInfoRequest::memory_selection` is now `Option<u8>`, and the
  constructors are split accordingly: `new(group_of_dtc)` / `clear_all()` for the ordinary case,
  `new_with_memory_selection(group_of_dtc, selection)` / `clear_all_in_memory(selection)` when
  addressing user-defined DTC memory. ISO 14229-1:2020 Table 296 marks `MemorySelection` `U`
  (user option), so it is absent from the wire unless the client is targeting user-defined
  memory — the crate previously required it, which meant the plain 3-byte request (the only
  form in the 2013 edition, and the one in the standard's own Table 300 flow example) failed to
  decode with `InsufficientData`, while every encode emitted a spurious 4th byte.

- **Breaking:** `SecurityAccessLevel::value` now takes `&self` instead of `self`, matching the
  crate's other by-reference accessors. No call-site change is needed: the type is `Copy`.

- `DtcRecord::new`, `DtcSnapshotRecordNumber::new`, `DtcExtDataRecordNumber::new`,
  `DtcStoredDataRecordNumber::new`, `DataFormatIdentifier::new`, `NegativeResponse::new`,
  `NegativeResponse::request_service`, `RequestDownloadRequest::new`, `RequestUploadRequest::new`
  and all four `UdsServiceType` SID conversions (`from_request_sid`, `to_request_sid`,
  `from_response_sid`, `to_response_sid`) are now `const fn`. Most of the crate was already
  `const fn new`, and the gaps fell exactly on the primitives a caller wants in a `const` table:
  DTC constants, record numbers, the format identifier, and the SID map a server dispatch table
  is built from. `NegativeResponse::new` was blocked only because `to_request_sid` was not const.

- The DTC iterators now implement `size_hint` (exact) and
  [`FusedIterator`](core::iter::FusedIterator). They deliberately do **not** implement
  `ExactSizeIterator`: its `len()` would have to count items yielded, which exceeds the
  complete-record count when a partial tail is present, contradicting the inherent `len()`.
  Documented on each type.

- **Breaking:** `DtcSeverityAndStatusIter` -> `WwhObdDtcSeverityIter`, and
  `ReadDtcInfoResponse::severity_and_status_iter` -> `wwh_obd_dtc_severity_iter`. The old name pointed at
  the wrong variant: it reads as "the severity iterator" but only handles the 5-byte records of
  `WwhObdDtcByMaskRecord` (0x42), while the 0x08/0x09 `DtcSeverityList` records are 6 bytes with
  an extra functional-unit byte. The `DtcSeverityList` doc previously had to carry a warning that
  the iterator did not apply to it.

- **Breaking:** `DataFormatIdentifier::new` now takes its arguments in **wire order** —
  `new(compression_method, encryption_method)`, compression being the high nibble. It previously
  took encryption first, contradicting both the wire layout and the type's own doc comment, and
  since both parameters are `u8` the compiler could not catch a transposition. **Review any
  call site passing two different non-zero values.** `From<u8>` is unaffected and remains the
  usual path. Added `DataFormatIdentifier::NONE` for the common no-compression/no-encryption case.

- **Breaking:** `CommunicationControlRequest::suppress_positive_response()` is now a public
  field, matching the other six suppressable requests. The type stays encapsulated, but because
  of the `control_type`/`node_id` invariant — `node_id` must be present exactly when
  `control_type` is an enhanced-address variant — not because of SPRMIB, which is independent and
  fused onto the sub-function byte only at the wire boundary. `control_type()` remains a getter.

- **Breaking:** `ReadDataByIdentifierResponse::records()` is now the public field `records`,
  matching every other opaque response slice. It carries no invariant.

- `CommunicationControlResponse::control_type` (public) and `NegativeResponse`'s private fields
  are both deliberate and now documented, so the remaining asymmetry is not read as an oversight.

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
  | `ControlDTCSettingRequest`      | `ControlDtcSettingRequest`      |
  | `ControlDTCSettingResponse`     | `ControlDtcSettingResponse`     |
  | `ReadDTCInfoRequest`             | `ReadDtcInfoRequest`             |
  | `ReadDTCInfoResponse`            | `ReadDtcInfoResponse`            |
  | `ReadDTCInfoSubFunction`         | `ReadDtcInfoSubFunction`         |

  Enum variants follow the same rule mechanically (`UdsServiceType::ReadDTCInfo` ->
  `ReadDtcInfo`, `ISOSAEReserved` -> `IsoSaeReserved` across every enum that declares it, and
  so on). Four variant groups did **not** rename mechanically:

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
  seven structs must be built through `new()` rather than a struct literal — which is why
  `DtcFaultDetectionCounterRecord` gained one (see below); it had no constructor at all, so this
  attribute had made it impossible for a downstream crate to build.
  `ReadDtcInfoSubFunction` is the important one: it has sub-functions the crate does not model
  yet, so adding one post-tag would otherwise have been breaking.

- **Breaking:** `Error::InsufficientData` now carries an `automotive_wire_codec::Incomplete`
  (with `needed` and `available` byte counts) instead of a bare `usize`.

- `automotive-wire-codec` is now a public dependency: its `Incomplete` and `TrailingBytes`
  types are re-exported at the crate root (`uds_protocol::{Incomplete, TrailingBytes}`) and
  are considered part of `uds_protocol`'s public API. A semver-major release of
  `automotive-wire-codec` is therefore a breaking change for `uds_protocol`.

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

### Fixed

- Documentation corrections across the shared format identifiers and DTC record numbers:
  `DataFormatIdentifier` named only `RequestDownloadRequest` (it is also used by
  `RequestUploadRequest` and four `RequestFileTransferRequest` variants) and pointed at a
  `data_format_identifier` *field* that is now private behind an accessor; the same staleness
  affected `MemoryFormatIdentifier` and `LengthFormatIdentifier`. `DtcStoredDataRecordNumber`
  described itself as a `DTCSnapshot` record, and its `new()` had an empty summary line and a
  malformed `Error::ReservedForLegislativeUse` link. `DtcSettingType` was the only type whose doc
  comment sat after its derives. Two redundant intra-doc link targets in
  `communication_control.rs` are gone, so `cargo doc --document-private-items` is now clean.

- **Breaking:** `ReadDtcInfoResponse::decode` now rejects a record list whose length is not a
  whole number of records, with `Error::IncorrectMessageLengthOrInvalidFormat` (NRC `0x13`).
  It previously passed the tail through verbatim, so a malformed frame decoded successfully and
  only failed later, during iteration. This is the same strictness the crate already applies to
  trailing bytes everywhere else. All four record-carrying variants are checked at their own
  width: `DtcList` and `DtcFaultDetectionCounterList` at 4 bytes, `DtcSeverityList` at 6,
  `WwhObdDtcByMaskRecord` at 5. Empty record lists remain valid — a server with no matching DTCs
  answers with the header and no records. Iterators reached from a decoded response therefore
  never see a partial tail; a hand-constructed variant still can, so they keep their `Result`
  item type.

- **All three DTC iterators looped forever on a partial trailing record.** `next()` returned
  `Some(Err(..))` without advancing, so the error was yielded indefinitely: `for` loops and
  `count()` hung, and `collect::<Vec<Result<_, _>>>()` allocated without bound. Reachable from
  untrusted wire input, because `ReadDtcInfoResponse::decode` passes the record tail through
  verbatim without checking that it divides evenly:

  ```rust
  let (resp, _) = Response::decode(&[0x59, 0x02, 0xFF, 0x01, 0x02])?; // 2 leftover bytes
  for r in resp.dtc_and_status_iter().unwrap() { /* never returns */ }
  ```

  `collect_all()` was the only safe path, and only incidentally —
  `collect::<Result<Vec, _>>()` short-circuits on the first error. The fuzz targets missed it
  because they call `decode` and never drive the iterators. Each iterator now consumes the
  partial tail, reporting the error exactly once and terminating.

- **Breaking:** `TesterPresentRequest` no longer rewrites reserved sub-function bytes.
  `[0x3E, 0x01]` decoded and re-encoded as `[0x3E, 0x00]`: the reserved value was parsed and
  then discarded. The normalization was deliberate, but it left the service inconsistent with
  its own response type, which retains the same values, and with every other service
  (`ResetType::IsoSaeReserved`, `DiagnosticSessionType::IsoSaeReserved`, ...). The value is now
  retained in a private field — callers still cannot mint a reserved value, `new(suppress)`
  keeps its signature and its `0x00` encoding — so a server can report
  `subFunctionNotSupported` naming the byte it actually received.

- `TesterPresentResponse::new()` is now `const`. `CommunicationControlRequest::new` and
  `::new_with_node_id` remain non-`const`: both call `u8::from` on the control type in their
  error path, and trait methods are not callable in a `const fn` on stable.

- The README service table was missing rows for `DynamicallyDefineDataIdentifier` (0x2C) and
  `AccessTimingParameters` (0x83), both enumerated in `UdsServiceType`, and named two services
  differently from the code (`ECUReset`, `ControlDTCSetting`). The table's 27 rows now match the
  27 request SIDs in `UdsServiceType` exactly.

- **Breaking:** The `utoipa` and `clap` features now imply `std`. Neither compiled without it:
  their derive macros expand to `std::`, `String` and `Vec` paths inside this crate, so
  `cargo build --no-default-features --features utoipa` failed with 318 resolution errors, as
  did the `clap` equivalent. Only the `--all-features` / `--no-default-features` /
  `--no-default-features --features alloc` combinations were ever built, so the optional
  integrations were never exercised in isolation.

- **Breaking:** `utoipa` additionally implies `serde`. A `ToSchema` in this crate describes the
  *`serde`* representation — several types serialize as a single protocol byte rather than as
  their Rust shape, and their schemas are hand-written to match — so a `utoipa`-without-`serde`
  build published a schema for a wire format it could not produce. Supporting that combination
  also meant every type reachable only through a `serde` repr needed a second `cfg` predicate and
  an `allow(dead_code)` to stay compilable in a configuration no one wants. The feature powerset
  is 16 combinations rather than 20 as a result, all clean.

- The `serde` feature now works on a bare-metal target. The dependency was declared with
  serde's default features on, which pulls `serde/std`, so
  `cargo build --no-default-features --features serde --target thumbv6m-none-eabi` failed even
  though every host-side build passed — a host build proves nothing here, because the host has
  `std` available for serde to compile against regardless of this crate being `#![no_std]`.
  serde is now wired `default-features = false`, picking up its `alloc` and `std` layers
  through weak `serde?/alloc` and `serde?/std` features only when this crate's own `alloc`/`std`
  features are enabled.

- `FunctionalGroupIdentifier::VODBSystem` is now `VobdSystem`. Beyond the casing change, the
  old name transposed the letters: ISO 14229-1 Table **D.15** names functional group `0xFE`
  `VOBDsystem` (vehicle OBD system). Table D.1 is the `groupOfDTC` definition, not the
  functional-group one.

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

### CI

- New `features` job running `cargo hack check --feature-powerset --no-dev-deps` (20
  combinations). The previous matrix only ever built `--all-features`,
  `--no-default-features`, and `--no-default-features --features alloc`, which is why the
  `utoipa`/`clap` breakage went unnoticed.
- The bare-metal job now also builds `serde` and `alloc,serde` for `thumbv6m-none-eabi`, the only
  place the serde `default-features` defect was observable.
- Publication is now gated on `no-std` and `features` in addition to the existing jobs. A tag
  could previously publish a crate whose `no_std` build or feature graph was broken.
