# API Consistency — Phase 2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Finish the cohesive no_std breaking change: drop the misleading `…Tx`/`…Rx` suffixes from bidirectional descriptor types, wrap the remaining raw enum variants in their descriptors (RDBI excepted), and remove the duplicated variable-length integer codec and the duplicate primitive macros.

**Architecture:** Seven commits. (1) Rename 13 bidirectional types (drop suffix). (2) Extract a `read_be_uint`/`write_be_uint` helper pair and use it at the 4 hand-rolled sites. (3) Merge the two identical primitive macros. (4) Wrap `WriteDataByIdentifier` (req + resp) in the enums. (5) Wrap `RoutineControl` (req + resp) using `SuppressablePositiveResponse`. (6) Add `Decode` to 5 DTC parameter types + a 25-variant `ReadDTCInfoRequest::Decode`, and wrap `Request::ReadDTCInfo`. (7) Full-matrix verification.

**Tech Stack:** Rust, no_std + no_alloc, `embedded_io::Write` for encode, borrowed `&[u8]` for decode. Spec: `docs/superpowers/specs/2026-06-03-api-consistency-phase2-design.md`.

______________________________________________________________________

## Conventions for every task

- Local per-task verification: `cargo test --all-features` (fast host run).
- Commit message format:
  ```
  <imperative summary>

  Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
  ```
- Platform is macOS: in-place `sed` requires `sed -i ''`.

______________________________________________________________________

## Task 1: Rename bidirectional descriptor types (drop `…Tx`/`…Rx`)

Pure mechanical rename across the crate. The compiler + a final grep are the safety net. `ReadDataByIdentifierRequestTx` is **NOT** renamed (it is the one genuinely TX-only type).

**Files:** all of `src/` (and a README check).

- [ ] **Step 1: Apply the renames**

Each old name is a full unique identifier, so a global text replace is safe. Run these from the repo root (the trailing `''` after `-i` is required on macOS):

```bash
cd /Users/zacharyheylmun/dev/rust/uds_protocol
for pair in \
  "SecurityAccessRequestTx:SecurityAccessRequest" \
  "SecurityAccessResponseTx:SecurityAccessResponse" \
  "TransferDataRequestTx:TransferDataRequest" \
  "TransferDataResponseTx:TransferDataResponse" \
  "RequestFileTransferRequestTx:RequestFileTransferRequest" \
  "RequestFileTransferResponseTx:RequestFileTransferResponse" \
  "RequestDownloadResponseTx:RequestDownloadResponse" \
  "RoutineControlRequestTx:RoutineControlRequest" \
  "RoutineControlResponseTx:RoutineControlResponse" \
  "WriteDataByIdentifierRequestTx:WriteDataByIdentifierRequest" \
  "ReadDTCInfoResponseRx:ReadDTCInfoResponse" \
  "NamePayloadTx:NamePayload" \
  "SentDataPayloadTx:SentDataPayload" ; do
  old="${pair%%:*}"; new="${pair##*:}"
  grep -rl "$old" src README.md | xargs sed -i '' "s/${old}/${new}/g"
done
```

Note: the order matters only for substrings; none of these old names is a substring of another old name, so order is irrelevant here. `ReadDataByIdentifierRequestTx` contains none of the above as a substring, so it is untouched.

- [ ] **Step 2: Verify no old names remain (except the intentional one)**

```bash
grep -rn "RequestTx\|ResponseTx\|ResponseRx\|PayloadTx" src README.md
```

Expected: only `ReadDataByIdentifierRequestTx` matches. If any other old name appears, the rename missed a spot — re-run the relevant replacement.

- [ ] **Step 3: Build + test + fmt**

Run:

```bash
cargo build --all-features
cargo test --all-features
cargo fmt -- --check
```

Expected: PASS. (`cargo fmt` may reflow the now-shorter names in `use` lists; run `cargo fmt` if `--check` complains.)

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "$(printf 'rename bidirectional descriptor types: drop misleading Tx/Rx suffixes\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 2: Deduplicate the variable-length big-endian integer codec

Add one concrete helper pair to `src/common/util.rs` (kept `pub(crate)`) and call it from the four hand-rolled sites.

**Files:**

- Modify: `src/common/util.rs`

- Modify: `src/services/request_file_transfer.rs` (`SizePayload`, `FileSizePayload`, `DirSizePayload`)

- Modify: `src/services/request_download.rs` (`RequestDownloadRequest`)

- [ ] **Step 1: Write failing tests for the helpers**

In `src/common/util.rs`, inside the existing `#[cfg(test)] mod tests`, add:

```rust
    #[test]
    fn be_uint_roundtrip() {
        use crate::common::util::{read_be_uint, write_be_uint};
        let mut buf = [0u8; 16];
        let mut w = buf.as_mut_slice();
        let written = write_be_uint(0x00AB_CDEFu128, 3, &mut w).unwrap();
        assert_eq!(written, 3);
        assert_eq!(&buf[..3], &[0xAB, 0xCD, 0xEF]);
        let v = read_be_uint(&buf[..3], 3).unwrap();
        assert_eq!(v, 0x00AB_CDEF);
    }

    #[test]
    fn be_uint_zero_width() {
        use crate::common::util::{read_be_uint, write_be_uint};
        let mut buf = [0u8; 4];
        let mut w = buf.as_mut_slice();
        assert_eq!(write_be_uint(0, 0, &mut w).unwrap(), 0);
        assert_eq!(read_be_uint(&[], 0).unwrap(), 0);
    }

    #[test]
    fn read_be_uint_rejects_short_and_overwide() {
        use crate::common::util::read_be_uint;
        assert!(read_be_uint(&[0x01], 2).is_err());
        assert!(read_be_uint(&[0u8; 17], 17).is_err());
    }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test --all-features be_uint`
Expected: FAIL (helpers don't exist).

- [ ] **Step 3: Implement the helpers**

At the top of `src/common/util.rs`, add the import and helpers (place above the existing `param_length_*` functions):

```rust
use crate::Error;

/// Maximum width of a big-endian unsigned integer this codec handles.
const BE_UINT_MAX_BYTES: usize = 16;

/// Read the first `n` big-endian bytes of `src` as a left-padded `u128`.
///
/// # Errors
/// Returns [`Error::InsufficientData`] if `src` is shorter than `n`, or
/// [`Error::IncorrectMessageLengthOrInvalidFormat`] if `n > 16`.
pub(crate) fn read_be_uint(src: &[u8], n: usize) -> Result<u128, Error> {
    if n > BE_UINT_MAX_BYTES {
        return Err(Error::IncorrectMessageLengthOrInvalidFormat);
    }
    if src.len() < n {
        return Err(Error::InsufficientData(n));
    }
    let mut bytes = [0u8; BE_UINT_MAX_BYTES];
    bytes[BE_UINT_MAX_BYTES - n..].copy_from_slice(&src[..n]);
    Ok(u128::from_be_bytes(bytes))
}

/// Write the low `n` big-endian bytes of `value` to `writer`, returning `n`.
///
/// # Errors
/// Returns [`Error::IncorrectMessageLengthOrInvalidFormat`] if `n > 16`, or
/// [`Error::IoError`] if the writer fails.
pub(crate) fn write_be_uint(
    value: u128,
    n: usize,
    writer: &mut impl embedded_io::Write,
) -> Result<usize, Error> {
    if n > BE_UINT_MAX_BYTES {
        return Err(Error::IncorrectMessageLengthOrInvalidFormat);
    }
    let bytes = value.to_be_bytes();
    writer
        .write_all(&bytes[BE_UINT_MAX_BYTES - n..])
        .map_err(Error::io)?;
    Ok(n)
}
```

- [ ] **Step 4: Route the helpers through `common/mod.rs`**

In `src/common/mod.rs`, the `mod util;` line is followed by a `pub use util::{param_length_*}`. Add the new helpers to the crate-internal surface by changing that line's neighbours to also re-export them `pub(crate)`:

```rust
mod util;
pub use util::{param_length_u16, param_length_u32, param_length_u64, param_length_u128};
pub(crate) use util::{read_be_uint, write_be_uint};
```

- [ ] **Step 5: Use the helpers in `request_file_transfer.rs`**

Add `use crate::common::{read_be_uint, write_be_uint};` to the imports. Then:

In `impl Encode for SizePayload`, replace the two `uncompressed`/`compressed` `to_be_bytes()` + `write_all(&…[U128_MAX_BYTES - n..])` blocks with:

```rust
        write_be_uint(self.file_size_uncompressed, n, writer)?;
        write_be_uint(self.file_size_compressed, n, writer)?;
```

(keep the `file_size_parameter_length` byte write before them, and the `n > U128_MAX_BYTES` guard.)

In `impl Decode for SizePayload`, replace the `u_bytes`/`c_bytes` blocks with:

```rust
        let file_size_uncompressed = read_be_uint(&buf[1..], n)?;
        let file_size_compressed = read_be_uint(&buf[1 + n..], n)?;
```

and use those in the returned `Self`.

In `impl Encode for FileSizePayload`, replace the two write blocks with the same two `write_be_uint(... n ...)` calls. In `impl Decode for FileSizePayload`, replace the `u_bytes`/`c_bytes` blocks with:

```rust
        let file_size_uncompressed = read_be_uint(&buf[2..], n)?;
        let file_size_compressed = read_be_uint(&buf[2 + n..], n)?;
```

In `impl Encode for DirSizePayload`, replace the single `bytes` write block with `write_be_uint(self.dir_info_length, n, writer)?;`. In `impl Decode for DirSizePayload`, replace the `bytes` block with `let dir_info_length = read_be_uint(&buf[2..], n)?;`.

Keep every existing length/`total` bounds check and `n > U128_MAX_BYTES` guard exactly as-is; only the padding+convert lines change. `U128_MAX_BYTES` may now be unused — if the compiler warns, remove the `const U128_MAX_BYTES` definition.

- [ ] **Step 6: Use the helpers in `request_download.rs`**

Add `use crate::common::{read_be_uint, write_be_uint};` to the imports.

In `impl Encode for RequestDownloadRequest::encode`, replace the `addr_bytes`/`size_bytes` blocks with:

```rust
        write_be_uint(u128::from(self.memory_address), addr_len, writer)?;
        write_be_uint(u128::from(self.memory_size), size_len, writer)?;
```

where `addr_len` / `size_len` are the existing `memory_address_length` / `memory_size_length` reads.

In `impl Decode`, replace the `addr_bytes`/`size_bytes` blocks with:

```rust
        let memory_address = read_be_uint(&buf[2..], addr_len)? as u64;
        let memory_size = read_be_uint(&buf[2 + addr_len..], size_len)? as u32;
```

Keep the `total` bounds check. Add `#[allow(clippy::cast_possible_truncation)]` on the `decode` fn if clippy flags the `as u64`/`as u32` (the widths are bounded by the format nibble, ≤8 and ≤4).

- [ ] **Step 7: Verify and commit**

Run: `cargo test --all-features && cargo clippy --all-features && cargo fmt`
Expected: PASS, clippy clean. The existing payload round-trip tests must still pass.

```bash
git add -A
git commit -m "$(printf 'dedup variable-length big-endian integer codec into util helpers\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 3: Merge the duplicate primitive macros

**Files:** `src/common/primitive_generics.rs`

- [ ] **Step 1: Replace the two macros with one**

Replace the entire contents of `src/common/primitive_generics.rs` (both `unsigned_primitive_encode_decode!` and `signed_primitive_encode_decode!` definitions and their two invocations) with a single macro and one invocation:

```rust
use crate::{Decode, Encode, Error};

/// Implement [`Encode`] and [`Decode`] for integer primitives (no_std-compatible).
macro_rules! primitive_encode_decode {
    ( $($primitive:ty), * ) => {
        $(
        impl Encode for $primitive {
            fn encoded_size(&self) -> usize {
                core::mem::size_of::<$primitive>()
            }
            fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
                writer.write_all(&self.to_be_bytes()).map_err(Error::io)?;
                Ok(self.encoded_size())
            }
        }
        impl<'a> Decode<'a> for $primitive {
            fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
                const SIZE: usize = core::mem::size_of::<$primitive>();
                if buf.len() < SIZE {
                    return Err(Error::InsufficientData(SIZE));
                }
                let (head, tail) = buf.split_at(SIZE);
                let value = <$primitive>::from_be_bytes(head.try_into().unwrap());
                Ok((value, tail))
            }
        }
        )*
    };
}

primitive_encode_decode!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
```

- [ ] **Step 2: Verify and commit**

Run: `cargo test --all-features && cargo clippy --all-features && cargo fmt -- --check`
Expected: PASS (same 10 impls as before, generated by one macro).

```bash
git add src/common/primitive_generics.rs
git commit -m "$(printf 'merge identical signed/unsigned primitive codec macros\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 4: Wrap `WriteDataByIdentifier` in the enums

**Files:**

- Modify: `src/services/write_data_by_identifier.rs` (add `Decode` for `WriteDataByIdentifierRequest`)

- Modify: `src/request.rs`, `src/response.rs`

- [ ] **Step 1: Write a failing round-trip test**

In `src/request.rs` `mod tests` (create the test fn alongside existing ones), add:

```rust
    #[test]
    fn write_data_by_identifier_request_roundtrips() {
        // SID 0x2E, DID 0xF190, one data byte 0x01
        let wire = [0x2E, 0xF1, 0x90, 0x01];
        let (req, rest) = Request::decode(&wire).unwrap();
        assert!(rest.is_empty());
        assert!(matches!(req, Request::WriteDataByIdentifier(_)));
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --all-features write_data_by_identifier_request_roundtrips`
Expected: FAIL — `Request::WriteDataByIdentifier` still holds `&[u8]` (the `matches!` and/or type will mismatch once wired; before wiring it won't compile against the new variant).

- [ ] **Step 3: Add `Decode` for `WriteDataByIdentifierRequest`**

In `src/services/write_data_by_identifier.rs`, ensure `Decode` is imported (it is, from Phase 1). After the `impl Encode for WriteDataByIdentifierRequest` block add:

```rust
impl<'a> Decode<'a> for WriteDataByIdentifierRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        Ok((Self { payload: buf }, &[]))
    }
}
```

- [ ] **Step 4: Rewire `Request::WriteDataByIdentifier`**

In `src/request.rs`:

- Add `WriteDataByIdentifierRequest` to the `services::{…}` import.

- Change the variant from `WriteDataByIdentifier(&'a [u8])` to `WriteDataByIdentifier(WriteDataByIdentifierRequest<'a>)` and update its doc comment.

- In `decode`, change the arm to:

  ```rust
  UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(
      <WriteDataByIdentifierRequest as Decode>::decode_exact(payload)?,
  ),
  ```

- In `encoded_size`, remove `WriteDataByIdentifier` from the grouped `ReadDataByIdentifier | WriteDataByIdentifier | ReadDTCInfo => bytes.len()` arm (leaving `ReadDataByIdentifier | ReadDTCInfo`) and add `Self::WriteDataByIdentifier(req) => req.encoded_size(),`.

- In `encode`, likewise split it out: remove from the grouped raw arm and add `Self::WriteDataByIdentifier(req) => req.encode(writer)?,`.

- `service` arm is unchanged (`Self::WriteDataByIdentifier(_) => …`, already ignores payload).

- [ ] **Step 5: Rewire `Response::WriteDataByIdentifier`**

In `src/response.rs`:

- Add `WriteDataByIdentifierResponse` to the crate imports.

- Change the variant from `WriteDataByIdentifier(&'a [u8])` to `WriteDataByIdentifier(WriteDataByIdentifierResponse)` and update its doc comment (note: `WriteDataByIdentifierResponse` is fixed-size, no lifetime).

- In `decode`, change the arm to:

  ```rust
  UdsServiceType::WriteDataByIdentifier => Self::WriteDataByIdentifier(
      <WriteDataByIdentifierResponse as Decode>::decode_exact(payload)?,
  ),
  ```

- In `encoded_size`, remove `WriteDataByIdentifier` from the `ReadDataByIdentifier | WriteDataByIdentifier => bytes.len()` arm and add `Self::WriteDataByIdentifier(resp) => resp.encoded_size(),`.

- In `encode`, split it out the same way: `Self::WriteDataByIdentifier(resp) => resp.encode(writer)?,`.

- `response_sid` arm unchanged.

- [ ] **Step 6: Run tests, add response round-trip, verify**

Add to `src/response.rs` `mod tests`:

```rust
    #[test]
    fn write_data_by_identifier_response_roundtrips() {
        // SID 0x6E, echoed DID 0xF190
        let wire = [0x6E, 0xF1, 0x90];
        let (resp, rest) = Response::decode(&wire).unwrap();
        assert!(rest.is_empty());
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }
```

Run: `cargo test --all-features && cargo clippy --all-features && cargo fmt`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "$(printf 'wrap WriteDataByIdentifier request/response in their descriptor types\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 5: Wrap `RoutineControl` in the enums (with SPRMIB fidelity)

**Files:**

- Modify: `src/services/routine_control.rs` (use `SuppressablePositiveResponse`; add `Decode` for both)

- Modify: `src/request.rs`, `src/response.rs`

- [ ] **Step 1: Write failing round-trip tests (incl. suppress bit)**

In `src/request.rs` `mod tests` add:

```rust
    #[test]
    fn routine_control_request_roundtrips_with_suppress_bit() {
        // SID 0x31, sub 0x81 (StartRoutine + SPRMIB), RID 0xFF00, param 0xAA
        let wire = [0x31, 0x81, 0xFF, 0x00, 0xAA];
        let (req, rest) = Request::decode(&wire).unwrap();
        assert!(rest.is_empty());
        assert!(req.is_positive_response_suppressed());
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }
```

In `src/response.rs` `mod tests` add:

```rust
    #[test]
    fn routine_control_response_roundtrips() {
        // SID 0x71, sub 0x01, RID 0xFF00, status 0x10
        let wire = [0x71, 0x01, 0xFF, 0x00, 0x10];
        let (resp, rest) = Response::decode(&wire).unwrap();
        assert!(rest.is_empty());
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &wire);
    }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test --all-features routine_control_request_roundtrips_with_suppress_bit routine_control_response_roundtrips`
Expected: FAIL (variants still hold inline fields; no `Decode`).

- [ ] **Step 3: Rework `RoutineControlRequest` to use `SuppressablePositiveResponse`**

In `src/services/routine_control.rs`, change the imports to:

```rust
use crate::common::SuppressablePositiveResponse;
use crate::{Decode, Encode, Error, RoutineControlSubFunction};
```

Replace the `RoutineControlRequest` struct + its `impl` + `impl Encode` with:

```rust
/// Used by a client to execute a defined sequence of events and obtain any relevant results.
///
/// The payload is the routine identifier (2 bytes, big-endian) followed by any optional
/// routine input parameters, exactly as it appears on the wire after the sub-function byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequest<'d> {
    sub_function: SuppressablePositiveResponse<RoutineControlSubFunction>,
    raw_payload: &'d [u8],
}

impl<'d> RoutineControlRequest<'d> {
    /// Create a new `RoutineControlRequest`.
    #[must_use]
    pub const fn new(
        suppress_positive_response: bool,
        sub_function: RoutineControlSubFunction,
        raw_payload: &'d [u8],
    ) -> Self {
        Self {
            sub_function: SuppressablePositiveResponse::new(suppress_positive_response, sub_function),
            raw_payload,
        }
    }

    /// Whether the server should suppress the positive response (SPRMIB).
    #[must_use]
    pub fn suppress_positive_response(&self) -> bool {
        self.sub_function.suppress_positive_response()
    }

    /// The routine control operation (start, stop, or request results).
    #[must_use]
    pub fn sub_function(&self) -> RoutineControlSubFunction {
        self.sub_function.value()
    }

    /// The raw payload bytes: routine identifier followed by optional parameters.
    #[must_use]
    pub const fn raw_payload(&self) -> &[u8] {
        self.raw_payload
    }
}

impl Encode for RoutineControlRequest<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.raw_payload.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.sub_function)])
            .map_err(Error::io)?;
        writer.write_all(self.raw_payload).map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

impl<'a> Decode<'a> for RoutineControlRequest<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let sub_function = SuppressablePositiveResponse::try_from(buf[0])?;
        Ok((
            Self {
                sub_function,
                raw_payload: &buf[1..],
            },
            &[],
        ))
    }
}
```

- [ ] **Step 4: Add `Decode` for `RoutineControlResponse`**

In the same file, leave the `RoutineControlResponse` struct and its `impl Encode` as renamed in Task 1 (public fields `routine_control_type: RoutineControlSubFunction`, `raw_status_record: &[u8]`), and add after its `impl Encode`:

```rust
impl<'a> Decode<'a> for RoutineControlResponse<'a> {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let routine_control_type = RoutineControlSubFunction::try_from(buf[0])?;
        Ok((
            Self {
                routine_control_type,
                raw_status_record: &buf[1..],
            },
            &[],
        ))
    }
}
```

Update the existing in-file `mod test` if it constructs `RoutineControlRequest` with the old public-field/`new(sub_function, payload)` signature — switch to `RoutineControlRequest::new(false, RoutineControlSubFunction::StartRoutine, &payload)`.

- [ ] **Step 5: Rewire the enums**

In `src/request.rs`:

- Add `RoutineControlRequest` to the `services::{…}` import.
- Change the variant from the `RoutineControl { sub_function: u8, raw_payload: &'a [u8] }` struct form to `RoutineControl(RoutineControlRequest<'a>)`; update its doc comment.
- `decode` arm becomes:
  ```rust
  UdsServiceType::RoutineControl => Self::RoutineControl(
      <RoutineControlRequest as Decode>::decode_exact(payload)?,
  ),
  ```
  (delete the old manual `if payload.is_empty()` + field construction.)
- `encoded_size` arm: `Self::RoutineControl(req) => req.encoded_size(),` (remove the old `{ raw_payload, .. } => 1 + raw_payload.len()`).
- `encode` arm: `Self::RoutineControl(req) => req.encode(writer)?,` (remove the old field-writing block).
- `service` arm: `Self::RoutineControl(_) => UdsServiceType::RoutineControl,`.
- `is_positive_response_suppressed`: add `Self::RoutineControl(req) => req.suppress_positive_response(),` before the `_ => false` arm.

In `src/response.rs`:

- Add `RoutineControlResponse` to the crate imports.

- Change the variant from `RoutineControl { routine_control_type: u8, raw_status_record: &'a [u8] }` to `RoutineControl(RoutineControlResponse<'a>)`; update its doc comment.

- `decode` arm becomes:

  ```rust
  UdsServiceType::RoutineControl => Self::RoutineControl(
      <RoutineControlResponse as Decode>::decode_exact(payload)?,
  ),
  ```

- `encoded_size` arm: `Self::RoutineControl(resp) => resp.encoded_size(),`.

- `encode` arm: `Self::RoutineControl(resp) => resp.encode(writer)?,`.

- `response_sid` arm: `Self::RoutineControl(_) => UdsServiceType::RoutineControl.response_to_byte(),`.

- [ ] **Step 6: Run tests, verify, commit**

Run: `cargo test --all-features && cargo clippy --all-features && cargo fmt`
Expected: PASS, including the new suppress-bit round-trip.

```bash
git add -A
git commit -m "$(printf 'wrap RoutineControl in descriptors; round-trip SPRMIB via SuppressablePositiveResponse\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 6: `Decode` for the DTC params + `ReadDTCInfoRequest`; wrap `Request::ReadDTCInfo`

**Files:**

- Modify: `src/common/dtc_snapshot.rs`, `src/common/dtc_ext_data.rs`, `src/common/dtc_status.rs` (add `Decode` to the 5 param types)

- Modify: `src/services/read_dtc_information.rs` (add `ReadDTCInfoRequest::Decode`)

- Modify: `src/request.rs`

- [ ] **Step 1: Write a failing round-trip test (encode-as-oracle)**

In `src/services/read_dtc_information.rs`, in the `read_dtc_info_request_encode_tests` module, add (`decode_exact` returns just the value, so assert equality directly):

```rust
    #[test]
    fn read_dtc_info_request_roundtrips() {
        use crate::Decode;
        let cases = [
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportSupportedDTC),
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportDTC_ByStatusMask(
                DTCStatusMask::from(0xFF),
            )),
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ReportWWHOBDDTC_ByMaskRecord(
                FunctionalGroupIdentifier::EmissionsSystemGroup,
                DTCStatusMask::from(0x08),
                DTCSeverityMask::CheckImmediately,
            )),
            ReadDTCInfoRequest::new(ReadDTCInfoSubFunction::ISOSAEReserved(0x57)),
        ];
        for req in cases {
            let mut buf = [0u8; 16];
            let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
            let decoded = <ReadDTCInfoRequest as Decode>::decode_exact(&buf[..written]).unwrap();
            assert_eq!(decoded, req);
        }
    }
```

`ReadDTCInfoRequest` derives `PartialEq` already (verify; it derives `Clone, Copy, Debug, PartialEq`).

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --all-features read_dtc_info_request_roundtrips`
Expected: FAIL (no `Decode` for `ReadDTCInfoRequest`, and param types lack `Decode`).

- [ ] **Step 3: Add `Decode` to the 5 DTC parameter types**

These mirror their Phase 1 `Encode` (read exactly 1 byte). Each round-trips its `Encode` (verified: `new`/`from` followed by `value()`/`bits()`/`.0` is the identity).

In `src/common/dtc_snapshot.rs`, add `Decode` to the import (`use crate::{Decode, Encode, Error};`) and after the `impl Encode for DTCSnapshotRecordNumber`:

```rust
impl<'a> Decode<'a> for DTCSnapshotRecordNumber {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        Ok((Self::new(buf[0]), &buf[1..]))
    }
}
```

In `src/common/dtc_ext_data.rs`, add `Decode` to the import and after `impl Encode for DTCExtDataRecordNumber`:

```rust
impl<'a> Decode<'a> for DTCExtDataRecordNumber {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        Ok((Self::new(buf[0]), &buf[1..]))
    }
}
```

In `src/common/dtc_status.rs` (already imports `Decode`), add after each type's `Encode` impl:

```rust
impl<'a> Decode<'a> for DTCStoredDataRecordNumber {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        Ok((Self::from(buf[0]), &buf[1..]))
    }
}

impl<'a> Decode<'a> for DTCSeverityMask {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        Ok((Self::from(buf[0]), &buf[1..]))
    }
}

impl<'a> Decode<'a> for FunctionalGroupIdentifier {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        Ok((Self::from(buf[0]), &buf[1..]))
    }
}
```

(`DTCStoredDataRecordNumber` uses the lenient `From<u8>`, not `new` — decode must not reject reserved record numbers. `DTCSeverityMask` and `FunctionalGroupIdentifier` both have `From<u8>`.)

- [ ] **Step 4: Implement `ReadDTCInfoRequest::Decode` (25-variant inverse)**

In `src/services/read_dtc_information.rs`, after the `impl Encode for ReadDTCInfoRequest` block (relocated near the type in Phase 1), add:

```rust
impl<'a> Decode<'a> for ReadDTCInfoRequest {
    #[allow(clippy::too_many_lines)]
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        use ReadDTCInfoSubFunction as S;
        if buf.is_empty() {
            return Err(Error::InsufficientData(1));
        }
        let sub = buf[0];
        let rest = &buf[1..];
        let (dtc_subfunction, rest) = match sub {
            0x01 => {
                let (m, r) = DTCStatusMask::decode(rest)?;
                (S::ReportNumberOfDTC_ByStatusMask(m), r)
            }
            0x02 => {
                let (m, r) = DTCStatusMask::decode(rest)?;
                (S::ReportDTC_ByStatusMask(m), r)
            }
            0x03 => (S::ReportDTCSnapshotIdentification, rest),
            0x04 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                let (n, r) = DTCSnapshotRecordNumber::decode(r)?;
                (S::ReportDTCSnapshotRecord_ByDTCNumber(rec, n), r)
            }
            0x05 => {
                let (n, r) = DTCStoredDataRecordNumber::decode(rest)?;
                (S::ReportDTCStoredData_ByRecordNumber(n), r)
            }
            0x06 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                let (n, r) = DTCExtDataRecordNumber::decode(r)?;
                (S::ReportDTCExtDataRecord_ByDTCNumber(rec, n), r)
            }
            0x07 => {
                let (s, r) = DTCSeverityMask::decode(rest)?;
                let (m, r) = DTCStatusMask::decode(r)?;
                (S::ReportNumberOfDTC_BySeverityMaskRecord(s, m), r)
            }
            0x08 => {
                let (s, r) = DTCSeverityMask::decode(rest)?;
                let (m, r) = DTCStatusMask::decode(r)?;
                (S::ReportDTC_BySeverityMaskRecord(s, m), r)
            }
            0x09 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                (S::ReportSeverityInfoOfDTC(rec), r)
            }
            0x0A => (S::ReportSupportedDTC, rest),
            0x0B => (S::ReportFirstTestFailedDTC, rest),
            0x0C => (S::ReportFirstConfirmedDTC, rest),
            0x0D => (S::ReportMostRecentTestFailedDTC, rest),
            0x0E => (S::ReportMostRecentConfirmedDTC, rest),
            0x14 => (S::ReportDTCFaultDetectionCounter, rest),
            0x15 => (S::ReportDTCWithPermanentStatus, rest),
            0x16 => {
                let (n, r) = DTCExtDataRecordNumber::decode(rest)?;
                (S::ReportDTCExtDataRecord_ByRecordNumber(n), r)
            }
            0x17 => {
                let (m, r) = DTCStatusMask::decode(rest)?;
                (S::ReportUserDefMemoryDTC_ByStatusMask(m), r)
            }
            0x18 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                let (n, r) = DTCSnapshotRecordNumber::decode(r)?;
                let (mem, r) = u8::decode(r)?;
                (
                    S::ReportUserDefMemoryDTCSnapshotRecord_ByDTCNumber(rec, n, mem),
                    r,
                )
            }
            0x19 => {
                let (rec, r) = DTCRecord::decode(rest)?;
                let (n, r) = DTCExtDataRecordNumber::decode(r)?;
                let (mem, r) = u8::decode(r)?;
                (
                    S::ReportUserDefMemoryDTCExtDataRecord_ByDTCNumber(rec, n, mem),
                    r,
                )
            }
            0x1A => {
                let (n, r) = DTCExtDataRecordNumber::decode(rest)?;
                (S::ReportSupportedDTCExtDataRecord(n), r)
            }
            0x42 => {
                let (g, r) = FunctionalGroupIdentifier::decode(rest)?;
                let (m, r) = DTCStatusMask::decode(r)?;
                let (s, r) = DTCSeverityMask::decode(r)?;
                (S::ReportWWHOBDDTC_ByMaskRecord(g, m, s), r)
            }
            0x55 => {
                let (g, r) = FunctionalGroupIdentifier::decode(rest)?;
                (S::ReportWWHOBDDTC_WithPermanentStatus(g), r)
            }
            0x56 => {
                let (g, r) = FunctionalGroupIdentifier::decode(rest)?;
                let (rg, r) = u8::decode(r)?;
                (
                    S::ReportDTCInformation_ByDTCReadinessGroupIdentifier(g, rg),
                    r,
                )
            }
            other => (S::ISOSAEReserved(other), rest),
        };
        Ok((ReadDTCInfoRequest::new(dtc_subfunction), rest))
    }
}
```

- [ ] **Step 5: Wrap `Request::ReadDTCInfo`**

In `src/request.rs`:

- Add `ReadDTCInfoRequest` to the `services::{…}` import.
- Change the variant from `ReadDTCInfo(&'a [u8])` to `ReadDTCInfo(ReadDTCInfoRequest)` (no lifetime — it is owned); update its doc comment.
- `decode` arm:
  ```rust
  UdsServiceType::ReadDTCInfo => {
      Self::ReadDTCInfo(<ReadDTCInfoRequest as Decode>::decode_exact(payload)?)
  }
  ```
- `encoded_size`: remove `ReadDTCInfo` from the remaining grouped raw arm (now just `Self::ReadDataByIdentifier(bytes) => bytes.len()`) and add `Self::ReadDTCInfo(req) => req.encoded_size(),`.
- `encode`: split out similarly: `Self::ReadDTCInfo(req) => req.encode(writer)?,`.
- `service` arm unchanged.

After this, the only raw-bytes request variants remaining are `ReadDataByIdentifier(&[u8])` and `Other { data }` — as designed.

- [ ] **Step 6: Run tests, verify, commit**

Run: `cargo test --all-features && cargo clippy --all-features && cargo fmt`
Expected: PASS, including `read_dtc_info_request_roundtrips`.

```bash
git add -A
git commit -m "$(printf 'add Decode for DTC params and ReadDTCInfoRequest; wrap Request::ReadDTCInfo\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 7: Full-matrix verification

No code changes; if any command fails, fix the offending task and re-run.

- [ ] **Step 1: host build + test**

```bash
cargo build --all-features --release
cargo test --all-features
```

- [ ] **Step 2: no_std / no_alloc bare-metal**

```bash
cargo build --no-default-features --target thumbv6m-none-eabi
cargo build --no-default-features --features alloc --target thumbv6m-none-eabi
```

- [ ] **Step 3: clippy combos**

```bash
cargo clippy --all-features
cargo clippy --no-default-features
cargo clippy --no-default-features --features alloc
```

- [ ] **Step 4: fmt + doc + leftover-name sweep**

```bash
cargo fmt -- --check
cargo doc --release --all-features --no-deps
grep -rn "RequestTx\|ResponseTx\|ResponseRx\|PayloadTx" src README.md
```

Expected: all green; the grep shows only `ReadDataByIdentifierRequestTx`.

- [ ] **Step 5: (No commit)** — verification only. Phase 2 complete.

______________________________________________________________________

## Self-review notes

- **Spec coverage:** Decision 1 (naming) → Task 1; Decision 2 (wrapping) → Tasks 4–6 (WDBI, RoutineControl, ReadDTCInfo; RDBI deliberately left raw); Decision 3 (varint dedup) → Task 2; Decision 4 (macro merge) → Task 3; testing/matrix → Task 7.
- **SPRMIB fidelity** locked by `routine_control_request_roundtrips_with_suppress_bit` (Task 5).
- **`ReadDTCInfoRequest::Decode`** correctness guarded by encode-as-oracle round-trip (Task 6), reusing the Phase 1 `Encode`.
- **Type consistency:** all `Decode<'a>`/`Encode` signatures match `src/traits.rs`; param-type `Decode`s use the lenient `From<u8>`/`new` that round-trips each `Encode`; `read_be_uint`/`write_be_uint` are `pub(crate)` per the spec.
- **RDBI** remains the single documented raw exception (request `&[u8]`, `ReadDataByIdentifierRequestTx` keeps its suffix).
