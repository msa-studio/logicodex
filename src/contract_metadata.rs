#![allow(dead_code)]

// =========================================================================
// Contract metadata extension hooks.
//
// This is a lightweight Community Edition foundation for contract-compliant
// static extensions. It declares stable categories and sidecar metadata paths;
// it does not validate governance, audit, trust, or enterprise policy.
// =========================================================================

use std::path::{Path, PathBuf};

/// Stable contract-extension families that the compiler may recognize without
/// hardcoding individual implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionFamily {
    Library,
    Ffi,
    RuntimeProfile,
    Frontend,
    Il,
    Diagnostic,
    Agent,
    AuditMetadata,
}

/// Official Logicodex library layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryLayer {
    Core,
    Std,
    Framework,
}

impl LibraryLayer {
    pub fn from_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            "core" => Some(Self::Core),
            "std" => Some(Self::Std),
            "framework" => Some(Self::Framework),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Std => "std",
            Self::Framework => "framework",
        }
    }
}

/// Minimal engine-visible hint for a contract-compliant static library module.
///
/// This intentionally stores paths and categories only. It does not parse or
/// enforce the sidecar file during normal compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractMetadataHint {
    pub family: ExtensionFamily,
    pub module: String,
    pub layer: LibraryLayer,
    pub source_path: PathBuf,
    pub contract_path: PathBuf,
}

/// Return the official library layer for a dotted module name such as
/// `core.math`, `std.io`, or `framework.http`.
pub fn module_library_layer(module: &str) -> Option<LibraryLayer> {
    let first = module.split('.').next().unwrap_or("");
    LibraryLayer::from_prefix(first)
}

/// Whether the dotted module name belongs to an official library namespace.
pub fn is_official_library_module(module: &str) -> bool {
    module_library_layer(module).is_some()
}

/// Contract sidecar path for a `.ldx` source module.
///
/// Example: `lib/core/math.ldx` -> `lib/core/math.std.toml`.
pub fn library_contract_sidecar_path(source_path: &Path) -> PathBuf {
    let mut path = source_path.to_path_buf();
    path.set_extension("std.toml");
    path
}

/// Build a lightweight contract metadata hint for official library modules.
pub fn library_contract_hint(module: &str, source_path: &Path) -> Option<ContractMetadataHint> {
    let layer = module_library_layer(module)?;
    Some(ContractMetadataHint {
        family: ExtensionFamily::Library,
        module: module.to_string(),
        layer,
        source_path: source_path.to_path_buf(),
        contract_path: library_contract_sidecar_path(source_path),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_official_library_layers() {
        assert_eq!(module_library_layer("core.math"), Some(LibraryLayer::Core));
        assert_eq!(module_library_layer("std.io"), Some(LibraryLayer::Std));
        assert_eq!(
            module_library_layer("framework.http"),
            Some(LibraryLayer::Framework)
        );
        assert_eq!(module_library_layer("app.models"), None);
    }

    #[test]
    fn sidecar_uses_std_toml_extension() {
        let path = library_contract_sidecar_path(Path::new("lib/core/math.ldx"));
        assert_eq!(path, PathBuf::from("lib/core/math.std.toml"));
    }

    #[test]
    fn library_contract_hint_is_path_only() {
        let hint = library_contract_hint("core.math", Path::new("lib/core/math.ldx"))
            .expect("core.math is official");
        assert_eq!(hint.family, ExtensionFamily::Library);
        assert_eq!(hint.layer, LibraryLayer::Core);
        assert_eq!(hint.module, "core.math");
        assert_eq!(hint.contract_path, PathBuf::from("lib/core/math.std.toml"));
    }

    #[test]
    fn non_library_module_has_no_contract_hint() {
        assert!(library_contract_hint("app.models", Path::new("app/models.ldx")).is_none());
    }
}
