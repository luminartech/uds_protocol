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
- **Bug to fix in implementation (regardless):** `UDSIdentifier::TryFrom<u16>` must round-trip
  every value it can produce — currently `u16::from(UDSIdentifier::UDSVersionData) == 0xFF00`
  but `try_from(0xFF00)` errors, and large valid manufacturer ranges fall through to `Err`.

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
  service consistent with its siblings.
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

## Testing

- Round-trip (`decode` → `encode` → identical bytes) for every reshaped variant: RDBI request
  (native and wire backings), WDBI request, RoutineControl req/resp, `Other` (incl. an
  unknown byte that previously normalized to `0x7F`), RequestTransferExit req/resp with and
  without a parameter record.
- `ReadDataByIdentifierRequest`: build from `&[u16]`, encode, decode, assert `dids()` yields
  the same values; odd-length payload rejected.
- `UDSIdentifier::TryFrom`/`From` round-trip across all variants and ranges (totality fix).
- `assert_encode_size_agrees` on every new/changed `Encode`.
- Full matrix green: default (`std`), `--no-default-features --features alloc`,
  `--no-default-features`, `thumbv6m-none-eabi`; clippy + fmt clean.
- A grep confirming no `…Tx`/`…Rx` type names and no `SUCCESS`/`PENDING` remain.

## Out of scope

- Implementing additional UDS services (still reached via `Other`).
- Any transport, session, or async layer.
- Merging `feature/no_std` to `main` — after this pass and an implementation-details review.
