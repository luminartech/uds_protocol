# UDS Protocol — API Consistency Phase 2 Design

**Date:** 2026-06-03
**Branch:** `feature/no_std`
**Status:** Design — pending user review, then implementation plan
**Follows:** `2026-06-03-api-exposure-and-consistency-design.md` (Phase 1, landed). This resolves
the Phase 2 open questions recorded there.

## Purpose

Phase 1 made the public surface deliberate and completed the codec-incomplete descriptors.
Phase 2 finishes the cohesive breaking change by removing the remaining inconsistencies a
reviewer sees immediately:

1. The `…Tx`/`…Rx` suffixes mislead — most "Tx" descriptors are actually bidirectional
   (they implement both `Encode` and `Decode` and appear in both the `Request` and
   `Response` enums).
2. The dispatch enums model the same class of service two different ways — some variants wrap
   a named descriptor type, others hold a bare `&[u8]` or inline fields.
3. The variable-length big-endian integer codec is hand-rolled in ~5 places.
4. Two byte-identical primitive macros exist where one would do.

All of this lands on `feature/no_std` so the entire breaking change ships together.

## Guiding principle (unchanged)

Concrete types, no generics, no user-supplied types. Lightweight typed descriptors that
faithfully model the ISO-14229 message elements on TX; borrowed slices / lazy iterators for
variable-length RX sequences. Simplicity for C developers new to Rust is a first-class goal —
which is why the naming rule below removes the `Tx`/`Rx` jargon everywhere it is not
load-bearing.

---

## Decision 1 — Suffix marks asymmetry, not direction

**Rule:** a `…Tx` or `…Rx` suffix is used **only** when a service's TX and RX representations
are genuinely different types. A type that is bidirectional (implements both `Encode` and
`Decode`, and is used on both the request-building and response-parsing paths) takes **no
suffix** and reads as plain `FooRequest` / `FooResponse`. The `<'a>` lifetime already signals
that a type borrows from the wire buffer.

**Renames (all verified bidirectional — they implement both `Encode` and `Decode`):**

| Current | New |
|---|---|
| `SecurityAccessRequestTx` | `SecurityAccessRequest` |
| `SecurityAccessResponseTx` | `SecurityAccessResponse` |
| `TransferDataRequestTx` | `TransferDataRequest` |
| `TransferDataResponseTx` | `TransferDataResponse` |
| `RequestFileTransferRequestTx` | `RequestFileTransferRequest` |
| `RequestFileTransferResponseTx` | `RequestFileTransferResponse` |
| `RequestDownloadResponseTx` | `RequestDownloadResponse` |
| `RoutineControlRequestTx` | `RoutineControlRequest` |
| `RoutineControlResponseTx` | `RoutineControlResponse` |
| `WriteDataByIdentifierRequestTx` | `WriteDataByIdentifierRequest` |
| `ReadDTCInfoResponseRx` | `ReadDTCInfoResponse` |
| `NamePayloadTx` | `NamePayload` |
| `SentDataPayloadTx` | `SentDataPayload` |

**Keeps its suffix — the single genuine asymmetry:** `ReadDataByIdentifierRequestTx`
(`&[u16]` DID list on TX; the wire cannot be reinterpreted as `&[u16]` zero-copy, so RX is
raw `&[u8]`).

Already correctly unsuffixed and unchanged: `RequestDownloadRequest`,
`WriteDataByIdentifierResponse`, `ReadDTCInfoRequest`, `PositionPayload`, `SizePayload`,
`FileSizePayload`, `DirSizePayload`, `FileOperationMode`, and all fixed-size service types.

All renames must update the explicit crate-root re-export lists in `src/lib.rs` (from Phase 1)
and every doc-comment reference.

## Decision 2 — Wrap descriptors in the enums where feasible

Each modeled `Request`/`Response` variant wraps a single named descriptor type wherever a
zero-copy decode allows it, replacing bare `&[u8]` and inline-field variants. This eliminates
the orphaned descriptor types (the enums now construct them) and makes the variants
round-trippable.

| Variant | Before | After | Decode work |
|---|---|---|---|
| `Request::WriteDataByIdentifier` | `(&[u8])` | `(WriteDataByIdentifierRequest)` | add trivial `Decode` (borrow whole payload) |
| `Request::ReadDTCInfo` | `(&[u8])` | `(ReadDTCInfoRequest)` | add **25-variant `Decode`** (inverse of Phase 1 `Encode`) |
| `Request::RoutineControl` | `{ sub_function: u8, raw_payload: &[u8] }` | `(RoutineControlRequest)` | add `Decode` via `SuppressablePositiveResponse<RoutineControlSubFunction>` + borrowed payload (see below) |
| `Response::WriteDataByIdentifier` | `(&[u8])` | `(WriteDataByIdentifierResponse)` | `Decode` already added in Phase 1 |
| `Response::RoutineControl` | `{ routine_control_type: u8, raw_status_record: &[u8] }` | `(RoutineControlResponse)` | add `Decode` |

**RDBI is the documented exception.** `Request::ReadDataByIdentifier(&[u8])` and
`Response::ReadDataByIdentifier(&[u8])` stay raw, because the DID list cannot be produced as
`&[u16]` zero-copy. `ReadDataByIdentifierRequestTx` remains a TX-only build/encode helper.

The `Request`/`Response` `Encode`/`encoded_size`/`service`/`response_sid` match arms and
`is_positive_response_suppressed` are updated for the new variant shapes.

### Sub-function byte fidelity (resolved)

The wire sub-function byte for RoutineControl carries the SPRMIB suppress bit (`0x80`) in its
high bit and the `routineControlType` in the low 7 bits. RoutineControl is modeled exactly
like every other sub-function service (`EcuResetRequest` is the template), using the existing
internal `SuppressablePositiveResponse<T>` wrapper:

```rust
pub struct RoutineControlRequest<'d> {
    sub_function: SuppressablePositiveResponse<RoutineControlSubFunction>,
    raw_payload: &'d [u8],
}
```

`RoutineControlSubFunction` already satisfies the wrapper's bounds (`TryFrom<u8, Error = Error>`
+ `From<…> for u8`). On decode, `SuppressablePositiveResponse::try_from(payload[0])?` splits the
byte into `(suppress_flag, RoutineControlSubFunction::try_from(byte & 0x7F))`; on encode,
`u8::from(self.sub_function)` re-applies the bit — so the suppress bit round-trips losslessly.
Following the `EcuResetRequest` pattern, the `SuppressablePositiveResponse` field is private
(the type stays `pub(crate)`); the public surface is
`new(suppress_positive_response: bool, sub_function: RoutineControlSubFunction, raw_payload: &'d [u8])`
plus `suppress_positive_response()`, `sub_function()`, and `raw_payload()` getters.

This also lets `Request::is_positive_response_suppressed()` forward to RoutineControl (today it
falls through to `false`).

`RoutineControlResponse.routine_control_type` is a plain `RoutineControlSubFunction` (responses
never carry the SPRMIB bit — a suppressed request produces no positive response), mirroring how
`EcuResetResponse` holds a plain `ResetType`.

**One intentional behavior change:** the typed form rejects reserved `routineControlType` values
(low 7 bits outside 0x01–0x03) on decode with `IncorrectMessageLengthOrInvalidFormat`, where the
old raw-`u8` variant accepted any byte. ISO defines only 0x01–0x03, and every sibling
sub-function service already rejects unknown values, so this makes RoutineControl consistent.
A round-trip test with the suppress bit set (e.g. `0x81`) locks the fidelity in.

## Decision 3 — Deduplicate the variable-length big-endian integer codec

The pattern
`let mut b = [0u8; N]; b[N-n..].copy_from_slice(&src[..n]); T::from_be_bytes(b)` (and its
`to_be_bytes()[N-n..]` encode twin) is duplicated across `SizePayload`, `FileSizePayload`,
`DirSizePayload` (`request_file_transfer.rs`) and `RequestDownloadRequest`
(`request_download.rs`).

**Action:** add one concrete, non-generic helper pair to `src/common/util.rs`:

- `read_be_uint(src: &[u8], n: usize) -> Result<u128, Error>` — left-pads `n` big-endian
  bytes into a `u128`; returns `Error::InsufficientData(n)` when `src.len() < n`, and rejects
  `n > 16`.
- `write_be_uint(value: u128, n: usize, writer: &mut impl embedded_io::Write) -> Result<usize, Error>`
  — writes the low `n` big-endian bytes of `value`.

`u128` is the widest case, so the `u64`/`u32` call sites (`RequestDownloadRequest`) cast down
from the returned `u128` (`as u64` / `as u32`). No generics — one concrete pair serves every
site. Whether these helpers are part of the public API or `pub(crate)` is settled in the plan
(default: `pub(crate)`, since they are an internal codec detail).

## Decision 4 — Merge the duplicate primitive macros

`unsigned_primitive_encode_decode!` and `signed_primitive_encode_decode!` in
`src/common/primitive_generics.rs` have byte-identical bodies (both use `to_be_bytes` /
`from_be_bytes`). Merge into a single `primitive_encode_decode!` macro invoked once with all
ten integer types (`u8, u16, u32, u64, u128, i8, i16, i32, i64, i128`).

---

## Components touched

- `src/lib.rs` — update explicit re-export lists for renamed types.
- `src/services/security_access.rs`, `transfer_data.rs`, `request_file_transfer.rs`,
  `request_download.rs`, `routine_control.rs`, `write_data_by_identifier.rs`,
  `read_dtc_information.rs` — type renames; add `Decode` impls where Decision 2 requires;
  call the new `read_be_uint`/`write_be_uint` helpers.
- `src/request.rs` / `src/response.rs` — rewire the four+ variants to wrap descriptors; update
  all match arms; rename referenced types.
- `src/common/util.rs` — add `read_be_uint` / `write_be_uint`.
- `src/common/primitive_generics.rs` — merge the two macros.

## Testing

- Round-trip (`Request`/`Response` `decode` → `encode` → bytes identical) for every
  newly-wrapped variant: `WriteDataByIdentifier` (req & resp), `ReadDTCInfo` (req),
  `RoutineControl` (req & resp).
- A `RoutineControl` round-trip with the SPRMIB suppress bit set, asserting the byte survives.
- `assert_encode_size_agrees` on every new/changed `Encode` impl.
- Unit tests for `read_be_uint`/`write_be_uint` (including `n=0`, max width, and
  `InsufficientData`), and confirmation the refactored payload decoders still pass their
  existing round-trip tests.
- Full CI matrix green: default (`std`), `--no-default-features --features alloc`,
  `--no-default-features`, `thumbv6m-none-eabi`; clippy clean on all host combos; `fmt` clean.

## Out of scope

- Implementing additional UDS services (still reached via `Other`).
- Any transport, session, or async layer.
- Merging `feature/no_std` to `main` — happens after Phase 2 lands, as the full cohesive
  breaking change.

## Risks

- **Broad rename churn.** 13 renamed types ripple across services, the enums, re-exports,
  tests, and docs. Mitigated by no external in-tree consumers and a green CI matrix; a final
  grep for the old names confirms none remain.
- **`ReadDTCInfoRequest::Decode` correctness across 25 variants.** Mitigated by per-variant
  round-trip tests reusing the Phase 1 `Encode` as the oracle (encode a known value, decode it
  back, assert equality).
- **Routine-control sub-function byte regression.** Explicitly guarded by the SPRMIB round-trip
  test (Decision 2).
