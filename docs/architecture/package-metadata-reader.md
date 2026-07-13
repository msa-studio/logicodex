# P1-B7a Package Metadata Reader Foundation

## Status

P1-B7a provides a read-only package metadata reader for `logicodex.toml`.

It is intentionally separate from `lod`:

- `lod` owns `[ffi]` and `[dependencies.c.*]` for C ABI linking and capability allow-lists.
- `package` owns `[package]` and plain `[dependencies]` metadata.

This foundation is not a package manager. It does not download packages, resolve version ranges, create lockfiles, access a registry, or change module resolution.

## Supported metadata

The `[package]` table requires:

- `name`
- `version`

It may also define:

- `edition`
- `root`

The plain `[dependencies]` table accepts dependency names mapped to quoted version strings.

Example:

```toml
[package]
name = "demo"
version = "0.1.0"
edition = "2021"
root = "src/main.ldx"

[dependencies]
math-utils = "1.2.0"
```

## Discovery rule

`PackageMetadata::discover_for_source` looks for `logicodex.toml` beside the supplied `.ldx` source.

- Missing manifest: `Ok(None)`.
- Manifest without `[package]`: `Ok(None)`.
- Manifest with valid package metadata: parsed metadata and manifest path.
- Manifest with invalid package metadata: explicit read or parse error carrying the manifest path.

## Boundary and compatibility

The reader ignores unrelated tables, including existing `lod` tables such as `[ffi]` and `[dependencies.c.*]`. This permits one root manifest to carry both package metadata and FFI/link configuration without merging their authorities.

Unknown keys inside `[package]`, duplicate required fields, malformed strings, invalid dependency names, missing required fields, and duplicate dependencies fail explicitly.

## Proof

The foundation is covered by:

- unit tests in `src/package.rs`
- behaviour-level integration tests in `tests/package_metadata_reader.rs`
- normal formatting, compile, full test, stdlib contract, and CPB runway gates

Future package-manager work must build on this reader rather than widening it silently.
