# UDS Protocol — API Exposure & Consistency Design

**Date:** 2026-06-03
**Branch:** `feature/no_std`
**Status:** Design — pending user review, then implementation plan
**Follows:** `2026-06-01-no-std-api-alignment-design.md` (this completes and tightens that work; it does **not** change tack)

## Purpose

The `no_std` rearchitecture is substantially landed but left two consistency gaps a
reviewer spots immediately: (1) several intended-public types are reachable only
through glob re-exports and some are codec-incomplete, and (2) the `Request` /
`Response` enums model the same class of service inconsistently (some wrap a typed
descriptor, others hand back raw bytes or decomposed fields). We want to ship this
major breaking change as **one cohesive chunk**, so we close these before publishing.

This is split into two phases. **Phase 1** (this doc, ready to implement) deliberately
exposes and completes the descriptor types without restructuring the dispatch enums.
**Phase 2** (framed here, to be planned separately) resolves the naming and
typed-vs-raw symmetry questions across the enums.

## Guiding principle (unchanged, clarified)

Carried from the prior design: **concrete types, no generics, no user-supplied types.**
Clarified here: "lightweight descriptor" means a concrete type that **faithfully models
the ISO-14229 message elements** so a caller can describe a request precisely — it does
**not** mean "minimal" or "raw." Where ISO enumerates the structure (e.g. the ~25
`ReadDTCInformation` sub-functions), the descriptor models it fully. Where the payload
is opaque or caller-defined (routine parameters, write data records, security seeds),
it is carried as a borrowed `&[u8]`.

- **TX side:** lightweight typed descriptors that implement `Encode`.
- **RX side:** borrowed slices / lazy iterators for variable-length response sequences
  (the `ReadDTCInfoResponseRx` model); fixed/small spec structures stay typed.

## Confirmed decisions for Phase 1

- **Deliberate exposure.** Every intended-public type is re-exported **by name** from
  the crate root. Glob re-exports (`pub use services::*;`, `pub use common::*;`) are
  replaced with explicit named lists in `lib.rs`. Rationale: an intentional public
  surface, individually documented types, and globs are how the current orphans hid.
- **Complete the codec-incomplete descriptors.** `ReadDTCInfoRequest` gains a real
  `Encode`; `WriteDataByIdentifierResponse` gains `Decode`. The DTC parameter types
  that `ReadDTCInfoRequest` needs gain `Encode`.
- **No enum restructuring in Phase 1.** `Request` / `Response` variant shapes are left
  exactly as they are; the typed-vs-raw symmetry decision is Phase 2.
- **No speculative code (YAGNI).** Parameter types gain `Encode` only (not `Decode`);
  `Decode` is added later only if Phase 2 decides to type the request RX path.

## Phase 1 — commit series

Each commit builds and tests green on its own across the CI matrix.

1. **Explicit crate-root re-exports.** Replace `pub use services::*;` and
   `pub use common::*;` in `src/lib.rs` with explicit named re-exports. No type changes;
   pure surface tightening. Verify nothing public is dropped or newly hidden.
1. **`Encode` for the 1-byte DTC parameter types** currently lacking it:
   `DTCSeverityMask`, `FunctionalGroupIdentifier`, `DTCSnapshotRecordNumber`,
   `DTCExtDataRecordNumber`, `DTCStoredDataRecordNumber`. (`DTCStatusMask` and
   `DTCRecord` already implement `Encode`.) Each is a small, faithful 1-byte/3-byte
   spec encoder with `assert_encode_size_agrees` coverage.
1. **`Encode` + `encoded_size` for `ReadDTCInfoRequest`** over all 25
   `ReadDTCInfoSubFunction` variants: write the sub-function byte, then encode each
   variant's typed parameters (now that they all implement `Encode`). Add encode tests
   with `assert_encode_size_agrees` for representative variants (no-param,
   single-param, multi-param, `ISOSAEReserved`).
1. **`Decode` for `WriteDataByIdentifierResponse`** (2-byte big-endian `u16`), with a
   round-trip test.
1. **Usage / round-trip tests for the TX builders** so none is dead:
   `ReadDataByIdentifierRequestTx`, `WriteDataByIdentifierRequestTx`,
   `RoutineControlRequestTx`, `RoutineControlResponseTx`. Confirms each is constructible
   and encodes to the expected wire bytes.

### Phase 1 testing

- All new `Encode`/`Decode` impls covered by `assert_encode_size_agrees` and explicit
  wire-byte assertions.
- Full matrix builds and passes: default (`std`),
  `--no-default-features --features alloc`, `--no-default-features`, and
  `thumbv6m-none-eabi`. Clippy clean on all host combos.
- A doc/grep check that the explicit re-export list covers everything the glob did.

## Phase 2 — open questions (to be planned after Phase 1)

Phase 2 is **not** specified here — it is the set of decisions to work through next.
Captured now so they are not lost. Notably, the RX side is already
borrowed-slice/bidirectional, so this is mostly naming and symmetry, not a heavy
re-parse-to-raw migration.

1. **`...ResponseTx` naming (Decision 4 follow-up).** `SecurityAccessResponseTx`,
   `TransferDataResponseTx`, `RequestDownloadResponseTx`, and
   `RequestFileTransferResponseTx` are decoded on the RX path (wrapped in `Response`)
   yet carry the TX-only `...Tx` suffix. They are really *bidirectional borrowed* types,
   a case Decision 4 never named. Ruling needed: rename to `...Rx`, keep `...Tx`, or
   define a bidirectional/no-suffix convention.
1. **Typed-vs-raw RX symmetry (the core fork).** The enums are inconsistent:
   `SecurityAccess` / `TransferData` / `RequestFileTransfer` / `RequestDownload` wrap a
   typed descriptor, while `ReadDataByIdentifier` / `WriteDataByIdentifier` /
   `ReadDTCInfo` / `RoutineControl` hand back raw `&[u8]` or decomposed
   `{ sub_function: u8, raw_payload }`. Decide whether decode should produce the typed
   descriptors (symmetric, round-trippable) or whether the raw holdouts should be
   normalized further toward raw. This is the inconsistency the code review flagged.
1. **Dedup the variable-length integer codec.** The
   `[0u8; N]` + `copy_from_slice` + `from_be_bytes` pattern (and its `to_be_bytes`
   encode twin) is duplicated across `SizePayload`, `FileSizePayload`, `DirSizePayload`
   (`request_file_transfer.rs`) and `RequestDownloadRequest` (`request_download.rs`).
   Extract a shared `read_var_be_uint` / `write_var_be_uint` helper.
1. **Merge the duplicated primitive macros.** `unsigned_primitive_encode_decode!` and
   `signed_primitive_encode_decode!` in `primitive_generics.rs` have byte-for-byte
   identical bodies; collapse to one macro.

## Out of scope

- Implementing additional UDS services (still reached via `Other`).
- Any transport, session, or async layer.
- The Phase 2 decisions themselves — only their framing is recorded here.

## Risks

- **Public-surface drift when de-globbing.** Mitigated by a grep/doc diff confirming the
  explicit list matches the glob's prior exports before committing.
- **`ReadDTCInfoRequest::encode` correctness across 25 variants.** Mitigated by
  per-variant `assert_encode_size_agrees` and wire-byte tests, and by reusing the
  parameter types' own `Encode` impls rather than re-deriving byte layouts.
