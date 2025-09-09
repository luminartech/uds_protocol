# Unified Diagnostics Services (UDS) Protocol

This crate aims to offer an ergonomic implementation of the UDS protocol in Rust.
It suppports both serialization and deserialization of UDS both protocol messages as well as custom data types.
It is not in a complete state yet with the 0.1.0 release, please check back soon!

[![Crates.io](https://img.shields.io/crates/v/uds_protocol.svg?style=for-the-badge)](https://crates.io/crates/uds_protocol)
[![Docs.rs](https://img.shields.io/docsrs/uds_protocol?style=for-the-badge)](https://docs.rs/uds_protocol)
[![Codecov](https://img.shields.io/codecov/c/github/luminartech/uds_protocol?style=for-the-badge)](https://app.codecov.io/github/luminartech/uds_protocol)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](./LICENSE-MIT)
[![APACHE License](https://img.shields.io/badge/license-APACHE-blue.svg?style=for-the-badge)](./LICENSE-APACHE)

This library provides serialization and deserialization of UDS messages.
It is based on the ISO 14229-1:2020 standard.

| Service Name                      | Request SID | Response SID | Support |
|-----------------------------------|-------------|--------------|---------|
| DiagnosticSessionControl          | 0x10        | 0x50         | ✓       |
| ECUReset                          | 0x11        | 0x51         | ✓       |
| ClearDiagnosticInformation        | 0x14        | 0x54         | ✓       |
| ReadDTCInformation                | 0x19        | 0x59         | Partial |
| ReadDataByIdentifier              | 0x22        | 0x62         | ✓       |
| ReadMemoryByAddress               | 0x23        | 0x63         |         |
| ReadScalingDataByIdentifier       | 0x24        | 0x64         |         |
| SecurityAccess                    | 0x27        | 0x67         | ✓       |
| CommunicationControl              | 0x28        | 0x68         | ✓       |
| Authentication                    | 0x29        | 0x69         |         |
| ReadDataByPeriodicIdentifier      | 0x2A        | 0x6A         |         |
| WriteDataByIdentifier             | 0x2E        | 0x6E         | ✓       |
| InputOutputControlByIdentifier    | 0x2F        | 0x6F         |         |
| RoutineControl                    | 0x31        | 0x71         | ✓       |
| RequestDownload                   | 0x34        | 0x74         | ✓       |
| RequestUpload                     | 0x35        | 0x75         |         |
| TransferData                      | 0x36        | 0x76         | ✓       |
| RequestTransferExit               | 0x37        | 0x77         | ✓       |
| RequestFileTransfer               | 0x38        | 0x78         | ✓       |
| WriteMemoryByAddress              | 0x3D        | 0x7D         |         |
| TesterPresent                     | 0x3E        | 0x7E         | ✓       |
| SecuredDataTransmission           | 0x84        | 0xC4         |         |
| ControlDTCSetting                 | 0x85        | 0xC5         | ✓       |
| ResponseOnEvent                   | 0x86        | 0xC6         |         |
| LinkControl                       | 0x87        | 0xC7         |         |
