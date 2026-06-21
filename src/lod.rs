//! lod Stage 0: Manifest-driven C ABI linking.
//!
//! This is the smallest meaningful slice of the future Logicodex dependency
//! manager. It does NOT yet provide a `lod` CLI, a lockfile, a registry, version
//! resolution, package download, or C header bindgen. It does exactly one thing:
//! read an optional `logicodex.toml` next to the program being compiled and turn
//! it into two compiler inputs --
//!
//!   1. the set of FFI symbols the program is allowed to call
//!      (-> `ffi::CapabilityPolicy::allow_symbol`), and
//!   2. the external C libraries to link
//!      (-> `LinkSpec.user_libs`, emitted as `-l<lib>`).
//!
//! It proves the full chain end to end: declare an `extern "C"` function, allow
//! it in the manifest, the FfiGatekeeper lets the call through, the `-l` flag is
//! injected, the native binary links, and the program calls the real C function.
//!
//! Manifest format (future-proof; grows to sqlite/raylib without migration):
//!
//! ```toml
//! # Top-level direct allow (symbols with no specific library, or libc).
//! [ffi]
//! allow = ["labs"]
//!
//! # Per-C-library: `link` is the -l name, `allow` is the symbols it provides.
//! [dependencies.c.libm]
//! link = "m"
//! allow = ["sqrt", "pow"]
//! ```
//!
//! Merge rule applied by the compiler:
//!   allowed_symbols += [ffi].allow + each [dependencies.c.*].allow
//!   user_libs       += each [dependencies.c.*].link
//!
//! NOTE ON PARSING: this module hand-parses the tiny, fixed subset of TOML the
//! manifest uses (a handful of known tables, string values, and string arrays)
//! rather than pulling in a full TOML crate. The grammar accepted is deliberately
//! small and explicit; anything outside it is a clear parse error. This keeps lod
//! dependency-free, consistent with Logicodex's preference for owning its core.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The parsed `logicodex.toml` manifest. Every field is optional so a minimal or
/// empty manifest is valid; absence simply contributes nothing.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Manifest {
    pub ffi: FfiSection,
    pub dependencies: Dependencies,
}

/// `[ffi]` -- top-level direct symbol allow-list (libc symbols, or symbols not
/// tied to a specific declared C library).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FfiSection {
    pub allow: Vec<String>,
}

/// `[dependencies]` -- currently only the C sub-table is understood.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Dependencies {
    /// `[dependencies.c.<name>]` keyed by the user's chosen library name.
    pub c: BTreeMap<String, CDependency>,
}

/// A single external C library dependency.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CDependency {
    /// The `-l` link name (e.g. "m" for libm, "sqlite3" for libsqlite3).
    /// Optional: a dependency may declare symbols it allows without (yet)
    /// linking, which is a useful honest intermediate state.
    pub link: Option<String>,
    /// The FFI symbols this library is allowed to provide.
    pub allow: Vec<String>,
}

/// The canonical manifest filename, looked up next to the `.ldx` source.
pub const MANIFEST_FILENAME: &str = "logicodex.toml";

/// Which table the parser is currently inside.
enum Section {
    None,
    Ffi,
    /// Inside `[dependencies.c.<name>]`; carries the library name.
    CDep(String),
    /// A recognised-but-unhandled table (e.g. `[dependencies]` itself, or
    /// `[package]`): keys inside are ignored, not an error.
    OtherIgnored,
}

impl Manifest {
    /// Parse a manifest from TOML text using the small hand-written grammar.
    pub fn from_toml_str(text: &str) -> Result<Self, ParseError> {
        let mut manifest = Manifest::default();
        let mut section = Section::None;

        for (lineno, raw) in text.lines().enumerate() {
            let line = strip_comment(raw).trim();
            if line.is_empty() {
                continue;
            }
            if let Some(header) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                let header = header.trim();
                section = classify_header(header, &mut manifest);
                continue;
            }
            // key = value
            let (key, value) = split_key_value(line, lineno + 1)?;
            match &section {
                Section::Ffi => {
                    if key == "allow" {
                        manifest.ffi.allow = parse_string_array(value, lineno + 1)?;
                    }
                }
                Section::CDep(name) => {
                    let dep = manifest.dependencies.c.entry(name.clone()).or_default();
                    match key {
                        "link" => dep.link = Some(parse_string(value, lineno + 1)?),
                        "allow" => dep.allow = parse_string_array(value, lineno + 1)?,
                        _ => {} // unknown key inside a C dep: ignored
                    }
                }
                Section::None | Section::OtherIgnored => {
                    // top-level keys (outside any known table) are ignored
                }
            }
        }
        Ok(manifest)
    }

    /// Look for `logicodex.toml` next to the given `.ldx` source file. Returns
    /// `Ok(None)` when no manifest exists (the common case -- most programs need
    /// no external C). Returns `Err` only when a manifest exists but cannot be
    /// read or parsed.
    pub fn discover(ldx_source: &Path) -> Result<Option<(Manifest, PathBuf)>, ManifestError> {
        let dir = ldx_source.parent().unwrap_or_else(|| Path::new("."));
        let manifest_path = dir.join(MANIFEST_FILENAME);
        if !manifest_path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(&manifest_path)
            .map_err(|e| ManifestError::Read(manifest_path.clone(), e.to_string()))?;
        let manifest = Manifest::from_toml_str(&text)
            .map_err(|e| ManifestError::Parse(manifest_path.clone(), e.to_string()))?;
        Ok(Some((manifest, manifest_path)))
    }

    /// All symbols this manifest allows: `[ffi].allow` plus every
    /// `[dependencies.c.*].allow`. These feed `CapabilityPolicy.allow_symbol`.
    pub fn allowed_symbols(&self) -> Vec<String> {
        let mut out = self.ffi.allow.clone();
        for dep in self.dependencies.c.values() {
            out.extend(dep.allow.iter().cloned());
        }
        out
    }

    /// All `-l` library names to link: every `[dependencies.c.*].link` that is
    /// present. These feed `LinkSpec.user_libs`.
    pub fn user_libs(&self) -> Vec<String> {
        self.dependencies
            .c
            .values()
            .filter_map(|dep| dep.link.clone())
            .collect()
    }
}

/// Build a capability policy for compiling `ldx_source`: start from the runtime
/// builtins (default-deny) and add every symbol the manifest (if any) allows.
/// No manifest -> the default-deny policy is returned unchanged. This is the one
/// place both `compile` and `check` go through, so they agree on what is allowed.
pub fn policy_for_source(ldx_source: &Path) -> Result<crate::ffi::CapabilityPolicy, ManifestError> {
    let mut policy = crate::ffi::CapabilityPolicy::with_runtime_builtins();
    if let Some((manifest, _)) = Manifest::discover(ldx_source)? {
        for sym in manifest.allowed_symbols() {
            policy.allow_symbol(sym);
        }
    }
    Ok(policy)
}

/// Classify a `[header]` and (for C deps) make sure the entry exists. Returns
/// the section the parser is now inside.
fn classify_header(header: &str, manifest: &mut Manifest) -> Section {
    if header == "ffi" {
        return Section::Ffi;
    }
    // [dependencies.c.<name>]
    if let Some(rest) = header.strip_prefix("dependencies.c.") {
        let name = rest.trim();
        if !name.is_empty() {
            manifest.dependencies.c.entry(name.to_string()).or_default();
            return Section::CDep(name.to_string());
        }
    }
    Section::OtherIgnored
}

/// Remove an unquoted `#` comment from a line. A `#` inside a quoted string is
/// preserved (the manifest grammar has no such case today, but this keeps the
/// stripper honest).
fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Split `key = value` into trimmed parts.
fn split_key_value(line: &str, lineno: usize) -> Result<(&str, &str), ParseError> {
    let eq = line
        .find('=')
        .ok_or_else(|| ParseError::new(lineno, "expected `key = value`"))?;
    let key = line[..eq].trim();
    let value = line[eq + 1..].trim();
    if key.is_empty() {
        return Err(ParseError::new(lineno, "empty key"));
    }
    Ok((key, value))
}

/// Parse a quoted string value: `"text"`.
fn parse_string(value: &str, lineno: usize) -> Result<String, ParseError> {
    let v = value.trim();
    let inner = v
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .ok_or_else(|| ParseError::new(lineno, "expected a quoted string"))?;
    if inner.contains('"') {
        return Err(ParseError::new(lineno, "unexpected quote inside string"));
    }
    Ok(inner.to_string())
}

/// Parse a string array: `["a", "b", "c"]` (also accepts an empty `[]`).
fn parse_string_array(value: &str, lineno: usize) -> Result<Vec<String>, ParseError> {
    let v = value.trim();
    let inner = v
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| ParseError::new(lineno, "expected a `[...]` array"))?;
    let inner = inner.trim();
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue; // tolerate a trailing comma
        }
        out.push(parse_string(part, lineno)?);
    }
    Ok(out)
}

/// A manifest parse error with a line number for actionable diagnostics.
#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl ParseError {
    fn new(line: usize, message: &str) -> Self {
        ParseError {
            line,
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Errors from discovering/parsing the manifest. A missing manifest is NOT an
/// error (that is `Ok(None)`); these are only for an existing-but-broken one.
#[derive(Debug)]
pub enum ManifestError {
    Read(PathBuf, String),
    Parse(PathBuf, String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Read(p, e) => {
                write!(f, "failed to read manifest {}: {e}", p.display())
            }
            ManifestError::Parse(p, e) => {
                write!(f, "failed to parse manifest {}: {e}", p.display())
            }
        }
    }
}

impl std::error::Error for ManifestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_manifest_allows_nothing() {
        let m = Manifest::from_toml_str("").unwrap();
        assert!(m.allowed_symbols().is_empty());
        assert!(m.user_libs().is_empty());
    }

    #[test]
    fn ffi_allow_collects_top_level_symbols() {
        let m = Manifest::from_toml_str("[ffi]\nallow = [\"labs\"]\n").unwrap();
        assert_eq!(m.allowed_symbols(), vec!["labs".to_string()]);
        assert!(m.user_libs().is_empty()); // no link declared
    }

    #[test]
    fn c_dependency_collects_symbols_and_link() {
        let toml = "[dependencies.c.libm]\nlink = \"m\"\nallow = [\"sqrt\", \"pow\"]\n";
        let m = Manifest::from_toml_str(toml).unwrap();
        let mut syms = m.allowed_symbols();
        syms.sort();
        assert_eq!(syms, vec!["pow".to_string(), "sqrt".to_string()]);
        assert_eq!(m.user_libs(), vec!["m".to_string()]);
    }

    #[test]
    fn allow_without_link_is_honest_intermediate() {
        let toml = "[dependencies.c.libm]\nallow = [\"sqrt\"]\n";
        let m = Manifest::from_toml_str(toml).unwrap();
        assert_eq!(m.allowed_symbols(), vec!["sqrt".to_string()]);
        assert!(m.user_libs().is_empty());
    }

    #[test]
    fn ffi_and_c_deps_merge() {
        let toml = "[ffi]\nallow = [\"labs\"]\n\n[dependencies.c.libm]\nlink = \"m\"\nallow = [\"sqrt\"]\n";
        let m = Manifest::from_toml_str(toml).unwrap();
        let mut syms = m.allowed_symbols();
        syms.sort();
        assert_eq!(syms, vec!["labs".to_string(), "sqrt".to_string()]);
        assert_eq!(m.user_libs(), vec!["m".to_string()]);
    }

    #[test]
    fn comments_and_blank_lines_ignored() {
        let toml = "# a comment\n\n[ffi]\n# inside\nallow = [\"x\"] # trailing\n";
        let m = Manifest::from_toml_str(toml).unwrap();
        assert_eq!(m.allowed_symbols(), vec!["x".to_string()]);
    }

    #[test]
    fn empty_array_is_allowed() {
        let m = Manifest::from_toml_str("[ffi]\nallow = []\n").unwrap();
        assert!(m.allowed_symbols().is_empty());
    }

    #[test]
    fn malformed_line_is_an_error() {
        let err = Manifest::from_toml_str("[ffi]\nallow\n");
        assert!(err.is_err());
    }
}
