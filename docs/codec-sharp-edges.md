# automotive-wire-codec 0.2.0 — sharp-edge catalog (from uds_protocol spikes)

Findings from the migration spikes on branch `spike/wire-codec`, gathered while
porting `uds_protocol` decode/encode paths to `automotive-wire-codec` (`awc`)
0.2.0. Each entry is written to be paste-ready as a GitHub issue on
`luminartech/automotive_wire_codec` and to be actionable by someone who has
never seen this spike.

**Severity legend.** **blocker** = fix before any protocol crate migrates;
**wart** = fix eventually; **gap** = missing feature that forces callers to
re-derive functionality.

**Evidence convention.** Commit shas name the spike commit on `spike/wire-codec`
that produced the cited code (`git log --oneline`). Installed codec source is
`~/.cargo/registry/src/.../automotive-wire-codec-0.2.0/`.

**Summary of findings**

| ID | Severity | One-line |
|---|---|---|
| SE-1 | blocker | `read_be_uint`/`write_be_uint` panic on wire-controlled widths (release-build DoS on the write half) |
| SE-2 | gap | `DecodeIterator` exposes no `len()`/`ExactSizeIterator` — cannot subsume the hand-rolled DTC iterators |
| SE-3 | gap | No `param_length`/`minimal_be_len` helper — the encode-side twin of `read_be_uint` is missing |
| SE-4 | gap | `read_be_uint` returns `u128`, forcing truncating `as` casts at every narrower call site |
| SE-5 | wart | No `encode_to_slice` convenience — every fixed-buffer call site writes the `&mut &mut [u8]` coercion by hand |
| SE-6 | wart | No migration guidance for associated-`Error` adoption (dual-trait coexistence, `From`-fragment pattern) |
| SE-7 | gap | No bounds-check-then-`Incomplete` helper — the "check length, else return `Incomplete`" idiom is hand-written ~63× in uds today |

Plus a **Migration notes** section (a constraint the migration must design
around, not a codec defect) and a **Refuted / not an issue** section (things the
spikes checked and found sound, recorded so the codec discussion need not
re-litigate them).

---

## SE-1: `read_be_uint`/`write_be_uint` panic on wire-controlled widths (blocker)

**Severity: blocker.** This is the headline finding; it gates every consumer.

**Context.** In UDS, integer widths are wire-controlled data, not caller
constants: `fileSizeParameterLength` (RequestFileTransfer / SizePayload) is a
raw byte off the frame, and ALFID nibbles drive `read_be_uint` widths. The codec
guards width with only `debug_assert!(n <= 16)` in both `read_be_uint`
(`read.rs:79`) and `write_be_uint` (`write.rs:56`) — a *programming-error*
contract. But for a protocol crate the out-of-range width is a *data* error
reachable from a hostile or corrupt frame. There is no `Result` path for it, so
a consumer that forgets an upstream guard has no way to fail safe.

**Observed behavior** (spike panic probes, exact):

*Debug builds* — both halves panic at the `debug_assert`:
- `read_be_uint(&buf, 255)` panics at `read.rs:79:5`: `read_be_uint: n must be <= 16`
- `write_be_uint(&mut w, v, 255)` panics at `write.rs:56:5`: `write_be_uint: n must be <= 16`

*Release builds* — the `debug_assert` compiles away, and the two halves diverge:
- `read_be_uint(&buf, 255)` **silently succeeds**: it consumes 255 bytes and
  returns a plausible-but-wrong value (for all-zero input: value `0`, `rest.len()`
  shrunk by 255). No error, no flag. Silent data corruption.
- `write_be_uint(&mut w, v, 255)` **panics anyway**, via slice-range underflow.
  The impl computes `value.to_be_bytes()[16 - n..]`; with `n = 255usize`,
  `16usize - 255usize` wraps, and slice indexing panics with the verbatim
  message:

  > range start index 18446744073709551377 out of range for slice of length 16

  This makes the write half a **release-build panic** on hostile width — an
  unauthenticated remote DoS if width reaches it unvalidated. It is the strongest
  evidence for this entry: the codec's own "just a debug_assert" stance does not
  hold on the write side, because the arithmetic bug re-introduces a panic in
  release.

**Evidence.** `spike/wire-codec`, `src/spike_codec.rs::width_contract_probe`
(commit `9142461`; lint touch-up `505b0ed`). Tests
`read_be_uint_panics_on_hostile_width_in_debug` and
`write_be_uint_panics_on_hostile_width_in_both_profiles` record both profiles.
Guard burden confirmed in the SizePayload spike (`0cd26fe`): the impl carries an
explicit `if n == 0 || n > MAX_SIZE_PARAM_WIDTH { return Err(...) }` precisely
because the codec cannot supply that check.

**Proposed change.** Make width a checked contract on both halves. Either:

```rust
// Option A — fallible width, minimal surface change:
pub fn read_be_uint(buf: &[u8], n: usize) -> Result<(u128, &[u8]), Error>;
//                                       where Error: From<InvalidWidth>
pub fn write_be_uint<W: Write>(w: &mut W, v: u128, n: usize) -> Result<usize, Error>;
// return Err(InvalidWidth { max: 16, got: n }) instead of debug_assert!,
// as a new error fragment with Into for both Decode and Encode error channels.

// Option B — make illegal widths unrepresentable:
pub struct Width(u8); // validated 0..=16 (or 1..=16, see below) at construction
pub fn read_be_uint(buf: &[u8], n: Width) -> Result<(u128, &[u8]), Incomplete>;
```

Whichever is chosen, **fix the write-half `16 - n` underflow first** — that is a
live release-build panic regardless of the API decision. Option A is the smaller
migration for existing callers; Option B is stronger (invalid width cannot be
constructed) but ripples into every call site's signature.

**Open semantic question (attach to this redesign, not a separate blocker):
zero-width (`n == 0`).** The codec accepts `n == 0` today (no lower bound; the
`debug_assert` only caps the top). Production `SizePayload::decode`
(`request_file_transfer.rs`) also silently accepts a `0x00` width byte — both
size reads trivially return `0` and decode succeeds with both fields zero. The
spike's stricter `Decode` impl rejects `n == 0` with
`Error::InvalidFileSizeParameterLength(0)`, following UDS's `1..=16` constraint
on `fileSizeParameterLength`. This is a real behavior divergence surfaced by the
spike (not exercised by the roundtrip or the hostile-width tests; visible by
inspection). The codec-level decision the redesign must make explicit: does
`read_be_uint`/`write_be_uint` *define* zero-width as a legal "encode/decode
nothing" (`n == 0` returns `Ok(0)` / writes nothing), or as an error? Whichever
the codec picks, document it — because whether the UDS-side `n == 0` rejection is
a correctness *improvement* or an unintended behavior change depends entirely on
that codec-level definition.

---

## SE-2: `DecodeIterator` exposes no length / `ExactSizeIterator` (gap)

**Severity: gap.** Blocks subsuming the hand-rolled DTC iterators without losing
their `len()` contract.

**Context.** UDS has several fixed-width record streams (DTC-and-status records,
etc.). The hand-rolled iterators expose `const fn len() -> usize` by dividing the
remaining byte count by the fixed record size, so callers can pre-allocate or
assert collection bounds up front. The codec's `DecodeIterator` adapter iterates
correctly but exposes no length at all — no `len()`, no `ExactSizeIterator`,
even though the element wire size is a compile-time constant for a
fixed-width record.

**Evidence.** `spike/wire-codec`, `src/spike_codec.rs` `impl awc::DecodeIter for
DTCRecord` + `dtc_record_tests` (commit `493a36e`). `DTCRecord` is a 3-byte
fixed record. Test `adapter_has_no_len_unlike_hand_rolled_iterators` documents
the gap: for a 12-byte buffer a hand-rolled 3-byte-record iterator reports
`12 / 3 = 4` up front, while the adapter can only be driven to exhaustion to
learn the count. (The count is `4`; the adapter yields no number.)

**Adjacent observations from the same spike** (fold into the same issue — they
describe the same trait's boundaries):

- **`DecodeIter::Error` is constrained to `From<Incomplete>` only, which is
  narrower than the hand-rolled iterators need.** `DTCRecord::DecodeIter` sets
  `type Error = crate::Error` (a 15+-variant enum), which satisfies
  `From<Incomplete>` trivially — sufficient for `DTCRecord`, whose only boundary
  case is "ran out of bytes." But a hand-rolled iterator over a *different* DTC
  record shape emits `Error::IncorrectMessageLengthOrInvalidFormat` (a distinct
  domain meaning for "partial record at an iteration boundary"), which is *not*
  reducible to `Incomplete`. Such an iterator must either wrap codec errors
  (ugly) or keep hand-rolling (defeats the adapter). Not a bug — the constraint
  is well-designed — but a real expressiveness ceiling: codec-backed iterators
  cannot emit domain-semantic errors beyond "out of bytes."
- **The crate's iterators differ in fusing behavior today.** The codec
  `DecodeIterator` adapter *fuses* after an error (test
  `partial_record_is_an_error_then_iterator_fuses`: yields `Ok`, then
  `Some(Err(...))`, then `None`). The uds hand-rolled iterators do *not* fuse —
  after an `Err` the remaining slice is untouched, so they yield the *same error
  forever*. Whichever behavior the codec standardizes on, the catalog flags that
  the two families disagree today; a migration that swaps one for the other is a
  behavior change callers can observe.

**Proposed change.** Add an opt-in static-size hook so stateless fixed-width
impls can advertise their record size, and have the adapter surface a length:

```rust
pub trait DecodeIter<'a>: Sized {
    type Error: From<Incomplete>;
    fn decode_next(buf: &'a [u8]) -> Result<Option<(Self, &'a [u8])>, Self::Error>;

    /// Wire size of one element, if fixed at compile time. Default: None.
    const WIRE_SIZE: Option<usize> = None;
}

// Then the adapter can implement:
impl<'a, T: DecodeIter<'a>> DecodeIterator<'a, T> {
    /// Remaining element count — available only when WIRE_SIZE is Some.
    pub fn len(&self) -> Option<usize> {
        T::WIRE_SIZE.map(|w| self.remaining.len() / w)
    }
}
// and impl ExactSizeIterator only for a SizedDecodeIter subtrait
// (or via specialization when WIRE_SIZE is Some).
```

`len()` returns `Some` only when `WIRE_SIZE` is `Some`; variable-width impls keep
the `None` default and lose nothing. If a true `ExactSizeIterator` impl is
wanted, gate it behind a `SizedDecodeIter` marker subtrait rather than the base
trait so variable-width impls stay expressible.

---

## SE-3: no `param_length` / `minimal_be_len` helper — the encode-side twin is missing (gap)

**Severity: gap.**

**Context.** `read_be_uint(buf, n)` reads a big-endian integer of a *given*
width. Its natural encode-side partner is "how many big-endian bytes does this
value need?" — `(BITS - leading_zeros).div_ceil(8)`. The codec has the reader
but not this. uds carries four typed copies (`param_length_u16/u32/u64/u128`,
`src/shared/util.rs`) that are pure bit-math with no UDS semantics, and any
protocol emitting minimal-width length/size/address fields must re-derive it.

**Evidence.** `spike/wire-codec`, `src/spike_codec.rs` `impl awc::Encode for
SizePayload::encoded_size` (commit `0cd26fe`) calls
`crate::param_length_u128(core::cmp::max(a, b)).max(1)` to compute the wire
width — the codec offered nothing, so the spike reached back into uds's own
helper. Definitions: `src/shared/util.rs:53-78`.

**Proposed change.** Promote a single generic minimal-width helper into the codec
as the encode-side twin of `read_be_uint`:

```rust
/// Minimal number of big-endian bytes needed to represent `value`
/// (0 needs 0 bytes; callers wanting >=1 apply `.max(1)`).
pub const fn minimal_be_len(value: u128) -> u8 {
    ((u128::BITS - value.leading_zeros()).div_ceil(8)) as u8
}
```

One `u128` entry point covers all widths (callers pass narrower values by
widening, which is lossless). Consumers re-export it or delete their local
copies. Name is open (`param_length` / `minimal_be_len` / `be_byte_width`); the
signature is the point.

---

## SE-4: `read_be_uint` returns `u128`, forcing truncating casts at narrower call sites (gap)

**Severity: gap.**

**Context.** `read_be_uint` returns `(u128, &[u8])` regardless of the requested
width. When the destination field is narrower than `u128` — the common case —
the caller must add an `as` cast and silence
`clippy::cast_possible_truncation`. The cast *silences* the lint; it does not
*prove* the value fits. Soundness then rests on an out-of-band invariant, and if
that invariant is ever loosened the redundant cast becomes a silent truncation
bug with no compiler help.

**Evidence.** `spike/wire-codec`, `src/spike_codec.rs` `impl awc::Decode for
RequestDownloadRequest` (commit `493be58`). Its fields are `u64`
(`memory_address`) and `u32` (`memory_size`), so the impl needs
`memory_address as u64` and `memory_size as u32`, both under a function-level
`#[allow(clippy::cast_possible_truncation)]`. Contrast: the SizePayload spike
(`0cd26fe`) needed *no* cast because its fields are already `u128`. The casts are
sound here only because `MemoryFormatIdentifier::try_from`
(`src/shared/format_identifiers.rs:38-62`) structurally caps `addr_len <= 4` and
`size_len <= 3` — nothing at the call site enforces it.

**Proposed change.** Add a width-generic reader that returns the value already in
the caller's field type and validates width against that type (which *also*
closes SE-1 for this call site — an out-of-range `n` for the target type becomes
a typed error, not a debug_assert that vanishes in release):

```rust
pub fn read_be_uint_into<'a, T>(buf: &'a [u8], n: usize)
    -> Result<(T, &'a [u8]), Error>
where
    T: TryFrom<u128> + ...,           // unsigned integer target
{
    // error (not debug_assert) if n > size_of::<T>();  see SE-1
    // read n big-endian bytes, return already-typed value
}
```

Call sites become `read_be_uint_into::<u64>(rest, addr_len)?` and
`read_be_uint_into::<u32>(rest, size_len)?` — no `as`, no `#[allow]`, and an
out-of-range width fails loudly instead of truncating silently.

**Incidental doc/code mismatch noticed during this spike** (not a codec issue —
recorded so it is not lost): `src/services/request_download.rs:34`'s doc comment
says the memory address is "max 5 bytes", but the code's `MemoryFormatIdentifier`
match pattern is `1..5`, which (Rust exclusive range) allows only up to 4. A
pre-existing uds doc/code discrepancy; out of scope to fix here, but it is
exactly the kind of loosened-bound that would turn the SE-4 cast into a live
truncation bug.

---

## SE-5: no `encode_to_slice` convenience for fixed buffers (wart)

**Severity: wart. Confirmed** (the spikes produced no counter-evidence; the
boilerplate appears in every encode test).

**Context.** `Encode::encode` takes an `embedded_io::Write`. A `&mut [u8]`
implements `Write`, but to hand a stack buffer to a `&mut impl Write` parameter
the caller must first bind a re-borrowable slice cursor — `let mut w: &mut [u8]
= &mut buf;` — and pass `&mut w` (a `&mut &mut [u8]`). This coercion dance recurs
at every fixed-buffer encode call site, which for embedded/no-alloc UDS is *every*
encode call site.

**Evidence.** `spike/wire-codec`, `src/spike_codec.rs`. The `let mut w: &mut [u8]
= &mut buf;` + `encode(&x, &mut w)` pattern appears verbatim in every encode
test: EcuReset request/response (`6520fd8`), SizePayload (`0cd26fe`), and the
RequestDownload/DTC tests. No spike found a cleaner spelling; the boilerplate is
uniform and unavoidable with the current surface.

**Proposed change.** Add a slice convenience to the `Encode` trait that hides the
cursor coercion and reports bytes written:

```rust
pub trait Encode {
    type Error;
    fn encode(&self, w: &mut impl embedded_io::Write) -> Result<usize, Self::Error>;
    fn encoded_size(&self) -> usize;

    /// Encode into a fixed slice; Err if the buffer is too small.
    fn encode_to_slice(&self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut cursor: &mut [u8] = buf;
        self.encode(&mut cursor)
    }
}
```

Call sites collapse to `let n = x.encode_to_slice(&mut buf)?;`.

---

## SE-6: no migration guidance for associated-`Error` adoption (wart)

**Severity: wart.** Not a code defect — a documentation gap that every consumer
(uds, doip, someip, …) will hit identically.

**Context.** Migrating to the codec means adding `type Error = crate::Error;` to
every `Decode`/`Encode`/`DecodeIter` impl block and defining the `From`
fragments that lift the codec's error atoms (`Incomplete`, `TrailingBytes`,
`embedded_io::ErrorKind`) into the crate's own error enum. In uds this touched
~46 impl blocks. The pattern is mechanical but non-obvious the first time, and
there is no codec-side note on: (a) how a crate mid-migration runs its old
concrete-error decode and the new associated-`Error` decode side by side; (b)
the exact `From`-fragment set to implement; (c) that a single blanket primitive
impl is impossible *because* the error is associated, so per-type impls (or a
macro) are the only options.

**Evidence.** `spike/wire-codec`. The Task-2 error restructure (prep commit
`bdc61e5`) established the `From<Incomplete>`/`From<TrailingBytes>` fragments;
every spike impl (`6520fd8`, `0cd26fe`, `493a36e`, `493be58`) sets `type Error =
Error` and relies on those fragments lifting through `?`. The pattern worked
cleanly but was re-derived by inspection, not from any codec doc.

**Proposed change.** Ship a short "Adopting associated `Error`" section in the
codec's docs / a `MIGRATION.md`, containing: the canonical `From`-fragment set to
implement (`From<Incomplete>`, `From<TrailingBytes>`,
`From<embedded_io::ErrorKind>`); a worked dual-trait coexistence example (old
concrete-error impl and new associated-`Error` impl on the same type during
transition); and an explicit statement that no blanket primitive impl is possible
so per-type impls are expected. A `#[derive(Encode)]`/`#[derive(Decode)]`
proc-macro was considered and **deferred** — it adds a proc-macro dependency at
the base of the stack and cuts against the "concrete and simple" grain; revisit
only after doip/someip confirm the boilerplate is genuinely painful across
consumers. The guidance is the near-term deliverable regardless.

---

## SE-7: no bounds-check-then-`Incomplete` helper (gap)

**Severity: gap.** New finding from the prep pass; a concrete ergonomics data
point for a small codec API addition.

**Context.** The single most repeated decode idiom in uds is "if the buffer is
shorter than what I need, return `Incomplete { needed, available }`." During the
Task-2 error restructure this exact construction was written by hand **~63
times** across the crate. The codec provides `Incomplete` and the readers that
produce it, but no helper for the extremely common "check a length up front, else
error" shape, so each call site rebuilds it.

**Evidence.** `spike/wire-codec` / prep pass. `Incomplete { needed, available:
buf.len() }` construction counted at ~63 sites in the Task-2 error restructure
(prep commit `bdc61e5`). The uniformity is the signal: a one-line helper would
replace all of them.

**Proposed change.** Ship a bounds-check-then-error helper on the codec:

```rust
/// Ok(()) if `buf` has at least `needed` bytes, else Err(Incomplete).
#[inline]
pub fn ensure_len(buf: &[u8], needed: usize) -> Result<(), Incomplete> {
    if buf.len() < needed {
        Err(Incomplete { needed, available: buf.len() })
    } else {
        Ok(())
    }
}
```

Call sites become `ensure_len(buf, total)?;` before an aggregate read, replacing
the hand-built struct literal at ~63 uds sites (and the equivalents in every
other consumer).

---

## Migration notes (not codec defects — constraints the migration designs around)

**Invariant-bearing private fields block out-of-module `Decode` impls, and the
`new()` workaround is not wire-faithful.** This is guidance for *how* to migrate,
not a codec bug — recorded here so the constraint is answered once, per type,
rather than rediscovered.

uds encapsulates invariant-bearing structs: their fields are private and
construction goes through a validating constructor (per the crate's
field-visibility convention). `RequestDownloadRequest` is one such type — fields
`data_format_identifier`, `address_and_length_format_identifier`,
`memory_address`, `memory_size` are all private. A `Decode` impl written in a
*sibling* module (as the spike's `src/spike_codec.rs` is, and as a centralized
codec-impls module would be) therefore **cannot** construct `Self { .. }` and must
fall back to the public constructor.

That fallback has a semantic cost. `RequestDownloadRequest::new(...)`
*recomputes* the minimal ALFID nibble widths from the decoded values (via
`leading_zeros().div_ceil(8)`) rather than preserving the wire's *declared*
widths. ISO 14229-1 does not require senders to use minimal widths, so a wire
that over-declares (e.g. ALFID says 4 address bytes for a value needing 1) would
decode to a struct whose `address_and_length_format_identifier` **disagrees with
the original wire byte**. The spike's `codec_decode_matches_existing_decode` test
(commit `493be58`) passes only because its test vector uses minimal-width values,
so the recomputed identifier coincidentally equals the declared one — a property
of the test data, not of the approach.

**Consequence for the no-std/no-alloc migration.** If codec `Decode`/`Encode`
impls are centralized in a module or crate separate from the domain-type
definitions, then for *every* invariant-bearing type one of these must be chosen
explicitly:

1. Define the codec impls *inside* (or with `pub(crate)` privileged access to)
   the type's defining module, so `Self { .. }` construction preserves
   wire-declared state; or
2. Expose a `pub(crate)` raw/unchecked constructor that preserves
   wire-declared-but-non-minimal state; or
3. Accept semantic drift from recomputation (only safe for types with no
   redundant-encoding freedom on the wire).

This question must be answered per invariant-bearing type in scope for
migration, not just `RequestDownloadRequest`.

---

## Refuted / not an issue

Recorded so the codec discussion need not re-open them. Each was checked by a
spike and found sound.

- **Optional trailing fields decode cleanly.** ISO 14229-1's optional
  trailing-byte pattern (e.g. `EcuResetResponse::power_down_time`) maps directly
  onto the `(value, rest)` tuple style: match the read, destructure on `Ok`, use
  a default and the prior `rest` on `Err(_)`. No friction. (`6520fd8`)
- **`TrailingBytes` lifts through `?` correctly.** `Decode::decode_exact`
  surfaces trailing data as `TrailingBytes(usize)`, and the Task-2
  `From<TrailingBytes>` fragment raises it seamlessly. Test
  `decode_exact_surfaces_trailing_bytes_via_error`. (`6520fd8` / `bdc61e5`)
- **`read_array::<N>` destructuring is a strength, not a gap.**
  `read_array::<3>(buf)?` returns `([a, b, c], rest)` — cleaner than manual
  indexing; used directly in `DTCRecord::decode`. Good API for fixed-width reads.
  (`493a36e`)
- **`(value, rest)` threading beats manual offset arithmetic.** The codec-style
  `RequestDownloadRequest::decode` threads `rest` through four chained reads with
  zero manual offset math or raw indexing, versus the old decoder's `total`
  precheck + four hand-computed index/slice expressions. No correctness gap;
  flatter and less error-prone. One behavioral nuance to flag during migration
  (below), but the shape itself is not an issue. (`493be58`)
- **Per-field vs whole-record `Incomplete` is a behavior change to flag, not a
  defect.** The old decoder's upfront length check reports
  `Incomplete { needed: <whole record>, available: buf.len() }`; the codec-style
  per-read decode reports `{ needed, available }` for whichever field's read
  first underruns (e.g. `{ needed: 4, available: 3 }` where the old reported
  `{ needed: 8, available: 5 }`). Neither is "more correct" — they answer
  different questions (bytes missing from the whole record vs. from the current
  field). Callers doing partial-buffer reassembly that pattern-match on
  `Incomplete.needed` should be audited during migration. (`493be58`)
- **Codec + stack builds cleanly on no-std / bare-metal.** `cargo check
  --no-default-features --features spike-codec` passes on host and on both
  `thumbv8m.main-none-eabihf` and `thumbv6m-none-eabi`; no `std` is pulled in via
  `embedded-io` or `automotive-wire-codec`, and no feature-unification surprises.
  The std-elimination path is sound. (`7831141`)
- **Crate lints are satisfied by codec impls without ceremony.** Under
  `#![warn(clippy::pedantic, missing_docs)]`, the codec-trait impl blocks needed
  no doc comments and raised no pedantic/`missing_docs` findings. Standard Rust
  lint behavior; no codec-imposed friction. (`6520fd8`)
