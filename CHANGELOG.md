# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/) (treating
`0.x` breaking changes as minor bumps, per the Cargo/SemVer convention for
pre-1.0 crates).

## [Unreleased]

These changes require at least a 0.1.0 -> 0.2.0 bump before the next release.

### Changed

- **Breaking:** `Error::InsufficientData` now carries an `automotive_wire_codec::Incomplete`
  (with `needed` and `available` byte counts) instead of a bare `usize`.
- `automotive-wire-codec` is now a public dependency: its `Incomplete` and `TrailingBytes`
  types are re-exported at the crate root (`uds_protocol::{Incomplete, TrailingBytes}`) and
  are considered part of `uds_protocol`'s public API. A semver-major release of
  `automotive-wire-codec` is therefore a breaking change for `uds_protocol`.

### Added

- **Breaking:** New `Error::TrailingBytes` variant, produced when a decode leaves unconsumed
  bytes in the input. Both `Error::InsufficientData` and `Error::TrailingBytes` map to
  NRC `0x13` (`IncorrectMessageLengthOrInvalidFormat`).

### Removed

- The `byteorder-embedded-io` dependency, superseded by `automotive-wire-codec`.
