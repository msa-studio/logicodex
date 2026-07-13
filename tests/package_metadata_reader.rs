//! Behaviour-level tests for the P1-B7a read-only package metadata foundation.

use logicodex::package::{PackageDependency, PackageMetadata, PackageMetadataError};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_project(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after Unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "logicodex-package-metadata-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("temporary project directory must be created");
    dir
}

#[test]
fn parses_package_fields_and_plain_dependencies() {
    let metadata = PackageMetadata::from_toml_str(
        r#"
[package]
name = "demo"
version = "0.1.0"
edition = "2026"
root = "src/main.ldx"

[dependencies]
core-utils = "1.2.3"
vendor.math = "0.4"
"#,
    )
    .expect("valid package metadata must parse");

    assert_eq!(metadata.name, "demo");
    assert_eq!(metadata.version, "0.1.0");
    assert_eq!(metadata.edition.as_deref(), Some("2026"));
    assert_eq!(metadata.root.as_deref(), Some("src/main.ldx"));
    assert_eq!(
        metadata.dependencies.get("core-utils"),
        Some(&PackageDependency::Version("1.2.3".to_string()))
    );
    assert_eq!(
        metadata.dependencies.get("vendor.math"),
        Some(&PackageDependency::Version("0.4".to_string()))
    );
}

#[test]
fn discovers_manifest_next_to_source() {
    let dir = temp_project("discover");
    let source = dir.join("main.ldx");
    fs::write(&source, "PAPAR 1;\n").expect("source fixture must be written");
    fs::write(
        dir.join("logicodex.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .expect("manifest fixture must be written");

    let discovered = PackageMetadata::discover_for_source(&source)
        .expect("discovery must not fail")
        .expect("package metadata must be discovered");

    assert_eq!(discovered.0.name, "demo");
    assert_eq!(discovered.1, dir.join("logicodex.toml"));

    fs::remove_dir_all(dir).expect("temporary project directory must be removed");
}

#[test]
fn lod_only_manifest_is_not_package_metadata() {
    let dir = temp_project("lod-only");
    let source = dir.join("main.ldx");
    fs::write(&source, "PAPAR 1;\n").expect("source fixture must be written");
    fs::write(
        dir.join("logicodex.toml"),
        "[ffi]\nallow = [\"labs\"]\n\n[dependencies.c.libm]\nlink = \"m\"\n",
    )
    .expect("manifest fixture must be written");

    let discovered =
        PackageMetadata::discover_for_source(&source).expect("discovery must not fail");
    assert!(discovered.is_none());

    fs::remove_dir_all(dir).expect("temporary project directory must be removed");
}

#[test]
fn malformed_package_metadata_reports_manifest_path() {
    let dir = temp_project("malformed");
    let source = dir.join("main.ldx");
    fs::write(&source, "PAPAR 1;\n").expect("source fixture must be written");
    let manifest = dir.join("logicodex.toml");
    fs::write(&manifest, "[package]\nname = \"demo\"\n").expect("manifest fixture must be written");

    let error = PackageMetadata::discover_for_source(&source)
        .expect_err("missing package.version must fail");

    match error {
        PackageMetadataError::Parse(path, message) => {
            assert_eq!(path, manifest);
            assert!(message.contains("missing package.version"));
        }
        other => panic!("expected package metadata parse error, got {other}"),
    }

    fs::remove_dir_all(dir).expect("temporary project directory must be removed");
}
