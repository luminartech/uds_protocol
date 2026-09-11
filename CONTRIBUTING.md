# Contributing

Pull requests are welcome, as are bug reports and questions in the issue
tracker.

The crate implements ISO 14229-1:2020. The service table in
[`README.md`](README.md) tracks which services are supported, partially
supported, or not yet started — it is the place to look before adding one.

## Building and testing

The minimum supported Rust version is **1.85.0**, declared as `rust-version` in
`Cargo.toml`.

`default = ["std"]`, and the feature graph is layered: `std` implies `alloc`,
and both `utoipa` and `clap` imply `std` because their derive macros expand to
`std::`, `String` and `Vec` paths. `utoipa` additionally implies `serde`,
because a `ToSchema` describes this crate's *serde* representation. `serde` is
the only optional integration usable on a bare-metal target.

```sh
cargo test                                      # default: std
cargo test --all-features                       # everything
cargo test --no-default-features                # the no_std core alone
cargo test --no-default-features --features alloc,serde
```

The `no_std` core must keep building for a bare-metal target:

```sh
rustup target add thumbv7em-none-eabihf
cargo check --no-default-features --target thumbv7em-none-eabihf
```

The integration tests each guard a different property:

- `tests/spec_conformance.rs` — encodings against ISO 14229-1:2020.
- `tests/public_api.rs` — the public surface, so that additions to it are
  deliberate rather than incidental.
- `tests/openapi_schema.rs` — the `utoipa` schemas, which must describe the
  serde representation rather than the Rust shape.

## Wire-format changes

Several types serialize as a single protocol byte rather than as their Rust
shape. A round-trip test written against the crate's own output passes whenever
`encode` and `decode` share the same misreading, so a change that alters the
bytes on the wire should be pinned by a case in `tests/spec_conformance.rs`
with the specification's bytes written out literally.

The fuzz targets in `fuzz/fuzz_targets/` cover request decoding, response
decoding, and round-tripping. A decode bug is best reported as a fuzz input.

## Commits and pull requests

Commit subjects follow [Conventional Commits](https://www.conventionalcommits.org/)
(`feat:`, `fix:`, `docs:`, `build:`, `chore:`, with a `!` for a breaking
change), because the changelog is organized around them. Say what changed and
why in the body.

This repository merges with **merge commits** — squash and rebase are both
disabled — so the subjects of the commits on your branch are what land on
`main`, and those are what release-plz reads to build the changelog and compute
the next version. Keep them clean; a tidy PR title does not stand in for them
here. CI additionally lints the PR title for conventional-commit grammar.

Installing the hooks checks the same grammar locally, before the push rather
than in CI:

```sh
pre-commit install --install-hooks
```

## Releases

[release-plz](https://release-plz.dev) owns versioning, the changelog, tags,
GitHub releases, and the crates.io publish. There is no release workflow in
this repository and no version to bump by hand: a push to `main` maintains an
open release PR, and merging that PR publishes. `release-plz.toml` holds the
configuration, shared with `simple_doip` and `automotive_wire_codec`.

## Continuous integration

CI is a thin caller for the org-wide reusable workflow in
[`luminartech/rust_workflow`](https://github.com/luminartech/rust_workflow);
`.github/workflows/main.yml` owns only the triggers and this crate's
configuration. Changes to the jobs themselves belong in that repository.
