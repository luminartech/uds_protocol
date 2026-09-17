# Security policy

## Reporting a vulnerability

Report security issues through GitHub's private vulnerability reporting: open
the [Security tab](https://github.com/luminartech/uds_protocol/security) and
choose **Report a vulnerability**. That opens a private advisory visible only
to the maintainers.

Please do not open a public issue for a security report.

A report is most useful with the crate version, the feature set enabled, and a
byte sequence or test case that reproduces the behavior.

## Supported versions

This crate is pre-1.0. Fixes land on the latest version published to
[crates.io](https://crates.io/crates/uds_protocol); there are no maintained
release branches, and older versions do not receive backported fixes. If you are
not on the latest version, please check whether it still reproduces there before
reporting.

The [Releases page](https://github.com/luminartech/uds_protocol/releases)
carries the same versions with their changelog entries — release-plz publishes
the crate and cuts the release together, so the two do not drift.

## Scope

`uds_protocol` encodes and decodes ISO 14229-1:2020 messages. It is a codec: it
does not own a transport, a session, or a key. Three properties of UDS matter
when assessing a report, because they are the specification's design rather
than defects in this crate:

- **UDS carries no transport security.** ISO 14229 defines a service layer.
  Confidentiality and integrity, where they exist at all, belong to the
  transport underneath it — DoIP, ISO-TP over CAN — and to the layers above.
  A UDS message read off the wire in the clear is the protocol working as
  specified.
- **`SecurityAccess` (0x27) is not authentication.** It is a seed/key unlock
  whose strength lives entirely in the ECU's key algorithm, which this crate
  neither generates nor validates — it encodes and decodes the exchange. The
  services ISO 14229-1:2020 defines for actual cryptographic protection,
  `Authentication` (0x29) and `SecuredDataTransmission` (0x84), are **not
  implemented here**. A system built on this crate has no authenticated
  diagnostic channel unless it provides one itself.
- **Several supported services write to the ECU by design.**
  `RequestDownload` (0x34), `RequestUpload` (0x35), `TransferData` (0x36) and
  `RequestFileTransfer` (0x38) exist to move memory and files. That this crate
  will encode and decode them is the point; deciding who may send them is the
  ECU's responsibility, not the codec's.

What is in scope: anything that makes the crate misbehave on attacker-supplied
bytes — a panic, an out-of-bounds read, an unbounded allocation, a decode that
accepts a frame it should reject, or a hang reachable from the wire. The fuzz
targets in `fuzz/fuzz_targets/` cover request decoding, response decoding and
round-tripping; a reproducer expressed as a fuzz input is ideal.
