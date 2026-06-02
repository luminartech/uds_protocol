# Unified Diagnostics Services (UDS) Protocol

This crate aims to offer an ergonomic implementation of the UDS protocol for tooling and test workloads in Rust.
Embedded support is an explicit, non-goal of this library.
It supports both serialization and deserialization of UDS both protocol messages as well as custom data types.
It is not in a complete state yet with the 0.1.0 release, please check back soon!

[![Crates.io](https://img.shields.io/crates/v/uds_protocol.svg?style=for-the-badge)](https://crates.io/crates/uds_protocol)
[![Docs.rs](https://img.shields.io/docsrs/uds_protocol?style=for-the-badge)](https://docs.rs/uds_protocol)
[![Codecov](https://img.shields.io/codecov/c/github/luminartech/uds_protocol?style=for-the-badge)](https://app.codecov.io/github/luminartech/uds_protocol)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](./LICENSE-MIT)
[![APACHE License](https://img.shields.io/badge/license-APACHE-blue.svg?style=for-the-badge)](./LICENSE-APACHE)

This library provides serialization and deserialization of UDS messages.
It is based on the ISO 14229-1:2020 standard.

| Service Name                          | Request SID | Response SID | Support |
|---------------------------------------|-------------|--------------|---------|
| `DiagnosticSessionControl`            | 0x10        | 0x50         | ✓       |
| `ECUReset`                            | 0x11        | 0x51         | ✓       |
| `ClearDiagnosticInformation`          | 0x14        | 0x54         | ✓       |
| `ReadDTCInformation`                  | 0x19        | 0x59         | Partial |
| `ReadDataByIdentifier`                | 0x22        | 0x62         | ✓       |
| `ReadMemoryByAddress`                 | 0x23        | 0x63         |         |
| `ReadScalingDataByIdentifier`         | 0x24        | 0x64         |         |
| `SecurityAccess`                      | 0x27        | 0x67         | ✓       |
| `CommunicationControl`                | 0x28        | 0x68         | ✓       |
| `Authentication`                      | 0x29        | 0x69         |         |
| `ReadDataByPeriodicIdentifier`        | 0x2A        | 0x6A         |         |
| `WriteDataByIdentifier`               | 0x2E        | 0x6E         | ✓       |
| `InputOutputControlByIdentifier`      | 0x2F        | 0x6F         |         |
| `RoutineControl`                      | 0x31        | 0x71         | ✓       |
| `RequestDownload`                     | 0x34        | 0x74         | ✓       |
| `RequestUpload`                       | 0x35        | 0x75         |         |
| `TransferData`                        | 0x36        | 0x76         | ✓       |
| `RequestTransferExit`                 | 0x37        | 0x77         | ✓       |
| `RequestFileTransfer`                 | 0x38        | 0x78         | ✓       |
| `WriteMemoryByAddress`                | 0x3D        | 0x7D         |         |
| `TesterPresent`                       | 0x3E        | 0x7E         | ✓       |
| `SecuredDataTransmission`             | 0x84        | 0xC4         |         |
| `ControlDTCSetting`                   | 0x85        | 0xC5         | ✓       |
| `ResponseOnEvent`                     | 0x86        | 0xC6         |         |
| `LinkControl`                         | 0x87        | 0xC7         |         |

## Integration

`uds_protocol` is a synchronous, allocation-free codec. It owns no sockets, buffers, or
async runtime. To use it over any transport (`DoIP`, `UDSonIP`, ISO-TP, …):

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

## Service coverage

These services decode into typed [`Request`]/[`Response`] variants: `DiagnosticSessionControl`,
`EcuReset`, `SecurityAccess`, `CommunicationControl`, `TesterPresent`, `ControlDTCSettings`,
`ReadDataByIdentifier`, `WriteDataByIdentifier`, `ClearDiagnosticInfo`, `ReadDTCInfo`,
`RoutineControl`, `RequestDownload`, `TransferData`, `RequestTransferExit`, `RequestFileTransfer`,
and `NegativeResponse`.

All other services enumerated in [`UdsServiceType`] (e.g. `Authentication`, `ReadMemoryByAddress`,
`RequestUpload`, `ResponseOnEvent`) are not individually modeled. Frames for them decode into
[`Request::Other`] / [`Response::Other`], carrying the service type and raw payload bytes for
pass-through.
