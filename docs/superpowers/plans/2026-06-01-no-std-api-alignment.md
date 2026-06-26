# no_std API Alignment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align the `uds_protocol` public API with its revised scope — a pure, synchronous, `no_std`/`no_alloc` UDS codec — before publishing, so breaking changes happen once.

**Architecture:** Remove the orphaned `DiagnosticDefinition` abstraction and all generic identifier machinery in favor of concrete types and raw payload byte slices. Make TX builders mirror the raw-bytes RX side, apply the Tx/Rx naming convention strictly, unify the unknown-service escape hatch into symmetric `Other` variants, and clean up the codec traits. Simplicity for C developers new to Rust is a first-class acceptance criterion.

**Tech Stack:** Rust 2024, `no_std`, `embedded-io` (sync `Write`), `byteorder-embedded-io`, `thiserror` (no_std). No async, no allocation in the baseline.

**Spec:** `docs/superpowers/specs/2026-06-01-no-std-api-alignment-design.md`

**Execute tasks in order.** Each task ends green (builds + tests pass) and is committed independently.

______________________________________________________________________

## File Structure

Files created:

- `src/test_util.rs` — `#[cfg(test)]` helper asserting `encode` length equals `encoded_size()`.

Files deleted:

- `src/protocol_definitions.rs` — `ProtocolIdentifier` / `ProtocolPayloadTx` / `ProtocolRoutinePayloadTx` are removed.

Files modified (primary responsibility):

- `src/traits.rs` — remove `Identifier`/`RoutineIdentifier`/`impl_identifier!`, blanket impls, `DiagnosticDefinition`, and `Encode::is_positive_response_suppressed`; expand `Decode` docs.
- `src/lib.rs` — register `test_util`; drop removed re-exports + `UdsSpec`; add service-coverage docs.
- `src/common/diagnostic_identifier.rs` — direct `Encode`/`Decode` for `UDSIdentifier`/`UDSRoutineIdentifier`.
- `src/services/read_data_by_identifier.rs` — `ReadDataByIdentifierRequestTx<'d>` over `&'d [u16]`.
- `src/services/write_data_by_identifier.rs` — `WriteDataByIdentifierRequestTx<'d>` + `WriteDataByIdentifierResponse { identifier: u16 }`.
- `src/services/routine_control.rs` — `RoutineControlRequestTx<'d>` / `RoutineControlResponseTx<'d>`.
- `src/services/control_dtc_settings.rs` — inherent `suppress_positive_response()`.
- `src/request.rs` / `src/response.rs` — `Other { service, data }`; remove `UdsResponse`; inherent suppression.
- `src/error.rs` — remove `ServiceNotImplemented`.
- `README.md` — integration + borrow-model docs.

______________________________________________________________________

## Task 1: Add the `encode`/`encoded_size` agreement test helper

Created first so every later task can assert the invariant directly.

**Files:**

- Create: `src/test_util.rs`

- Modify: `src/lib.rs`

- [ ] **Step 1: Create the helper module**

Create `src/test_util.rs`:

```rust
//! Test-only helpers shared across the crate.

use crate::Encode;

/// Assert that an [`Encode`] value writes exactly `encoded_size()` bytes.
///
/// Guards against the two methods drifting, which would corrupt callers that pre-size
/// a buffer from `encoded_size()`.
pub(crate) fn assert_encode_size_agrees<T: Encode>(value: &T) {
    let mut buf = [0u8; 512];
    let mut writer = buf.as_mut_slice();
    let written = value.encode(&mut writer).unwrap();
    assert_eq!(
        written,
        value.encoded_size(),
        "encode wrote {written} bytes but encoded_size() reported {}",
        value.encoded_size()
    );
}
```

- [ ] **Step 2: Register the module in `src/lib.rs`**

After the `mod error;` line (near the other `mod` declarations), add:

```rust
#[cfg(test)]
mod test_util;
```

- [ ] **Step 3: Build and test**

Run: `cargo build && cargo test`
Expected: PASS — the helper is `#[cfg(test)]` and unused so far (a dead-code warning is acceptable until Task 2 uses it; if `#![warn]` escalates it, this resolves once Task 2 lands).

- [ ] **Step 4: Commit**

```bash
git add src/test_util.rs src/lib.rs
git commit -m "$(printf 'add encode/encoded_size agreement test helper\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 2: De-genericize `ReadDataByIdentifierRequestTx` to `&[u16]`

**Files:**

- Modify: `src/services/read_data_by_identifier.rs`

- [ ] **Step 1: Replace the file with a concrete `&[u16]` version**

Replace the entire contents of `src/services/read_data_by_identifier.rs` with:

```rust
//! `ReadDataByIdentifier` (0x22) service implementation
use crate::{Encode, Error, NegativeResponseCode};

const READ_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ResponseTooLong,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
];

/// Zero-alloc TX request to read data by identifier. Borrows the DID list from the caller.
///
/// A Data Identifier is a 16-bit value, so the list is held as `&[u16]`; each DID is
/// written big-endian on the wire.
///
/// See ISO-14229-1:2020, Table 11.2.1 for format information
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadDataByIdentifierRequestTx<'d> {
    /// The list of Data Identifiers to read.
    pub dids: &'d [u16],
}

impl<'d> ReadDataByIdentifierRequestTx<'d> {
    /// Create a new request from a slice of data identifiers.
    #[must_use]
    pub const fn new(dids: &'d [u16]) -> Self {
        Self { dids }
    }

    /// Get the allowed Nack codes for this request
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &READ_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for ReadDataByIdentifierRequestTx<'_> {
    fn encoded_size(&self) -> usize {
        self.dids.len() * 2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        for did in self.dids {
            writer.write_all(&did.to_be_bytes()).map_err(Error::io)?;
        }
        Ok(self.encoded_size())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn encode_read_did_request_tx() {
        let ids = [0xF180u16, 0xF186u16];
        let req = ReadDataByIdentifierRequestTx::new(&ids);
        let mut buf = [0u8; 16];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 4); // 2 DIDs * 2 bytes each
        assert_eq!(&buf[..4], &[0xF1, 0x80, 0xF1, 0x86]);
        assert_encode_size_agrees(&req);
    }
}
```

- [ ] **Step 2: Build and test**

Run: `cargo build && cargo test read_data_by_identifier`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/services/read_data_by_identifier.rs
git commit -m "$(printf 'de-genericize ReadDataByIdentifierRequestTx to &[u16]\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 3: De-genericize `WriteDataByIdentifier` request + response

**Files:**

- Modify: `src/services/write_data_by_identifier.rs`

- [ ] **Step 1: Replace the file with concrete raw-bytes request + `u16` response**

Replace the entire contents of `src/services/write_data_by_identifier.rs` with:

```rust
//! `WriteDataByIdentifier` (0x2E) service implementation
use crate::{Encode, Error, NegativeResponseCode};

const WRITE_DID_NEGATIVE_RESPONSE_CODES: [NegativeResponseCode; 5] = [
    NegativeResponseCode::IncorrectMessageLengthOrInvalidFormat,
    NegativeResponseCode::ConditionsNotCorrect,
    NegativeResponseCode::RequestOutOfRange,
    NegativeResponseCode::SecurityAccessDenied,
    NegativeResponseCode::GeneralProgrammingFailure,
];

/// Zero-alloc TX request to write data by identifier. Borrows the raw payload from the caller.
///
/// The payload is the DID (2 bytes, big-endian) followed by the data record, exactly as
/// it appears on the wire after the service byte.
///
/// See ISO-14229-1:2020, Section 11.7.2.1
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WriteDataByIdentifierRequestTx<'d> {
    /// The raw payload bytes: DID followed by the data record.
    pub payload: &'d [u8],
}

impl<'d> WriteDataByIdentifierRequestTx<'d> {
    /// Create a new write-by-identifier request from raw payload bytes.
    #[must_use]
    pub const fn new(payload: &'d [u8]) -> Self {
        Self { payload }
    }

    /// Get the allowed [`NegativeResponseCode`] variants for this request.
    #[must_use]
    pub fn allowed_nack_codes() -> &'static [NegativeResponseCode] {
        &WRITE_DID_NEGATIVE_RESPONSE_CODES
    }
}

impl Encode for WriteDataByIdentifierRequestTx<'_> {
    fn encoded_size(&self) -> usize {
        self.payload.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer.write_all(self.payload).map_err(Error::io)?;
        Ok(self.payload.len())
    }
}

/// Positive response to `WriteDataByIdentifier`: echoes the DID that was written.
///
/// See ISO-14229-1:2020, Section 11.7.3.1
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct WriteDataByIdentifierResponse {
    /// The DID that was written to.
    pub identifier: u16,
}

impl WriteDataByIdentifierResponse {
    /// Create a new response echoing the identifier that was written.
    #[must_use]
    pub const fn new(identifier: u16) -> Self {
        Self { identifier }
    }
}

impl Encode for WriteDataByIdentifierResponse {
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&self.identifier.to_be_bytes())
            .map_err(Error::io)?;
        Ok(2)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn test_write_response_encode() {
        let response = WriteDataByIdentifierResponse::new(0xBEEF);
        let mut buf = [0u8; 4];
        let written = Encode::encode(&response, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 2);
        assert_eq!(buf[0], 0xBE);
        assert_eq!(buf[1], 0xEF);
        assert_encode_size_agrees(&response);
    }

    #[test]
    fn test_write_request_encode() {
        // DID 0xF186 + one data byte 0x01
        let payload = [0xF1, 0x86, 0x01];
        let request = WriteDataByIdentifierRequestTx::new(&payload);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&request, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(written, 3);
        assert_eq!(&buf[..3], &[0xF1, 0x86, 0x01]);
        assert_encode_size_agrees(&request);
    }
}
```

- [ ] **Step 2: Build and test**

Run: `cargo build && cargo test write_data_by_identifier`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add src/services/write_data_by_identifier.rs
git commit -m "$(printf 'de-genericize WriteDataByIdentifier to raw bytes + u16 response\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 4: De-genericize `RoutineControl` and rename to `...Tx`

**Files:**

- Modify: `src/services/routine_control.rs`

- Modify: `src/services/mod.rs` (only if it names the old types explicitly)

- [ ] **Step 1: Replace the file with concrete raw-bytes `...Tx` types**

Replace the entire contents of `src/services/routine_control.rs` with:

```rust
//! Routine Control (0x31) Service is used to perform functions on the ECU that may not be covered by other services.
//!
//! It can also be used to check the ECU's health, erase memory, or other custom manufacturer/supplier routines.
//! However, some routines may have side effects or require certain preconditions to be met.
use crate::{Encode, Error, RoutineControlSubFunction};

/// Used by a client to execute a defined sequence of events and obtain any relevant results.
///
/// The payload is the routine identifier (2 bytes, big-endian) followed by any optional
/// routine input parameters, exactly as it appears on the wire after the sub-function byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlRequestTx<'d> {
    /// The routine control operation (start, stop, or request results).
    pub sub_function: RoutineControlSubFunction,
    /// The raw payload bytes: routine identifier followed by optional parameters.
    pub raw_payload: &'d [u8],
}

impl<'d> RoutineControlRequestTx<'d> {
    /// Create a new `RoutineControlRequestTx`.
    #[must_use]
    pub const fn new(sub_function: RoutineControlSubFunction, raw_payload: &'d [u8]) -> Self {
        Self {
            sub_function,
            raw_payload,
        }
    }
}

impl Encode for RoutineControlRequestTx<'_> {
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

/// `RoutineControlResponseTx` is a variable-length response that can contain routine status.
///
/// The status record is the routine identifier echo plus any routine-info / status bytes,
/// held as raw bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct RoutineControlResponseTx<'d> {
    /// The sub-function echoed from the routine control request.
    pub routine_control_type: RoutineControlSubFunction,
    /// Raw routine status record bytes (routine identifier + routine info + status).
    pub raw_status_record: &'d [u8],
}

impl<'d> RoutineControlResponseTx<'d> {
    /// Create a new `RoutineControlResponseTx`.
    #[must_use]
    pub const fn new(
        routine_control_type: RoutineControlSubFunction,
        raw_status_record: &'d [u8],
    ) -> Self {
        Self {
            routine_control_type,
            raw_status_record,
        }
    }
}

impl Encode for RoutineControlResponseTx<'_> {
    fn encoded_size(&self) -> usize {
        1 + self.raw_status_record.len()
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&[u8::from(self.routine_control_type)])
            .map_err(Error::io)?;
        writer
            .write_all(self.raw_status_record)
            .map_err(Error::io)?;
        Ok(self.encoded_size())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn encode_routine_control_request_tx() {
        // RID 0xFF00 (EraseMemory) + 1 parameter byte
        let payload = [0xFF, 0x00, 0xAA];
        let req = RoutineControlRequestTx::new(RoutineControlSubFunction::StartRoutine, &payload);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x01, 0xFF, 0x00, 0xAA]);
        assert_encode_size_agrees(&req);
    }

    #[test]
    fn encode_routine_control_response_tx() {
        let record = [0xFF, 0x00, 0x10];
        let resp =
            RoutineControlResponseTx::new(RoutineControlSubFunction::StartRoutine, &record);
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &[0x01, 0xFF, 0x00, 0x10]);
        assert_encode_size_agrees(&resp);
    }
}
```

- [ ] **Step 2: Update `services/mod.rs` if it names the old types**

Run: `grep -n "RoutineControl" src/services/mod.rs`
If it re-exports explicit names (e.g. `pub use routine_control::{RoutineControlRequest, RoutineControlResponse};`), rename them to `RoutineControlRequestTx, RoutineControlResponseTx`. If it uses `pub use routine_control::*;`, no change is needed.

- [ ] **Step 3: Build and test**

Run: `cargo build && cargo test routine_control`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/services/routine_control.rs src/services/mod.rs
git commit -m "$(printf 'de-genericize RoutineControl to raw-bytes RoutineControl*Tx types\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 5: Remove `DiagnosticDefinition` and `UdsSpec`

**Files:**

- Modify: `src/traits.rs`

- Modify: `src/lib.rs`

- [ ] **Step 1: Delete the `DiagnosticDefinition` trait from `src/traits.rs`**

Remove this entire block (currently around lines 143–154):

```rust
/// Trait for diagnostic definitions that specifies the identifier and payload
/// types used when constructing and parsing UDS requests and responses.
pub trait DiagnosticDefinition<'a> {
    /// UDS Data Identifier type.
    type DID: Identifier + Clone + core::fmt::Debug + PartialEq + 'static;
    /// Payload type for read/write data by identifier etc.
    type DiagnosticPayload: Encode + Clone + core::fmt::Debug + PartialEq + 'a;
    /// UDS Routine Identifier type.
    type RID: RoutineIdentifier + Clone + core::fmt::Debug + PartialEq + 'static;
    /// Payload type for routine control requests/responses.
    type RoutinePayload: Encode + Clone + core::fmt::Debug + PartialEq + 'a;
}
```

- [ ] **Step 2: Delete `UdsSpec` and its impl from `src/lib.rs`**

Remove the `UdsSpec` struct (around lines 38–45) and its `impl<'a> DiagnosticDefinition<'a> for UdsSpec { ... }` block (around lines 47–52).

- [ ] **Step 3: Update the `traits` re-export in `src/lib.rs`**

Change:

```rust
pub use traits::{Decode, DecodeIter, DiagnosticDefinition, Encode, Identifier, RoutineIdentifier};
```

to:

```rust
pub use traits::{Decode, DecodeIter, Encode, Identifier, RoutineIdentifier};
```

(`Identifier`/`RoutineIdentifier` are removed in Task 7; leaving them here keeps this commit compiling.)

- [ ] **Step 4: Build and test**

Run: `cargo build && cargo test`
Expected: PASS — nothing consumed `DiagnosticDefinition` or `UdsSpec`.

- [ ] **Step 5: Commit**

```bash
git add src/traits.rs src/lib.rs
git commit -m "$(printf 'remove orphaned DiagnosticDefinition trait and UdsSpec\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 6: Delete `protocol_definitions.rs`

**Files:**

- Delete: `src/protocol_definitions.rs`

- Modify: `src/lib.rs`

- [ ] **Step 1: Confirm no remaining references outside the module itself**

Run: `grep -rn "ProtocolIdentifier\|ProtocolPayloadTx\|ProtocolRoutinePayloadTx\|protocol_definitions" src/ | grep -v "src/protocol_definitions.rs"`
Expected: only the two lines in `src/lib.rs` (the `mod` + `pub use`). Tasks 3–4 already removed the `ProtocolPayloadTx` test usages. If any other reference appears, stop and fix it first.

- [ ] **Step 2: Delete the module file**

Run: `git rm src/protocol_definitions.rs`

- [ ] **Step 3: Remove the module declaration and re-export from `src/lib.rs`**

Delete these two lines:

```rust
mod protocol_definitions;
pub use protocol_definitions::{ProtocolIdentifier, ProtocolPayloadTx, ProtocolRoutinePayloadTx};
```

- [ ] **Step 4: Build and test**

Run: `cargo build && cargo test`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add -A src/protocol_definitions.rs src/lib.rs
git commit -m "$(printf 'delete protocol_definitions module (ProtocolIdentifier/PayloadTx)\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 7: Remove identifier machinery; add direct codec to UDS identifiers

This task is atomic: the blanket `impl<T: Identifier>` and a direct `impl` for `UDSIdentifier` cannot coexist (coherence conflict), so the traits and direct impls must change together.

**Files:**

- Modify: `src/traits.rs`

- Modify: `src/common/diagnostic_identifier.rs`

- Modify: `src/lib.rs`

- [ ] **Step 1: Remove identifier traits, macro, and blanket impls from `src/traits.rs`**

Delete the `Identifier` trait, the `impl_identifier!` macro, the `RoutineIdentifier` trait, and all three blanket impls (`Encode`, `Decode`, `DecodeIter` for `T: Identifier`) — currently around lines 70–141. Also delete the `traits.rs` test module's `MyIdentifier` enum and its two identifier tests (around lines 156–222), since they depend on `impl_identifier!`. Keep the `Encode`, `Decode`, `DecodeIter` trait definitions.

- [ ] **Step 2: Add direct `Encode`/`Decode` impls for the UDS identifier enums**

In `src/common/diagnostic_identifier.rs`, change the import line:

```rust
use crate::{Error, impl_identifier, traits::RoutineIdentifier};
```

to:

```rust
use crate::{Decode, Encode, Error};
```

Delete the two `impl_identifier!(UDSIdentifier);` / `impl_identifier!(UDSRoutineIdentifier);` lines and the `impl RoutineIdentifier for UDSRoutineIdentifier {}` line.

Add, at the end of the file:

```rust
impl Encode for UDSIdentifier {
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&u16::from(*self).to_be_bytes())
            .map_err(Error::io)?;
        Ok(2)
    }
}

impl<'a> Decode<'a> for UDSIdentifier {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::IncorrectMessageLengthOrInvalidFormat);
        }
        let raw = u16::from_be_bytes([buf[0], buf[1]]);
        Ok((Self::try_from(raw)?, &buf[2..]))
    }
}

impl Encode for UDSRoutineIdentifier {
    fn encoded_size(&self) -> usize {
        2
    }

    fn encode(&self, writer: &mut impl embedded_io::Write) -> Result<usize, Error> {
        writer
            .write_all(&u16::from(*self).to_be_bytes())
            .map_err(Error::io)?;
        Ok(2)
    }
}

impl<'a> Decode<'a> for UDSRoutineIdentifier {
    fn decode(buf: &'a [u8]) -> Result<(Self, &'a [u8]), Error> {
        if buf.len() < 2 {
            return Err(Error::IncorrectMessageLengthOrInvalidFormat);
        }
        let raw = u16::from_be_bytes([buf[0], buf[1]]);
        Ok((Self::from(raw), &buf[2..]))
    }
}

#[cfg(test)]
mod codec_tests {
    use super::*;
    use crate::test_util::assert_encode_size_agrees;

    #[test]
    fn uds_identifier_roundtrip() {
        let id = UDSIdentifier::ActiveDiagnosticSession;
        let mut buf = [0u8; 2];
        Encode::encode(&id, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(buf, [0xF1, 0x86]);
        let (decoded, rest) = <UDSIdentifier as Decode>::decode(&buf).unwrap();
        assert_eq!(decoded, id);
        assert!(rest.is_empty());
        assert_encode_size_agrees(&id);
    }

    #[test]
    fn uds_routine_identifier_roundtrip() {
        let id = UDSRoutineIdentifier::EraseMemory;
        let mut buf = [0u8; 2];
        Encode::encode(&id, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(buf, [0xFF, 0x00]);
        let (decoded, rest) = <UDSRoutineIdentifier as Decode>::decode(&buf).unwrap();
        assert_eq!(decoded, id);
        assert!(rest.is_empty());
        assert_encode_size_agrees(&id);
    }
}
```

- [ ] **Step 3: Update the `traits` re-export in `src/lib.rs`**

Change:

```rust
pub use traits::{Decode, DecodeIter, Encode, Identifier, RoutineIdentifier};
```

to:

```rust
pub use traits::{Decode, DecodeIter, Encode};
```

- [ ] **Step 4: Build and test**

Run: `cargo build && cargo test diagnostic_identifier`
Expected: PASS. Then `cargo build` (full crate) to confirm nothing else referenced the removed traits.

- [ ] **Step 5: Commit**

```bash
git add src/traits.rs src/common/diagnostic_identifier.rs src/lib.rs
git commit -m "$(printf 'remove Identifier machinery; add direct codec to UDS identifiers\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 8: Move `is_positive_response_suppressed` off the `Encode` trait

**Files:**

- Modify: `src/traits.rs`

- Modify: `src/services/ecu_reset.rs`, `control_dtc_settings.rs`, `tester_present.rs`, `security_access.rs`, `diagnostic_session_control.rs`

- Modify: `src/request.rs`

- [ ] **Step 1: Remove the method from the `Encode` trait**

In `src/traits.rs`, delete from the `Encode` trait:

```rust
    /// Whether the positive response for this message is suppressed (SPRMIB).
    fn is_positive_response_suppressed(&self) -> bool {
        false
    }
```

- [ ] **Step 2: Remove the trait-method overrides from each service `Encode` impl**

In each of `ecu_reset.rs`, `control_dtc_settings.rs`, `tester_present.rs`, `security_access.rs`, `diagnostic_session_control.rs`, delete the `fn is_positive_response_suppressed(&self) -> bool { ... }` block from inside its `impl Encode for ...` block. Confirm with: `grep -rn "fn is_positive_response_suppressed" src/services/` — expect no matches afterward.

- [ ] **Step 3: Add an inherent `suppress_positive_response()` to `ControlDTCSettingsRequest`**

`ControlDTCSettingsRequest` exposes a public `suppress_response` field but no getter matching the other services. In `src/services/control_dtc_settings.rs`, add to its inherent `impl ControlDTCSettingsRequest` block:

```rust
    /// Whether the server should suppress the positive response (SPRMIB).
    #[must_use]
    pub const fn suppress_positive_response(&self) -> bool {
        self.suppress_response
    }
```

(ecu_reset, tester_present, security_access, diagnostic_session_control, and communication_control already have inherent `suppress_positive_response()` getters.)

- [ ] **Step 4: Replace the `Request` trait override with an inherent method**

In `src/request.rs`, remove the `fn is_positive_response_suppressed(&self) -> bool { ... }` block from the `impl Encode for Request<'_>` block, and add this inherent method to the `impl Request<'_>` block (the one that also defines `service`):

```rust
    /// Whether the positive response for this request is suppressed (SPRMIB).
    #[must_use]
    pub fn is_positive_response_suppressed(&self) -> bool {
        match self {
            Self::CommunicationControl(req) => req.suppress_positive_response(),
            Self::ControlDTCSettings(req) => req.suppress_positive_response(),
            Self::DiagnosticSessionControl(req) => req.suppress_positive_response(),
            Self::EcuReset(req) => req.suppress_positive_response(),
            Self::SecurityAccess(req) => req.suppress_positive_response(),
            Self::TesterPresent(req) => req.suppress_positive_response(),
            _ => false,
        }
    }
```

The existing `suppression_forwards_to_inner_request` test calls `.is_positive_response_suppressed()` and now resolves to this inherent method, so it is unchanged.

- [ ] **Step 5: Build and test**

Run: `cargo build && cargo test`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/traits.rs src/request.rs src/services/
git commit -m "$(printf 'move is_positive_response_suppressed off the Encode trait\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 9: Add symmetric `Other` escape hatch; remove `UdsResponse` and `ServiceNotImplemented`

**Files:**

- Modify: `src/request.rs`

- Modify: `src/response.rs`

- Modify: `src/error.rs`

- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing tests for the `Other` variants**

In `src/request.rs` `mod tests`, add:

```rust
    #[test]
    fn unmodeled_service_decodes_to_other() {
        // 0x23 = ReadMemoryByAddress, enumerated but not modeled.
        let frame = [0x23, 0xAA, 0xBB];
        let (req, rest) = Request::decode(&frame).unwrap();
        assert!(rest.is_empty());
        match req {
            Request::Other { service, data } => {
                assert_eq!(service, UdsServiceType::ReadMemoryByAddress);
                assert_eq!(data, &[0xAA, 0xBB]);
            }
            other => panic!("expected Other, got {other:?}"),
        }
        let mut buf = [0u8; 8];
        let written = Encode::encode(&req, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &frame);
    }
```

In `src/response.rs`, add a test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmodeled_response_decodes_to_other() {
        // 0x63 = ReadMemoryByAddress positive response, not modeled.
        let frame = [0x63, 0x01, 0x02];
        let (resp, rest) = Response::decode(&frame).unwrap();
        assert!(rest.is_empty());
        match resp {
            Response::Other { service, data } => {
                assert_eq!(service, UdsServiceType::ReadMemoryByAddress);
                assert_eq!(data, &[0x01, 0x02]);
            }
            other => panic!("expected Other, got {other:?}"),
        }
        let mut buf = [0u8; 8];
        let written = Encode::encode(&resp, &mut buf.as_mut_slice()).unwrap();
        assert_eq!(&buf[..written], &frame);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test unmodeled`
Expected: FAIL — no `Other` variant exists.

- [ ] **Step 3: Add `Other` to `Request<'a>`**

In `src/request.rs`, add to the `Request<'a>` enum:

```rust
    /// A known-but-unmodeled (or unrecognized) service. Carries the service type and
    /// the raw payload bytes following the service identifier, for pass-through.
    Other {
        /// The service this frame addresses.
        service: UdsServiceType,
        /// Raw payload bytes after the service byte.
        data: &'a [u8],
    },
```

In `Request::decode`, replace `_ => return Err(Error::ServiceNotImplemented(service)),` with:

```rust
            _ => Self::Other {
                service,
                data: payload,
            },
```

In `Request::encoded_size`, add to the payload match: `Self::Other { data, .. } => data.len(),`

In `Request::encode`, add to the payload match:

```rust
            Self::Other { data, .. } => {
                writer.write_all(data).map_err(Error::io)?;
                data.len()
            }
```

In `Request::service`, add: `Self::Other { service, .. } => *service,`

- [ ] **Step 4: Add `Other` to `Response<'a>`; remove `UdsResponse`**

In `src/response.rs`, add to the `Response<'a>` enum:

```rust
    /// A known-but-unmodeled (or unrecognized) service response. Carries the service
    /// type and the raw payload bytes following the service identifier.
    Other {
        /// The service this response addresses.
        service: UdsServiceType,
        /// Raw payload bytes after the service byte.
        data: &'a [u8],
    },
```

In `Response::decode`, replace `_ => return Err(Error::ServiceNotImplemented(service)),` with:

```rust
            _ => Self::Other {
                service,
                data: payload,
            },
```

In `Response::response_sid`, add: `Self::Other { service, .. } => service.response_to_byte(),`

In `Response::encoded_size`, add: `Self::Other { data, .. } => data.len(),`

In `Response::encode`, add to the payload match:

```rust
            Self::Other { data, .. } => {
                writer.write_all(data).map_err(Error::io)?;
                data.len()
            }
```

Delete the entire `UdsResponse<'a>` struct and its `impl<'a> Decode<'a> for UdsResponse<'a>` (currently lines ~206–228).

- [ ] **Step 5: Remove the `UdsResponse` re-export from `src/lib.rs`**

Change `pub use response::{Response, UdsResponse};` to `pub use response::Response;`

- [ ] **Step 6: Remove `ServiceNotImplemented` from `src/error.rs`**

Delete:

```rust
    /// The service type is not yet implemented in this crate.
    #[error("UDS service not implemented: {0:?}")]
    ServiceNotImplemented(crate::UdsServiceType),
```

- [ ] **Step 7: Run tests**

Run: `cargo test`
Expected: PASS, including the new `unmodeled` tests. Confirm no stragglers: `grep -rn "UdsResponse\|ServiceNotImplemented" src/` — expect no matches.

- [ ] **Step 8: Commit**

```bash
git add src/request.rs src/response.rs src/error.rs src/lib.rs
git commit -m "$(printf 'add symmetric Other escape hatch; drop UdsResponse + ServiceNotImplemented\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 10: Document the `Decode` remainder / borrow contract

**Files:**

- Modify: `src/traits.rs`

- [ ] **Step 1: Expand the `Decode` trait doc comment**

Replace the existing `Decode` trait doc comment (the `/// RX-side trait: zero-copy decode ...` block above `pub trait Decode<'a>`) with:

```rust
/// RX-side trait: zero-copy decode from a byte slice.
///
/// Implementations borrow directly from the input buffer where possible. The decoded
/// value points into `buf` and is valid only as long as `buf` lives — for C developers
/// new to Rust, think of it like a `struct` overlaid on a `char buf[]`. Copy out any
/// fields you need to retain beyond the buffer's lifetime.
///
/// [`decode`](Self::decode) returns the value together with the unconsumed remainder of
/// the buffer, so leaf and sequence decoders can be composed. Frame-level decoders
/// (`Request`, `Response`) consume the whole buffer and return an empty remainder; use
/// [`decode_exact`](Self::decode_exact) when a buffer must contain exactly one value.
```

- [ ] **Step 2: Build docs**

Run: `cargo build && cargo doc --no-deps`
Expected: PASS, no rustdoc warnings.

- [ ] **Step 3: Commit**

```bash
git add src/traits.rs
git commit -m "$(printf 'document the Decode remainder / borrow contract\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 11: README integration + borrow-model docs

**Files:**

- Modify: `README.md`

- [ ] **Step 1: Append an Integration section to `README.md`**

Add at the end of `README.md`:

````markdown
## Integration

`uds_protocol` is a synchronous, allocation-free codec. It owns no sockets, buffers, or
async runtime. To use it over any transport (DoIP, UDSonIP, ISO-TP, …):

- **Decode** an inbound frame from the `&[u8]` you received.
- **Encode** an outbound frame into any `embedded_io::Write` (or a caller-owned buffer
  sized with `encoded_size()`).

Drive the I/O loop from your own sync or async layer — the crate never blocks or awaits.

### Encode (build a request)

```rust
use uds_protocol::{Encode, TesterPresentRequest};

let req = TesterPresentRequest::new(false);
let mut buf = [0u8; 8];
let mut writer = buf.as_mut_slice();
let written = Encode::encode(&req, &mut writer).unwrap();
// `buf[..written]` is the wire frame, ready to hand to your transport.
```

### Decode (parse a response)

```rust
use uds_protocol::{Decode, Response};

// `frame` is the &[u8] your transport handed you.
let frame = [0x7E, 0x00];
let (response, _rest) = Response::decode(&frame).unwrap();
```

The decoded value **borrows** from `frame`: it points into that buffer (like a `struct`
overlaid on a `char buf[]`) and is valid only while `frame` lives. Copy out any fields
you need to keep before the buffer is reused.
````

- [ ] **Step 2: Verify the doctests compile (README is included as crate docs)**

Run: `cargo test --doc`
Expected: PASS — `src/lib.rs` includes `README.md` via `#![doc = include_str!(...)]`, so the snippets run as doctests. If a referenced symbol (e.g. `TesterPresentRequest`) is not exported at the crate root, adjust the snippet's import path to match the actual re-export.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "$(printf 'document runtime-agnostic integration model and borrow semantics\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 12: Apply the agreement helper to the remaining service tests

**Files:**

- Modify: `src/services/ecu_reset.rs`, `tester_present.rs`, `communication_control.rs`, `diagnostic_session_control.rs`, `clear_dtc_information.rs`, `security_access.rs`, `request_download.rs`, `transfer_data.rs`, `request_file_transfer.rs`, `control_dtc_settings.rs`, `negative_response.rs`

- [ ] **Step 1: Add an agreement assertion to each service's existing encode test**

For each module above: add `use crate::test_util::assert_encode_size_agrees;` to the test module's imports, and append `assert_encode_size_agrees(&<value>);` to the existing test that builds and encodes a value (reuse the request/response value already constructed there). Example for `ecu_reset.rs`:

```rust
    use crate::test_util::assert_encode_size_agrees;
    // ...inside the existing encode test, after `request` is built:
    assert_encode_size_agrees(&request);
```

If a module has no encode test that builds a value, add a minimal one using that file's `new` constructor and an appropriately sized stack buffer.

- [ ] **Step 2: Test**

Run: `cargo test`
Expected: PASS. A failure here is a real `encode`/`encoded_size` mismatch — fix the offending `encoded_size` to match the bytes `encode` writes, then re-run.

- [ ] **Step 3: Commit**

```bash
git add src/services/
git commit -m "$(printf 'assert encode/encoded_size agreement across all services\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 13: Document the service-coverage boundary

**Files:**

- Modify: `src/lib.rs`

- [ ] **Step 1: Add a crate-root doc block listing modeled vs pass-through services**

In `src/lib.rs`, add this doc comment immediately above the `pub const SUCCESS: u8 = 0x80;` declaration:

```rust
/// ## Service coverage
///
/// These services decode into typed [`Request`]/[`Response`] variants:
/// `DiagnosticSessionControl`, `EcuReset`, `SecurityAccess`, `CommunicationControl`,
/// `TesterPresent`, `ControlDTCSettings`, `ReadDataByIdentifier`, `WriteDataByIdentifier`,
/// `ClearDiagnosticInfo`, `ReadDTCInfo`, `RoutineControl`, `RequestDownload`,
/// `TransferData`, `RequestTransferExit`, `RequestFileTransfer`, and `NegativeResponse`.
///
/// All other services enumerated in [`UdsServiceType`] (e.g. `Authentication`,
/// `ReadMemoryByAddress`, `RequestUpload`, `ResponseOnEvent`) are not individually
/// modeled. Frames for them decode into [`Request::Other`] / [`Response::Other`],
/// carrying the service type and raw payload bytes for pass-through.
```

- [ ] **Step 2: Build docs**

Run: `cargo doc --no-deps`
Expected: PASS, no warnings, no broken intra-doc links.

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "$(printf 'document the modeled vs pass-through service coverage boundary\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Task 14: Full verification across the feature matrix

**Files:** none (verification only)

- [ ] **Step 1: Host builds across feature combos**

```bash
cargo build
cargo build --no-default-features --features alloc
cargo build --no-default-features
```

Expected: all succeed.

- [ ] **Step 2: Tests**

Run: `cargo test`
Expected: all pass (pre-refactor baseline was 87; expect that plus the new `Other`, identifier-codec, and size-agreement tests).

- [ ] **Step 3: Clippy across host combos (crate sets `#![warn(clippy::pedantic, missing_docs)]`)**

```bash
cargo clippy --all-targets
cargo clippy --no-default-features --features alloc --all-targets
cargo clippy --no-default-features --all-targets
```

Expected: zero warnings.

- [ ] **Step 4: Bare-metal target**

```bash
cargo build --no-default-features --target thumbv6m-none-eabi
cargo build --no-default-features --features alloc --target thumbv6m-none-eabi
```

Expected: success. (If missing: `rustup target add thumbv6m-none-eabi`.)

- [ ] **Step 5: Confirm no orphaned references remain**

Run: `grep -rn "DiagnosticDefinition\|UdsSpec\|ProtocolIdentifier\|ProtocolPayloadTx\|impl_identifier\|ServiceNotImplemented\|UdsResponse\|: Identifier\|: RoutineIdentifier" src/`
Expected: no matches.

- [ ] **Step 6: Commit any verification fixes**

If steps 1–5 required changes (clippy fixes, doc tweaks), commit them; otherwise nothing to commit:

```bash
git add -A
git commit -m "$(printf 'verification fixes across the no_std feature matrix\n\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>')"
```

______________________________________________________________________

## Self-Review Notes

**Spec coverage:** Decision 1 → Task 5; Decision 2 → Tasks 6–7; Decision 3 → Tasks 2–4; Decision 4 (naming) → Task 4 (RoutineControl rename) + Task 3 (WriteDataByIdentifierRequestTx), already-conforming types unchanged; Decision 5 → Task 9; Decision 6 → Task 8; Decision 7 → Tasks 1 + 12; Decision 8 → Task 10; Decision 9 → Task 11; Decision 10 → Task 13. Open item (`ServiceNotImplemented` fate) resolved in Task 9: removed, fully subsumed by `Other`.

**Type consistency:** `assert_encode_size_agrees` (Task 1) is used identically everywhere. New public names — `ReadDataByIdentifierRequestTx<'d>` (`&[u16]`), `WriteDataByIdentifierRequestTx<'d>` / `WriteDataByIdentifierResponse { identifier: u16 }`, `RoutineControlRequestTx<'d>` / `RoutineControlResponseTx<'d>`, `Request::Other { service, data }` / `Response::Other { service, data }` — are referenced consistently across tasks.

**Verification:** Task 14 enforces the same matrix the existing CI uses (three host feature combos + `thumbv6m-none-eabi` + clippy), so the refactor lands against the project's real definition of green.
