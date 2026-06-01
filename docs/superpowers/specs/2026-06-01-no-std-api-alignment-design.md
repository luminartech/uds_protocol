# UDS Protocol — no_std API Alignment Design

**Date:** 2026-06-01
**Branch:** `feature/no_std`
**Status:** Approved scope, pending implementation plan

## Purpose

Before landing the `no_std` rearchitecture, align the public API with the crate's
revised scope so that breaking changes happen once, not in a series of post-publish
follow-ups. The scope changed from "desktop diagnostic tooling" to **a pure,
synchronous, runtime-agnostic UDS codec usable on `no_std` + `no_alloc` targets.**

## Confirmed scope (decisions)

- **Pure codec.** The crate encodes/decodes UDS messages only. Transport (DoIP,
  UDSonIP, sync or async) lives in downstream crates that depend on this one.
- **Fully synchronous.** No async anywhere. `Encode` writes into any
  `embedded_io::Write`; `Decode` borrows from a `&[u8]`. Callers — sync or async —
  own the I/O loop. The crate never owns a runtime, a socket, or a buffer it
  allocates.
- **`no_std` + `no_alloc` baseline.** `alloc` and `std` are strictly additive
  features. This is already true structurally and must stay true.

## Guiding principle — simplicity for C developers new to Rust

The growing user base is competent C developers who are new to Rust. **Simplicity is
a first-class acceptance criterion, not a nice-to-have:** prefer concrete types over
generics, obvious over clever, fewer types over more. This directly motivates
Decisions 1–3 (generics and trait bounds are the steepest part of Rust's learning
curve) and Decision 5 (fewer types). The one Rust concept that cannot be designed
away on a `no_alloc` target is **borrowing** — decoded values point into the caller's
buffer — so the documentation must teach that model explicitly in C-familiar terms
(see Decision 9).

## What is NOT changing

- The `Encode` (stream) / `Decode` (borrow-from-slice) split — this is the correct
  shape for `no_alloc` and is kept.
- The concrete fixed-size service types (`EcuResetRequest`, `TesterPresentRequest`,
  the `*Response` fixed types, `NegativeResponse`, etc.).
- The `UDSIdentifier` / `UDSRoutineIdentifier` enums (kept; see Decision 2).
- The `ReadDTCInfoResponseRx` "typed enum holding raw bytes + lazy iterators"
  pattern — this is the model the rest of the RX surface should resemble.
- The existing `no_std` / `alloc` / bare-metal CI matrix.

---

## Decisions

### Decision 1 — Remove the orphaned generic-typing abstraction

`DiagnosticDefinition` is exported with a worked example (`UdsSpec`) but **nothing
consumes it** — the only references are its own definition and the `UdsSpec` impl.
The decode path already produces raw `&[u8]` for the identifier-driven services. The
original highly-genericized design was found to be harder to understand than simply
carrying payloads, and the abstraction did not survive the `no_std` pass.

**Action:** Delete `DiagnosticDefinition` and `UdsSpec`. Commit fully to the
payload-carrying model: dispatch enums are non-generic and hand back raw payload
bytes; callers interpret those bytes themselves.

### Decision 2 — Strip all identifier machinery

With `DiagnosticDefinition` gone, the remaining generics exist only to support
pluggable, user-defined identifier types — which the payload-carrying model no longer
needs.

**Remove:**
- The `Identifier` and `RoutineIdentifier` traits.
- The `impl_identifier!` macro.
- The blanket `Encode` / `Decode` / `DecodeIter` impls for `T: Identifier`
  (`src/traits.rs`).
- `ProtocolIdentifier`, `ProtocolPayloadTx`, `ProtocolRoutinePayloadTx`
  (`src/protocol_definitions.rs` — the module is deleted).

**Keep:**
- `UDSIdentifier` and `UDSRoutineIdentifier` enums, with their existing
  `TryFrom<u16>` / `From<UDSIdentifier> for u16` conversions.
- Because these enums currently obtain `Encode`/`Decode` via the blanket impl, give
  each a **direct, concrete** `Encode` + `Decode` impl (2-byte big-endian) so users
  can still serialize/parse a known identifier when they want to. This is the only
  identifier-level codec that survives.

**Net effect:** the crate exposes raw payload bytes plus the canonical
`UDSIdentifier` / `UDSRoutineIdentifier` enums. No generic identifier plumbing.

### Decision 3 — De-genericize the service builders to match RX

The TX builders for the identifier services are the last place generics survive.
De-genericize them so TX mirrors the raw-bytes RX side.

| Service | Current (TX) | New (TX) carries |
|---|---|---|
| ReadDataByIdentifier | `ReadDataByIdentifierRequestTx<'d, DID>` | `&'d [u16]` DID list (BE-encoded on write) |
| WriteDataByIdentifier | `WriteDataByIdentifierRequest<Payload>` | `&'d [u8]` raw (DID + data) |
| WriteDataByIdentifier (resp) | `WriteDataByIdentifierResponse<DID>` | `u16` echoed identifier (fixed, 2 bytes) |
| RoutineControl (req) | `RoutineControlRequest<RI, RP>` | `sub_function` + `&'d [u8]` raw (RID + data) |
| RoutineControl (resp) | `RoutineControlResponse<RSR>` | `routine_control_type` + `&'d [u8]` raw status record |

Rationale for `&[u16]` on the Read-DID request: it is a list of identifiers, and a
`u16` slice avoids an endianness footgun while staying alloc-free and generic-free. A
DID is conceptually a 16-bit number, so `&[u16]` reads more clearly to a C developer
than a byte-pair-encoded `&[u8]` would. All other payloads are opaque bytes and carry
`&[u8]`, exactly matching what the RX enum variants already hold. **Resolved:** keep
`&[u16]` for Read-DID (was previously an open risk).

### Decision 4 — Apply the Tx/Rx naming convention strictly

**Rule:** fixed-size bidirectional types take **no suffix**; zero-copy borrowed TX
types take **`...Tx`**; zero-copy borrowed RX types take **`...Rx`**.

Renames required to make the existing (correct) intent consistent:

| Current name | New name | Reason |
|---|---|---|
| `WriteDataByIdentifierRequest` | `WriteDataByIdentifierRequestTx` | now carries borrowed bytes |
| `RoutineControlRequest` | `RoutineControlRequestTx` | now carries borrowed bytes |
| `RoutineControlResponse` | `RoutineControlResponseTx` | now carries borrowed bytes |

Types already conforming and unchanged: `ReadDataByIdentifierRequestTx`,
`TransferDataRequestTx`, `SecurityAccessRequestTx`, `RequestFileTransferRequestTx`,
`RequestDownloadResponseTx`, `SecurityAccessResponseTx`, `TransferDataResponseTx`,
`RequestFileTransferResponseTx`, `ReadDTCInfoResponseRx`, and all fixed-size
no-suffix types. `WriteDataByIdentifierResponse` stays unsuffixed (fixed `u16`).

### Decision 5 — Unify the raw passthrough / unknown-service escape hatch

Today the escape hatch is asymmetric: `Response` has both a raw `UdsResponse<'a>`
view and raw-slice variants, while unmodeled services on the `Request` side hard-error
with `ServiceNotImplemented`.

**Action:** Add symmetric `Other { service: UdsServiceType, data: &'a [u8] }` variants
to **both** `Request<'a>` and `Response<'a>`. A frame for a known-but-unmodeled service
decodes into `Other` rather than erroring, so downstream transport code can pass it
through. Remove the standalone `UdsResponse<'a>` type — `Response::Other` subsumes its
"don't parse, just hand me service + bytes" role. `ServiceNotImplemented` is retained
in `Error` only for genuinely unrecognized service bytes (if any remain) and may be
removed if `Other` covers all cases — to be settled in the implementation plan.

### Decision 6 — Move `is_positive_response_suppressed` off `Encode`

This is UDS protocol semantics (SPRMIB) bolted onto a serialization trait; it is
meaningless for most `Encode` impls. Remove it from the `Encode` trait. Expose it as
an **inherent method** on the request types that actually carry a suppress bit
(those already wrapping `SuppressablePositiveResponse`) and on `Request<'a>`, which
forwards to its inner variant. `Encode` becomes a clean, general codec trait.

### Decision 7 — Enforce the `encode` / `encoded_size` invariant

Many `encode` impls return `self.encoded_size()` after writing, with no guard against
the two diverging. A caller pre-sizing a buffer from `encoded_size()` would silently
overflow/underfill if they drift.

**Action:** Document the invariant on `Encode::encoded_size` ("must equal the byte
count `encode` writes"). Add a small generic test helper —
`assert_encode_size_agrees<T: Encode>(value: &T)` — that encodes into a counting
writer and asserts the returned length equals `encoded_size()`, and apply it across
every service's unit tests.

### Decision 8 — Document the `Decode` remainder contract

`Decode::decode` returns `(value, remaining)` for composition, but every top-level
frame decoder consumes the whole buffer and returns `&[]`. Document on the trait that
frame-level decoders are whole-buffer (use `decode_exact` semantics) and the remainder
is meaningful only for leaf/sequence decoding. No code change beyond doc comments.

### Decision 9 — Document the integration model (scope alignment)

The README is a stub. Add an **Integration** section stating the contract explicitly:

> This crate is a synchronous, allocation-free codec. To use it over any transport
> (DoIP, UDSonIP, ISO-TP, …), decode inbound frames from the received `&[u8]` and
> encode outbound frames into any `embedded_io::Write` or a caller-owned buffer sized
> via `encoded_size()`. The crate owns no sockets, buffers, or async runtime; drive
> I/O from your own sync or async layer.

Include one short encode + one short decode snippet. This prevents downstream authors
from re-introducing async coupling at the codec layer.

Because the audience is C developers new to Rust, the decode snippet must make the
**borrow** explicit: the decoded value points into the receive buffer you passed in
(like a `struct` overlaid on a `char buf[]`), and is valid only while that buffer
lives — copy out any fields you need to keep. This is the single Rust-specific concept
the docs must land clearly.

### Decision 10 — State the service-coverage boundary

`UdsServiceType` enumerates ~10 services the dispatch enums do not model
(`Authentication`, `ReadMemoryByAddress`, `RequestUpload`, `ResponseOnEvent`, etc.).
With Decision 5, frames for these decode into `Other` rather than erroring. Document,
in the crate root, the explicit list of fully-modeled services vs. those reached only
through `Other`, so coverage is a stated decision rather than an accident.

---

## Components touched

- `src/traits.rs` — remove `Identifier`/`RoutineIdentifier`/`impl_identifier!` and
  the three blanket impls; remove `is_positive_response_suppressed` from `Encode`;
  expand `Decode` doc comments; remove `DiagnosticDefinition`.
- `src/protocol_definitions.rs` — **deleted** (module + `pub use` removed from `lib.rs`).
- `src/lib.rs` — drop `UdsSpec`, `DiagnosticDefinition`, and `protocol_definitions`
  re-exports; update exports for renamed types.
- `src/common/diagnostic_identifier.rs` — replace `impl_identifier!` usage with direct
  `Encode`/`Decode` impls for `UDSIdentifier` / `UDSRoutineIdentifier`.
- `src/services/read_data_by_identifier.rs` — `ReadDataByIdentifierRequestTx<'d>` over
  `&'d [u16]`.
- `src/services/write_data_by_identifier.rs` — `WriteDataByIdentifierRequestTx<'d>`
  over `&'d [u8]`; `WriteDataByIdentifierResponse { identifier: u16 }`.
- `src/services/routine_control.rs` — `RoutineControlRequestTx<'d>` /
  `RoutineControlResponseTx<'d>` over `&'d [u8]`.
- `src/request.rs` / `src/response.rs` — add `Other { service, data }`; remove
  `UdsResponse`; move suppression to inherent method; update construction for renamed
  builders.
- `README.md` — add Integration section + snippets.
- Service test modules — apply `assert_encode_size_agrees`.

## Testing

- Preserve all existing round-trip tests; update them for renamed types and the new
  builder signatures.
- Add the `assert_encode_size_agrees` harness and apply per service.
- Add decode tests that exercise `Request::Other` / `Response::Other` for a
  known-but-unmodeled service byte.
- Verify the full matrix still builds and passes: default (`std`),
  `--no-default-features --features alloc`, `--no-default-features`, and
  `thumbv6m-none-eabi`. Clippy clean on all host combos.

## Out of scope (explicitly deferred)

- Implementing additional UDS services (`Authentication`, `ReadMemoryByAddress`,
  `RequestUpload`, etc.). They remain reachable via `Other`.
- Any transport, session, or async layer — belongs in downstream crates.
- An `embedded-io-adapters` std-bridge convenience (can be a later additive feature).

## Risks

- **Broad rename churn.** Mitigated by no external consumers (no `examples/`,
  `tests/`, or dependent crates in-tree) and a green CI matrix to catch breakage.
- **`Other` vs `ServiceNotImplemented` overlap.** The implementation plan must settle
  whether `ServiceNotImplemented` is fully removed or retained for truly unknown bytes.
