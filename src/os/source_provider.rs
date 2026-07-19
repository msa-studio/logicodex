// =========================================================================
// Source Code Provider
//
// Provides source code (.ldx files) to the compiler.
// Two modes:
//   1. Hosted: read from filesystem (std::fs)
//   2. Freestanding: embedded via include_str! / static bytes
//
// This allows the compiler to work in environments without a filesystem.
// The source is either read from disk (hosted) or embedded at compile time
// (freestanding test programs).
// =========================================================================

/// Source code provider trait.
/// Abstracts over filesystem vs embedded sources.
pub trait SourceProvider {
    /// Error type for source loading failures.
    type Error: core::fmt::Display;

    /// Load source code by name/path.
    /// Returns the full source text.
    fn load(&self, name: &str) -> Result<String, Self::Error>;

    /// Check if a source exists.
    fn exists(&self, name: &str) -> bool;
}

// ─── Hosted: Filesystem Provider ───

#[cfg(not(target_os = "none"))]
pub mod hosted {
    use super::SourceProvider;
    use std::fs;
    use std::path::Path;

    /// Load source from the filesystem.
    /// Standard mode for hosted compilation.
    pub struct FileSystemProvider;

    impl SourceProvider for FileSystemProvider {
        type Error = std::io::Error;

        fn load(&self, path: &str) -> Result<String, Self::Error> {
            fs::read_to_string(path)
        }

        fn exists(&self, path: &str) -> bool {
            Path::new(path).exists()
        }
    }

    /// Convenience: load a single source file.
    pub fn load_source_file(path: &str) -> Result<String, std::io::Error> {
        fs::read_to_string(path)
    }
}

// ─── Freestanding: Embedded Provider ───
// For bare-metal programs, source is embedded at compile time.

/// Provider that sources from embedded static strings.
/// Used when the compiler itself runs freestanding (self-hosted).
pub struct EmbeddedProvider<'a> {
    sources: &'a [(&'a str, &'a str)],
}

impl<'a> EmbeddedProvider<'a> {
    /// Create a new embedded provider.
    ///
    /// # Example
    /// ```
    /// let sources = &[
    ///     ("main.ldx", include_str!("../examples/hello.ldx")),
    /// ];
    /// let provider = EmbeddedProvider::new(sources);
    /// ```
    pub const fn new(sources: &'a [(&'a str, &'a str)]) -> Self {
        EmbeddedProvider { sources }
    }
}

impl<'a> SourceProvider for EmbeddedProvider<'a> {
    type Error = &'static str;

    fn load(&self, name: &str) -> Result<String, Self::Error> {
        for (src_name, src_content) in self.sources {
            if *src_name == name {
                return Ok((*src_content).to_string());
            }
        }
        Err("Source not found in embedded set")
    }

    fn exists(&self, name: &str) -> bool {
        self.sources.iter().any(|(n, _)| *n == name)
    }
}

// ─── Binary (Blob) Provider ───
// For very constrained environments: source is a raw byte slice.

pub struct BinaryProvider<'a> {
    data: &'a [u8],
}

impl<'a> BinaryProvider<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        BinaryProvider { data }
    }
}

impl<'a> SourceProvider for BinaryProvider<'a> {
    type Error = &'static str;

    fn load(&self, _name: &str) -> Result<String, Self::Error> {
        match core::str::from_utf8(self.data) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err("Source data is not valid UTF-8"),
        }
    }

    fn exists(&self, _name: &str) -> bool {
        !self.data.is_empty()
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_provider_finds_source() {
        let sources = &[
            ("hello.ldx", "print \"Hello, World!\""),
            ("math.ldx", "let x = 42"),
        ];
        let provider = EmbeddedProvider::new(sources);

        assert!(provider.exists("hello.ldx"));
        assert!(provider.exists("math.ldx"));
        assert!(!provider.exists("missing.ldx"));

        let hello = provider.load("hello.ldx").unwrap();
        assert_eq!(hello, "print \"Hello, World!\"");
    }

    #[test]
    fn embedded_provider_missing() {
        let sources = &[("a.ldx", "")];
        let provider = EmbeddedProvider::new(sources);

        let result = provider.load("nonexistent.ldx");
        assert!(result.is_err());
    }

    #[test]
    fn binary_provider_from_utf8() {
        let data = b"let x = 42";
        let provider = BinaryProvider::new(data);

        assert!(provider.exists("any"));
        let src = provider.load("any").unwrap();
        assert_eq!(src, "let x = 42");
    }

    #[test]
    fn binary_provider_invalid_utf8() {
        let data = &[0x80, 0x81, 0x82]; // Invalid UTF-8
        let provider = BinaryProvider::new(data);

        let result = provider.load("any");
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(target_os = "none"))]
    fn filesystem_provider_exists_for_tests() {
        use hosted::FileSystemProvider;
        let provider = FileSystemProvider;
        // The test itself exists
        assert!(provider.exists("src/lib.rs"));
    }
}
