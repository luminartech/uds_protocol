# UDS Protocol — Pre-Merge Public API Interrogation Resolutions

**Date:** 2026-06-08
**Branch:** `feature/no_std`
**Status:** Design — decisions approved, pending implementation plan
**Follows:** `2026-06-03-api-consistency-phase2-design.md` (this is the final pre-merge pass:
a structured interrogation of the public API shape before landing the breaking change)

## Purpose

Before merging the `no_std` rearchitecture, we interrogated each structural change to the
public API to confirm a coherent, defensible reason for it. This document records the
resolutions. The unifying theme that emerged: **the dispatch enums should be uniformly
typed and losslessly round-trippable, identifier endianness should never leak to the
caller, and module names should not lie.** Several decisions below tighten the Phase 1/2
work rather than change tack.

## Guiding principle (unchanged)

Concrete types, no generics, no user-supplied types. Simplicity for C developers new to
Rust is a first-class goal. Where ISO structures a message element, model it; where the
payload is opaque/caller-defined, carry it as a borrowed `&[u8]`. The one Rust concept that
cannot be designed away — borrowing — is taught explicitly in the docs.

---

## Decision A (+G) — `ReadDataByIdentifier` request: bidirectional, built-or-parsed

**Problem.** `ReadDataByIdentifierRequestTx<'d>` carried `&[u16]` and implemented only
`Encode`; it was the *only* `…Tx`-suffixed type left in the crate, and it was orphaned —
`Request::ReadDataByIdentifier` held a bare `&[u8]`, so the typed builder could not be used
with the dispatch enum. Building through `Request` forced the caller back to hand-encoding
DIDs big-endian, defeating the `&[u16]` safety it was meant to provide.

**Root constraint.** On a no-alloc target the TX-ergonomic input (`&[u16]` — a list of
16-bit numbers) and the only sound RX output (`&[u8]` — wire bytes cannot be reinterpreted
as `&[u16]` zero-copy) cannot be the same borrowed bytes.

**Resolution.** Replace it with a single bidirectional type whose backing reflects how it
was produced:

```rust
pub struct ReadDataByIdentifierRequest<'a> {
    dids: Dids<'a>, // private
}

// pub(crate) — never named by users
enum Dids<'a> {
    Native(&'a [u16]), // built via ::new(&[u16])
    Wire(&'a [u8]),    // borrowed from the wire on decode (big-endian)
}
```

Public surface stays small and endianness-free: `new(dids: &[u16])`,
`dids(&self) -> impl Iterator<Item = u16>` (the big-endian swap lives only in the `Wire`
arm of this iterator), plus `Encode`/`Decode`. `Decode` rejects odd-length payloads.

**Consequences.**
- The type is bidirectional, so it drops the `…Tx` suffix and composes into
  `Request::ReadDataByIdentifier(ReadDataByIdentifierRequest<'a>)`.
- This removes the last suffixed type in the crate. **The `…Tx`/`…Rx` suffix convention is
  eliminated entirely** (Decision G): no type carries a direction suffix; the `<'a>`
  lifetime already signals borrowing. (`EnableRxAndTx`-style names in `CommunicationControl`
  are unrelated ISO terms and stay.)
- This is the single type with a built-vs-parsed internal backing. That is justified because
  RDBI is the *only* modeled service that builds a **variable-length list** of multi-byte
  identifiers on TX. (Single fixed identifiers and RX-only lists are both strictly easier —
  see Decision B and the DTC iterators.) The `Dids` pattern would be reused if `0x2C
  DynamicallyDefinedDataIdentifier` were ever modeled; not built now (YAGNI).

**`Response::ReadDataByIdentifier(&'a [u8])` stays raw — deliberately.** A read-DID response
is `[DID][data record]…` where data-record lengths are application-defined; the library
cannot know them and therefore cannot extract them. This is documented as an intentional
asymmetry (structured request, opaque response), not a leftover inconsistency.

## Decision B — Typed identifiers on `[identifier][opaque tail]` services

**Problem.** Every `[16-bit identifier][opaque tail]` service buried the identifier as the
first two big-endian bytes of an opaque `&[u8]` on the request side — the same endianness
footgun as RDBI — and did so inconsistently with its own response:
`WriteDataByIdentifierRequest { payload: &[u8] }` vs `WriteDataByIdentifierResponse
{ identifier: u16 }`; `RoutineControlRequest { sub_function, raw_payload: &[u8] }` with the
RID buried in `raw_payload`.

**Resolution.** Pull the identifier out as a typed `u16`; keep the genuinely-opaque tail as
`&[u8]`:

```rust
WriteDataByIdentifierRequest { identifier: u16, data: &'a [u8] }
RoutineControlRequest        { sub_function, routine_id: u16, option_record: &'a [u8] }
RoutineControlResponse       { routine_control_type, routine_id: u16, status_record: &'a [u8] }
```

All bidirectional and zero-copy (decode reads the two big-endian bytes into the scalar; the
swap is hidden there). No two-way backing is needed — these are single fixed identifiers,
not lists.

**Decisions within B:**
- **Raw `u16`, not the identifier enums.** DIDs/RIDs are overwhelmingly manufacturer-defined;
  forcing `UDSIdentifier`/`UDSRoutineIdentifier` would make the common case awkward and decode
  lossy. A caller who wants a name calls `TryFrom`.
- **Stricter decode (accepted).** Typing the identifier means decode now requires the
  mandatory identifier bytes and errors if the payload is too short. ISO mandates them; this
  matches the reserved-value rejection adopted in Phase 2.
- **Private fields + getters + `#[non_exhaustive]`** across these descriptors, for a uniform
  public surface (WDBI request previously exposed a `pub` field; standardize).

## Decision B-followup — Identifier enums become catalogs, not codecs

`UDSIdentifier` / `UDSRoutineIdentifier` are nothing's carried type anymore (nothing in the
crate uses them except re-exports). Their role narrows to a **named catalog of spec-defined
IDs + `u16` converters** (`From`/`TryFrom<u16>`, `Display`/`Debug`).

- **Remove their standalone `Encode`/`Decode` impls.** The descriptors own the 2-byte
  big-endian codec (raw `u16`, total — accepts any DID). Keeping enum-level codecs creates a
  second wire path (the same smell removed in A) and, worse, `UDSIdentifier` cannot serve as
  a decoder: its `TryFrom<u16>` is **non-total** (rejects most of the legal DID space and
  omits its own `0xFF00`/`0xFF01` variants). The enum is a partial *recognizer*, not a codec.
- **Resolved (full faithful rebuild — confirmed):** `UDSIdentifier` becomes a **total,
  infallible `From<u16>`** mirroring `UDSRoutineIdentifier`; `TryFrom` and the now-dead
  `Error::InvalidDiagnosticIdentifier` are removed. The current enum has two defects: it
  cannot round-trip its own `0xFF00`/`0xFF01` variants, and it **mislabels** `0xF100–0xF17F`
  as `VehicleManufacturerSpecific` (per spec that range is
  `identificationOptionVehicleManufacturerSpecific`; the real VMS range is `0x0100–0xA5FF`
  plus others) while omitting ~13 ISO DID classes entirely. The rebuild follows the
  authoritative partition in the Appendix, verified against ISO 14229-1:2020 Table C.1.
  Nothing carries `UDSIdentifier` (Decision B), so this has zero internal ripple.
- **`UDSRoutineIdentifier` verified faithful & total** against ISO 14229-1:2020 Table F.1 —
  no change required (only a cosmetic singular/plural naming nit on `SafetySystemRoutineID`).

## Decision C — `Other` is a lossless raw pass-through

**Problem.** `Other { service: UdsServiceType, data }` reconstructed the wire byte *from the
typed service*, so an unrecognized byte decoded to `UnsupportedDiagnosticService` and
re-encoded as `0x7F` — lossy. On the response side an unknown byte re-encoded as `0x7F`,
which re-decodes as `NegativeResponse`: a semantic change, not just loss. This defeats the
stated purpose of `Other` (transport pass-through).

**Resolution.** Store the raw byte and echo it verbatim:

```rust
Request::Other  { sid: u8, data: &'a [u8] }
Response::Other { sid: u8, data: &'a [u8] }
```

Encode writes `sid` directly — lossless by construction, and naturally handles the
request-vs-response SID difference. The typed view is recovered on demand via `service()`
(Decision F). `UdsServiceType` stays a clean fieldless lookup enum (rejected the
alternative of `UnsupportedDiagnosticService(u8)` — too broad a blast radius for an enum used
everywhere as a key). The doc promise becomes unconditional: re-encoding is lossless for
every service byte.

## Decision D — Remove free-floating raw protocol constants

- **Delete public `PENDING` (`0x78`).** Exact duplicate of
  `NegativeResponseCode::RequestCorrectlyReceivedResponsePending`, and used nowhere.
- **Delete public `SUCCESS` (`0x80`).** Misnamed and misdocumented (it is the SPRMIB
  suppress-positive-response bit, not a "response-SID offset"). Its only user is
  `ControlDTCSettings`, which hand-rolls the suppress bit inline. **Migrate
  `ControlDTCSettingsRequest` to `SuppressablePositiveResponse<DtcSettings>`** like its six
  sibling sub-function services; `DtcSettings` already satisfies the wrapper's bounds. This
  removes the only user of `SUCCESS`, collapses the `0x80` bit into one place, and makes the
  service consistent with its siblings. Wire-neutral (the wrapper's `& 0x7F` / `| 0x80` is
  byte-identical to the current inline `& !SUCCESS` / `| SUCCESS`). Intrinsic surface changes:
  `pub setting` / `pub suppress_response` become a private wrapper exposed via `setting()` +
  `suppress_positive_response()` getters (EcuReset template). Aligned changes: **flip the
  constructor to suppress-first** — `new(suppress_positive_response: bool, setting: DtcSettings)`
  — to match the six sibling sub-function services (was `(setting, suppress_response)`), and add
  the missing `Eq` derive. Response (`ControlDTCSettingsResponse`) is unchanged (no SPRMIB bit).
- **Keep `CLEAR_ALL_DTCS`** — a correctly-typed, meaningful, used `DTCRecord` constant.

Net: no raw protocol-byte constants at the crate root; `CLEAR_ALL_DTCS` is the only public
constant, and it is typed.

## Decision E — Module structure tells the truth

`common/` mixed three kinds of types. Reorganize by ownership:

- **Single-consumer enums move to their service module:** `ResetType` → `ecu_reset`,
  `DiagnosticSessionType` → `diagnostic_session_control`, `SecurityAccessType` →
  `security_access`, `CommunicationControlType` + `CommunicationType` →
  `communication_control`, `DtcSettings` → `control_dtc_settings`, `RoutineControlSubFunction`
  → `routine_control`. (The last two currently live in `lib.rs`.)
- **DTC vocabulary becomes its own `dtc/` domain module:** `DTCRecord`, `DTCStatusMask`,
  `DTCSeverityMask`, `DTCSeverityRecord`, `DTCFormatIdentifier`, `DTCSnapshotRecordNumber`,
  `DTCStoredDataRecordNumber`, `DTCExtDataRecordNumber`, `FunctionalGroupIdentifier`,
  `CLEAR_ALL_DTCS`. The read-DTC iterators stay in `read_dtc_information.rs` (single owner).
- **`common/` is renamed `shared/`** and keeps only the genuinely protocol-wide types:
  `NegativeResponseCode`, `SuppressablePositiveResponse`, `UDSIdentifier`/
  `UDSRoutineIdentifier`, the primitive codec, `util`, and the `pub(crate)` format
  identifiers.

**Zero public-surface change:** the flat crate-root re-exports are preserved, so every
`uds_protocol::Foo` path resolves identically. This is an internal-only reorganization;
CI catches any missed import.

**Sequencing (move-first).** The reorg lands **before** the semantic reshaping (Decisions
A/B/C/H + I1–I3), as ~3 mechanical, individually-green commits — (1) `common/ → shared/`
rename, (2) extract `dtc/`, (3) relocate the six single-service enums — so the heavily-reviewed
semantic work is written once, in the final structure. Firm rule: **no commit mixes a
mechanical move with a semantic change** (keeps every diff legible; git rename-detection makes
the move commits trivially verifiable). Two verifications during the move: (a) the dependency
graph stays acyclic — `services/ → {dtc/, shared/}`, `dtc/ → shared/`, nothing above pointing
back down (a DTC type reaching into a service module is the red flag); (b) a grep/diff confirms
the crate-root public-name set is byte-identical before and after.

## Decision F — `Response::service()` for parity with `Request`

Add `Response::service() -> UdsServiceType` mirroring `Request::service()`; keep
`response_sid()` private (encode helper). Per variant it returns the obvious service. Two
clarified cases:

- **`NegativeResponse` → `UdsServiceType::NegativeResponse`** (the frame's own identity). The
  *failed* service is data inside the frame, already exposed as `NegativeResponse.request_service`.
- **`Other { sid, .. }` → `UdsServiceType::response_from_byte(sid)`** (derived typed view; the
  raw `sid` remains the lossless storage).

**Accepted documented edge:** `NegativeResponse::decode`/`encode` normalize an *unmodeled*
echoed service byte to `0x7F`. `request_service` stays a typed `UdsServiceType` for the
ergonomic win on every normal NACK; the lossy corner (an unmodeled failed service) is
documented rather than fixed, because it is far less consequential than the service being
unsupported in the first place.

## Decision H — `RequestTransferExit` carries its optional parameter record

`Request`/`Response::RequestTransferExit` were unit variants that silently discarded any
payload (and bypassed the trailing-byte rejection every other variant enforces) — lossy for
the ISO-defined optional `transferRequest/ResponseParameterRecord`. Give them thin
descriptors, consistent with Decision A's removal of bare-slice variants:

```rust
RequestTransferExitRequest<'a>  { parameter_record: &'a [u8] }
RequestTransferExitResponse<'a> { parameter_record: &'a [u8] }
```

Wrapped in the enums and round-tripped losslessly (empty slice when the record is absent).

---

## Components touched

- `src/lib.rs` — drop `SUCCESS`/`PENDING`; move `RoutineControlSubFunction`/`DtcSettings`
  out; update re-export lists for renames and the `shared/`/`dtc/` split; keep flat paths.
- `src/services/read_data_by_identifier.rs` — `ReadDataByIdentifierRequest<'a>` with private
  `Dids` backing, `new(&[u16])`, `dids()`, `Encode`/`Decode`.
- `src/services/write_data_by_identifier.rs`, `routine_control.rs` — typed `u16` identifier +
  opaque tail; getters; `#[non_exhaustive]`.
- `src/services/control_dtc_settings.rs` — adopt `SuppressablePositiveResponse<DtcSettings>`.
- `src/services/transfer_data.rs` (or a new `request_transfer_exit.rs`) — the two
  `RequestTransferExit*` descriptors.
- `src/request.rs` / `src/response.rs` — `Other { sid, data }`; wrap RDBI/RTE descriptors;
  add `Response::service()`; update all match arms.
- `src/{shared}/diagnostic_identifier.rs` — remove enum `Encode`/`Decode`; fix `TryFrom`
  totality.
- `src/services/negative_response.rs` — document the echoed-service edge.
- Module moves per Decision E (`common/` → `shared/`, new `dtc/`, service-local enums).

## Decode edge contracts (locked)

Unifying principle: **require the structurally-mandatory fixed elements; allow the opaque
variable tail to be empty.**

- **RDBI request** (`Dids::Wire`): **reject empty** (ISO requires ≥1 DID — its entire content
  is the mandatory list, no fixed prefix to anchor an empty case) and **reject odd-length**
  (`IncorrectMessageLengthOrInvalidFormat`). `Native(&[u16])` and its decoded `Wire(&[u8])`
  must encode to identical bytes and yield identical `dids()`.
- **WDBI request** `{identifier, data}`: min 2 bytes (DID mandatory); **empty `data` allowed**
  (a zero-length data record is app semantics the server NACKs, not a codec concern).
- **RoutineControl request** `{sub_function, routine_id, option_record}`: min 3 bytes
  (sub-function + RID); empty `option_record` allowed; reserved control-type rejected (Phase 2);
  SPRMIB bit round-trips via the wrapper.
- **RoutineControl response** `{routine_control_type, routine_id, status_record}`: min 3 bytes;
  empty `status_record` allowed; control-type decoded via **plain `try_from`** (no `0x80`
  masking) so a stray SPRMIB bit on a response is rejected — responses never suppress.
- **Other** `{sid, data}`: any non-empty frame; empty `data` allowed; re-encode lossless.
- **RequestTransferExit** req/resp `{parameter_record}`: record optional (empty allowed);
  lossless round-trip; fixes the prior silent-drop / trailing-byte acceptance.

## Testing

- Round-trip (`decode` → `encode` → identical bytes) for every reshaped variant: RDBI request
  (native and wire backings), WDBI request, RoutineControl req/resp, `Other` (incl. an
  unknown byte that previously normalized to `0x7F`), RequestTransferExit req/resp with and
  without a parameter record.
- Negative decode tests for every edge above: RDBI empty + odd-length; WDBI < 2 bytes;
  RoutineControl req/resp < 3 bytes; RoutineControl response with `0x80` set; reserved
  control-type.
- `ReadDataByIdentifierRequest`: build from `&[u16]`, encode, decode, assert `dids()` yields
  the same values.
- `UDSIdentifier::TryFrom`/`From` round-trip across all variants and ranges (totality fix).
- **Strengthen `assert_encode_size_agrees`** to also assert actual bytes consumed
  (`buf.len() - writer.len() == encoded_size()`), not just `encode`'s return value — today a
  write/return drift slips through. Apply it on every new/changed `Encode`.
- Full matrix green: default (`std`), `--no-default-features --features alloc`,
  `--no-default-features`, `thumbv6m-none-eabi`; clippy + fmt clean.
- A grep confirming no `…Tx`/`…Rx` type names and no `SUCCESS`/`PENDING` remain.

## Out of scope

- Implementing additional UDS services (still reached via `Other`).
- Any transport, session, or async layer.
- Merging `feature/no_std` to `main` — after this pass and an implementation-details review.

---

## Appendix — Authoritative DID partition (ISO 14229-1:2020 Table C.1)

The total, infallible `UDSIdentifier::From<u16>` rebuild (Decision B-followup) maps every
`u16` per this table. Open ranges carry the raw `u16`; named values are unit variants.
`0xF180–0xF19F` are the ~32 named singletons already modeled.

| Range | Class |
|-------|-------|
| `0x0000–0x00FF` | ISOSAEReserved |
| `0x0100–0xA5FF` | VehicleManufacturerSpecific |
| `0xA600–0xA7FF` | ReservedForLegislativeUse |
| `0xA800–0xACFF` | VehicleManufacturerSpecific |
| `0xAD00–0xAFFF` | ReservedForLegislativeUse |
| `0xB000–0xB1FF` | VehicleManufacturerSpecific |
| `0xB200–0xBFFF` | ReservedForLegislativeUse |
| `0xC000–0xC2FF` | VehicleManufacturerSpecific |
| `0xC300–0xCEFF` | ReservedForLegislativeUse |
| `0xCF00–0xEFFF` | VehicleManufacturerSpecific |
| `0xF000–0xF00F` | NetworkConfigDataForTractorTrailerApplication |
| `0xF010–0xF0FF` | VehicleManufacturerSpecific |
| `0xF100–0xF17F` | identificationOptionVehicleManufacturerSpecific |
| `0xF180–0xF19F` | named singletons (existing) |
| `0xF1A0–0xF1EF` | identificationOptionVehicleManufacturerSpecific |
| `0xF1F0–0xF1FF` | identificationOptionSystemSupplierSpecific |
| `0xF200–0xF2FF` | PeriodicDataIdentifier |
| `0xF300–0xF3FF` | DynamicallyDefinedDataIdentifier |
| `0xF400–0xF5FF` | OBDDataIdentifier |
| `0xF600–0xF6FF` | OBDMonitorDataIdentifier |
| `0xF700–0xF7FF` | OBDDataIdentifier |
| `0xF800–0xF8FF` | OBDInfoTypeDataIdentifier |
| `0xF900–0xF9FF` | TachographDataIdentifier |
| `0xFA00–0xFA0F` | AirbagDeploymentDataIdentifier |
| `0xFA10` | NumberOfEDRDevices |
| `0xFA11` | EDRIdentification |
| `0xFA12` | EDRDeviceAddressInformation |
| `0xFA13–0xFA18` | EDREntries |
| `0xFA19–0xFAFF` | SafetySystemDataIdentifier |
| `0xFB00–0xFCFF` | ReservedForLegislativeUse |
| `0xFD00–0xFEFF` | SystemSupplierSpecific |
| `0xFF00` | UDSVersionData |
| `0xFF01` | ReservedForISO15765-5 |
| `0xFF02–0xFFFF` | ISOSAEReserved |

`UDSRoutineIdentifier` was verified against Table F.1 and is already faithful/total
(`0x0000–0x00FF` ISOSAEReserved · `0x0100–0x01FF` TachographTestIds · `0x0200–0xDFFF` VMS ·
`0xE000–0xE1FF` OBDTestIds · `0xE200` ExecuteSPL · `0xE201` DeployLoopRoutineID ·
`0xE202–0xE2FF` SafetySystemRoutineIDs · `0xE300–0xEFFF` ISOSAEReserved · `0xF000–0xFEFF`
SystemSupplierSpecific · `0xFF00` eraseMemory · `0xFF01` checkProgrammingDependencies ·
`0xFF02–0xFFFF` ISOSAEReserved).
