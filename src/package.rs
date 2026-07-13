//! P1-B7a read-only package metadata reader.
//!
//! Separate from `lod`:
//! - `lod` owns `[ffi]` and `[dependencies.c.*]` for C ABI / FFI linking.
//! - `package` owns `[package]` and plain `[dependencies]` metadata.
//!
//! This is not a package manager.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const PACKAGE_MANIFEST_FILENAME: &str = "logicodex.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub edition: Option<String>,
    pub root: Option<String>,
    pub dependencies: BTreeMap<String, PackageDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageDependency {
    Version(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl ParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
pub enum PackageMetadataError {
    Read(PathBuf, String),
    Parse(PathBuf, String),
}

impl std::fmt::Display for PackageMetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageMetadataError::Read(path, error) => {
                write!(
                    f,
                    "failed to read package metadata {}: {error}",
                    path.display()
                )
            }
            PackageMetadataError::Parse(path, error) => {
                write!(
                    f,
                    "failed to parse package metadata {}: {error}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for PackageMetadataError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Package,
    Dependencies,
    Other,
}

impl PackageMetadata {
    pub fn from_toml_str(text: &str) -> Result<Self, ParseError> {
        let mut section = Section::None;
        let mut saw_package = false;

        let mut name: Option<String> = None;
        let mut version: Option<String> = None;
        let mut edition: Option<String> = None;
        let mut root: Option<String> = None;
        let mut dependencies = BTreeMap::new();

        for (idx, raw_line) in text.lines().enumerate() {
            let lineno = idx + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            if let Some(header) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                section = match header.trim() {
                    "package" => {
                        saw_package = true;
                        Section::Package
                    }
                    "dependencies" => Section::Dependencies,
                    _ => Section::Other,
                };
                continue;
            }

            let (key, value) = split_key_value(line, lineno)?;

            match section {
                Section::Package => match key {
                    "name" => set_once(
                        &mut name,
                        parse_string(value, lineno)?,
                        "package.name",
                        lineno,
                    )?,
                    "version" => set_once(
                        &mut version,
                        parse_string(value, lineno)?,
                        "package.version",
                        lineno,
                    )?,
                    "edition" => set_once(
                        &mut edition,
                        parse_string(value, lineno)?,
                        "package.edition",
                        lineno,
                    )?,
                    "root" => set_once(
                        &mut root,
                        parse_string(value, lineno)?,
                        "package.root",
                        lineno,
                    )?,
                    _ => {
                        return Err(ParseError::new(
                            lineno,
                            format!("unknown [package] key `{key}`"),
                        ))
                    }
                },
                Section::Dependencies => {
                    validate_dependency_name(key, lineno)?;
                    let dep = PackageDependency::Version(parse_string(value, lineno)?);
                    if dependencies.insert(key.to_string(), dep).is_some() {
                        return Err(ParseError::new(
                            lineno,
                            format!("duplicate dependency `{key}`"),
                        ));
                    }
                }
                Section::Other => {
                    // Ignore lod/future tables such as [ffi] and [dependencies.c.libm].
                }
                Section::None => {
                    return Err(ParseError::new(
                        lineno,
                        "key/value pair appears before a table header",
                    ));
                }
            }
        }

        if !saw_package {
            return Err(ParseError::new(1, "missing [package] table"));
        }

        Ok(Self {
            name: required_non_empty(name, "package.name")?,
            version: required_non_empty(version, "package.version")?,
            edition: optional_non_empty(edition, "package.edition")?,
            root: optional_non_empty(root, "package.root")?,
            dependencies,
        })
    }

    pub fn discover_for_source(
        ldx_source: &Path,
    ) -> Result<Option<(Self, PathBuf)>, PackageMetadataError> {
        let dir = ldx_source.parent().unwrap_or_else(|| Path::new("."));
        let path = dir.join(PACKAGE_MANIFEST_FILENAME);

        if !path.exists() {
            return Ok(None);
        }

        let text = std::fs::read_to_string(&path)
            .map_err(|e| PackageMetadataError::Read(path.clone(), e.to_string()))?;

        if !has_package_section(&text) {
            return Ok(None);
        }

        let metadata = Self::from_toml_str(&text)
            .map_err(|e| PackageMetadataError::Parse(path.clone(), e.to_string()))?;

        Ok(Some((metadata, path)))
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_quote = false;
    for (idx, ch) in line.char_indices() {
        if ch == '"' {
            in_quote = !in_quote;
        } else if ch == '#' && !in_quote {
            return &line[..idx];
        }
    }
    line
}

fn has_package_section(text: &str) -> bool {
    text.lines()
        .map(|line| strip_comment(line).trim())
        .any(|line| line == "[package]")
}

fn split_key_value(line: &str, lineno: usize) -> Result<(&str, &str), ParseError> {
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| ParseError::new(lineno, "expected `key = value`"))?;

    let key = key.trim();
    if key.is_empty() {
        return Err(ParseError::new(lineno, "empty key"));
    }

    Ok((key, value.trim()))
}

fn parse_string(value: &str, lineno: usize) -> Result<String, ParseError> {
    let inner = value
        .trim()
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .ok_or_else(|| ParseError::new(lineno, "expected a quoted string"))?;

    if inner.contains('"') {
        return Err(ParseError::new(lineno, "unexpected quote inside string"));
    }

    Ok(inner.to_string())
}

fn set_once(
    slot: &mut Option<String>,
    value: String,
    field: &str,
    lineno: usize,
) -> Result<(), ParseError> {
    if slot.is_some() {
        return Err(ParseError::new(lineno, format!("duplicate {field}")));
    }
    *slot = Some(value);
    Ok(())
}

fn required_non_empty(value: Option<String>, field: &str) -> Result<String, ParseError> {
    let value = value.ok_or_else(|| ParseError::new(1, format!("missing {field}")))?;
    if value.trim().is_empty() {
        return Err(ParseError::new(1, format!("{field} must not be empty")));
    }
    Ok(value)
}

fn optional_non_empty(value: Option<String>, field: &str) -> Result<Option<String>, ParseError> {
    match value {
        Some(value) if value.trim().is_empty() => {
            Err(ParseError::new(1, format!("{field} must not be empty")))
        }
        other => Ok(other),
    }
}

fn validate_dependency_name(name: &str, lineno: usize) -> Result<(), ParseError> {
    if name.is_empty() {
        return Err(ParseError::new(lineno, "empty dependency name"));
    }

    let valid = name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'));

    if !valid {
        return Err(ParseError::new(
            lineno,
            format!("invalid dependency name `{name}`"),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_package() {
        let p = PackageMetadata::from_toml_str("[package]\nname = \"demo\"\nversion = \"0.1.0\"\n")
            .unwrap();
        assert_eq!(p.name, "demo");
        assert_eq!(p.version, "0.1.0");
        assert!(p.dependencies.is_empty());
    }

    #[test]
    fn ignores_lod_tables() {
        let p = PackageMetadata::from_toml_str(
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[ffi]\nallow = [\"labs\"]\n\n[dependencies.c.libm]\nlink = \"m\"\nallow = [\"sqrt\"]\n"
        ).unwrap();
        assert!(p.dependencies.is_empty());
    }
}
