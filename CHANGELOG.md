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
- **Breaking:** `param_length_u16`/`param_length_u32`/`param_length_u64`/`param_length_u128` have
  been removed. Use `automotive_wire_codec::minimal_be_len` instead.
- **Breaking:** `uds_protocol`'s `Encode`/`Decode` implementations for the primitive numeric types
  (`u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64`) have been removed. Now that
  `Encode`/`Decode` are re-exports of `automotive-wire-codec`'s traits, implementing them on
  foreign primitive types would violate Rust's orphan rule — the codec intentionally ships no
  blanket impls for primitives. Callers who relied on `Encode`/`Decode` for a bare primitive
  should switch to the leaf helpers (`read_u16_be`/`write_u16_be`/etc., or
  `read_be_uint`/`write_be_uint`/`read_be_uint_into` for variable-width fields) re-exported from
  `automotive_wire_codec`.

### Changed (migration to `automotive-wire-codec` 0.3)

- **Breaking:** `uds_protocol::{Decode, DecodeIter, Encode}` are now re-exports of the
  `automotive-wire-codec` 0.3 traits (previously crate-local traits). This is the underlying
  cause of most of the other breaking changes in this release.
- **Breaking:** `Encode::encoded_size` is now `Result<usize, Self::Error>` (previously infallible
  `usize`), via the codec trait's correct-by-construction counting-sink default. Crate-local
  `encoded_size` overrides have been removed; callers must handle/unwrap the `Result`.
- **Breaking:** Added `Error::InvalidWidth`, produced when a wire-declared variable-width field
  requests a byte width the target type cannot hold. The underlying `automotive_wire_codec::InvalidWidth`
  fragment is also re-exported at the crate root (alongside `Incomplete` and `TrailingBytes`).
- **Breaking:** `decode_exact` trailing-bytes now surface as `Error::TrailingBytes` instead of
  `Error::IncorrectMessageLengthOrInvalidFormat` (both still map to NRC 0x13).
- This release remains a semver-major bump.
