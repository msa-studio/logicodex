//! Module loader -- Stage 0 of the Logicodex module system.
//!
//! Today `compile()` reads one file. Modules mean many files. This unit isolates
//! that change (mirroring how `src/lod.rs` isolates manifest handling): given a
//! root `.ldx` file, it discovers the full set of modules reachable through
//! `import` statements, parses each one, detects import cycles, and produces a
//! deterministic load order.
//!
//! It does NOT lower to HIR, mangle names, enforce visibility, or touch the
//! capability gate -- those are the caller's job (Phase 3/4). This module's sole
//! responsibility is: turn one root path into an ordered, cycle-free list of
//! parsed modules, or a clear error.
//!
//! ## Resolution
//!
//! `import` is resolved against the filesystem, relative to the importing file.
//! A dot is a directory separator:
//!
//! ```text
//! import math;          ->  <dir>/math.ldx
//! import models.user;   ->  <dir>/models/user.ldx
//! ```
//!
//! Resolution is deterministic: no search path, no implicit root. A module that
//! cannot be found is an error naming the exact path that was tried.
//!
//! ## Cycles
//!
//! Import cycles are forbidden in Stage 0. They are detected during the
//! depth-first walk and reported as a clear error (the cycle path), never an
//! infinite loop.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::ast::{Program, Stmt};

/// A single parsed module: its canonical module name (the dotted import path, or
/// `""` for the root), the file it came from, and its parsed AST.
#[derive(Debug, Clone)]
pub struct LoadedModule {
    /// Dotted module name as written in `import` (e.g. "math", "models.user").
    /// The root module has an empty name.
    pub name: String,
    /// The file this module was loaded from.
    pub path: PathBuf,
    /// The parsed program.
    pub program: Program,
}

/// The result of loading: every reachable module in a valid topological order,
/// dependencies before dependents, with the root last.
#[derive(Debug)]
pub struct ModuleGraph {
    /// Modules in dependency order (imported modules precede importers).
    pub modules: Vec<LoadedModule>,
}

/// Errors from loading the module graph.
#[derive(Debug)]
pub enum LoadError {
    /// A source file could not be read. Carries the path tried and the reason.
    Read(PathBuf, String),
    /// A source file failed to lex or parse. Carries the path and the message.
    Parse(PathBuf, String),
    /// An `import` named a module whose file does not exist. Carries the dotted
    /// module name and the exact filesystem path that was searched.
    ModuleNotFound { module: String, searched: PathBuf },
    /// An import cycle was detected. Carries the cycle as a chain of module
    /// names, e.g. ["a", "b", "a"].
    Cycle(Vec<String>),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Read(p, e) => write!(f, "failed to read module {}: {e}", p.display()),
            LoadError::Parse(p, e) => write!(f, "failed to parse module {}: {e}", p.display()),
            LoadError::ModuleNotFound { module, searched } => write!(
                f,
                "module `{module}` not found (searched {})",
                searched.display()
            ),
            LoadError::Cycle(chain) => {
                write!(f, "import cycle detected: {}", chain.join(" -> "))
            }
        }
    }
}

impl std::error::Error for LoadError {}

/// Resolve a dotted module name to a file path, relative to `base_dir`.
/// A dot becomes a directory separator; the final component gets `.ldx`.
///
///   resolve_module_path("/proj", "math")        -> /proj/math.ldx
///   resolve_module_path("/proj", "models.user") -> /proj/models/user.ldx
pub fn resolve_module_path(base_dir: &Path, module: &str) -> PathBuf {
    let mut path = base_dir.to_path_buf();
    let parts: Vec<&str> = module.split('.').collect();
    for (i, part) in parts.iter().enumerate() {
        if i + 1 == parts.len() {
            path.push(format!("{part}.ldx"));
        } else {
            path.push(part);
        }
    }
    path
}

/// The reserved internal prefix for mangled Logicodex module symbols. User
/// source is forbidden from defining a symbol starting with this prefix, which
/// makes a collision between a user name and a mangled name impossible by
/// construction (the namespace is reserved, not merely hoped to be free).
pub const RESERVED_PREFIX: &str = "__ldx_";

/// Mangle a Logicodex function/struct/enum name into its module-qualified
/// internal symbol. The root module (empty name) does not mangle -- its symbols
/// keep their source names so existing single-file programs are unchanged.
///
///   mangle_symbol("", "main")       -> "main"
///   mangle_symbol("math", "add")    -> "__ldx_mod_math__add"
///   mangle_symbol("models.user", "new") -> "__ldx_mod_models_user__new"
///
/// A dot in the module path becomes an underscore so the result is a single
/// valid identifier. This is for Logicodex symbols only: extern "C" symbols are
/// NEVER mangled (they must keep their exact ABI name or linking breaks).
pub fn mangle_symbol(module: &str, name: &str) -> String {
    if module.is_empty() {
        return name.to_string();
    }
    let module_part = module.replace('.', "_");
    format!("{RESERVED_PREFIX}mod_{module_part}__{name}")
}

/// Whether a user-written symbol name is forbidden because it intrudes on the
/// reserved mangling namespace.
pub fn is_reserved_symbol(name: &str) -> bool {
    name.starts_with(RESERVED_PREFIX)
}

/// Extract the dotted module names imported by a program, in source order.
pub fn imports_of(program: &Program) -> Vec<String> {
    program
        .statements
        .iter()
        .filter_map(|stmt| match stmt {
            Stmt::Use { module } => Some(module.clone()),
            _ => None,
        })
        .collect()
}

/// A function that lexes+parses one source string into a `Program`. Injected so
/// the loader stays testable without wiring the whole lexicon/lexer/parser chain
/// into its unit tests. The real compiler passes a closure that runs the v1.30
/// pipeline; tests pass a tiny stub.
pub type ParseFn<'a> = dyn Fn(&str, &Path) -> Result<Program, String> + 'a;

/// Load the full module graph reachable from `root_file`.
///
/// `parse` lexes+parses one file's source into a Program. The loader handles
/// discovery, resolution, cycle detection, and ordering.
pub fn load_graph(root_file: &Path, parse: &ParseFn<'_>) -> Result<ModuleGraph, LoadError> {
    let mut modules: Vec<LoadedModule> = Vec::new();
    // Map dotted module name -> index in `modules` once fully loaded.
    let mut done: BTreeSet<String> = BTreeSet::new();
    // Modules on the current DFS stack, for cycle detection.
    let mut on_stack: BTreeSet<String> = BTreeSet::new();
    // The base directory imports resolve against (the root file's directory).
    let base_dir = root_file
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    // We key the recursion on the dotted module name; the root is "".
    // For a non-root module, its file path is resolve_module_path(base_dir, name).
    fn visit(
        module_name: &str,
        file_path: &Path,
        base_dir: &Path,
        parse: &ParseFn<'_>,
        modules: &mut Vec<LoadedModule>,
        done: &mut BTreeSet<String>,
        on_stack: &mut BTreeSet<String>,
        stack: &mut Vec<String>,
    ) -> Result<(), LoadError> {
        if done.contains(module_name) {
            return Ok(());
        }
        if on_stack.contains(module_name) {
            // Cycle: build the chain from where module_name first appears.
            let mut chain: Vec<String> = stack.clone();
            chain.push(module_name.to_string());
            return Err(LoadError::Cycle(chain));
        }
        on_stack.insert(module_name.to_string());
        stack.push(module_name.to_string());

        let source = std::fs::read_to_string(file_path)
            .map_err(|e| LoadError::Read(file_path.to_path_buf(), e.to_string()))?;
        let program =
            parse(&source, file_path).map_err(|e| LoadError::Parse(file_path.to_path_buf(), e))?;

        // Recurse into imports first, so dependencies are emitted before us.
        for imported in imports_of(&program) {
            let imported_path = resolve_module_path(base_dir, &imported);
            if !imported_path.exists() {
                return Err(LoadError::ModuleNotFound {
                    module: imported,
                    searched: imported_path,
                });
            }
            visit(
                &imported,
                &imported_path,
                base_dir,
                parse,
                modules,
                done,
                on_stack,
                stack,
            )?;
        }

        on_stack.remove(module_name);
        stack.pop();
        done.insert(module_name.to_string());
        modules.push(LoadedModule {
            name: module_name.to_string(),
            path: file_path.to_path_buf(),
            program,
        });
        Ok(())
    }

    let mut stack: Vec<String> = Vec::new();
    visit(
        "",
        root_file,
        &base_dir,
        parse,
        &mut modules,
        &mut done,
        &mut on_stack,
        &mut stack,
    )?;

    Ok(ModuleGraph { modules })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Program, Stmt};
    use std::cell::RefCell;
    use std::collections::HashMap;

    fn use_stmt(module: &str) -> Stmt {
        Stmt::Use {
            module: module.to_string(),
        }
    }

    // A print of integer 0, used as trivial non-import body.
    fn trivial_body() -> Stmt {
        Stmt::Print {
            value: Expr::Integer(0),
        }
    }

    #[test]
    fn resolve_single_component() {
        let p = resolve_module_path(Path::new("/proj"), "math");
        assert_eq!(p, PathBuf::from("/proj/math.ldx"));
    }

    #[test]
    fn resolve_dotted_is_directory() {
        let p = resolve_module_path(Path::new("/proj"), "models.user");
        assert_eq!(p, PathBuf::from("/proj/models/user.ldx"));
    }

    #[test]
    fn root_module_does_not_mangle() {
        assert_eq!(mangle_symbol("", "main"), "main");
    }

    #[test]
    fn named_module_mangles_with_reserved_prefix() {
        assert_eq!(mangle_symbol("math", "add"), "__ldx_mod_math__add");
    }

    #[test]
    fn dotted_module_underscores_the_path() {
        assert_eq!(
            mangle_symbol("models.user", "new"),
            "__ldx_mod_models_user__new"
        );
    }

    #[test]
    fn reserved_prefix_is_detected() {
        assert!(is_reserved_symbol("__ldx_mod_math__add"));
        assert!(is_reserved_symbol("__ldx_anything"));
        assert!(!is_reserved_symbol("add"));
        assert!(!is_reserved_symbol("my_func"));
    }

    /// Build a parse stub backed by an in-memory name->Program map. The stub
    /// looks the file up by its file stem chain so tests need no real files...
    /// but load_graph checks `imported_path.exists()` on the real filesystem, so
    /// the filesystem-touching paths are covered by the integration tests in the
    /// compiler, not here. These unit tests cover resolution + ordering logic on
    /// programs directly via the lower-level helpers.
    #[test]
    fn imports_are_extracted_in_order() {
        let program = Program {
            statements: vec![
                use_stmt("alpha"),
                use_stmt("beta"),
                trivial_body(),
                use_stmt("gamma"),
            ],
        };
        assert_eq!(imports_of(&program), vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn no_imports_yields_empty() {
        let program = Program {
            statements: vec![trivial_body()],
        };
        assert!(imports_of(&program).is_empty());
    }

    // Full load_graph tests use a temp directory so imported_path.exists() holds.
    fn write_temp_tree(files: &[(&str, &str)]) -> (tempdir_shim::TempDir, PathBuf) {
        let dir = tempdir_shim::TempDir::new();
        for (rel, contents) in files {
            let full = dir.path().join(rel);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(&full, contents).unwrap();
        }
        let root = dir.path().join(files[0].0);
        (dir, root)
    }

    // Parse stub: maps each file path to a pre-built Program keyed by filename.
    fn stub_parser(
        table: HashMap<PathBuf, Program>,
    ) -> impl Fn(&str, &Path) -> Result<Program, String> {
        let table = RefCell::new(table);
        move |_src: &str, path: &Path| {
            table
                .borrow()
                .get(path)
                .cloned()
                .ok_or_else(|| format!("no stub program for {}", path.display()))
        }
    }

    #[test]
    fn load_orders_dependencies_before_root() {
        // root imports math; math has no imports.
        let (dir, root) = write_temp_tree(&[("main.ldx", "import math;"), ("math.ldx", "")]);
        let math_path = dir.path().join("math.ldx");
        let mut table = HashMap::new();
        table.insert(
            root.clone(),
            Program {
                statements: vec![use_stmt("math")],
            },
        );
        table.insert(
            math_path.clone(),
            Program {
                statements: vec![trivial_body()],
            },
        );
        let parse = stub_parser(table);
        let graph = load_graph(&root, &parse).expect("load ok");
        let names: Vec<&str> = graph.modules.iter().map(|m| m.name.as_str()).collect();
        // math (dependency) before "" (root).
        assert_eq!(names, vec!["math", ""]);
    }

    #[test]
    fn missing_module_reports_searched_path() {
        let (dir, root) = write_temp_tree(&[("main.ldx", "import ghost;")]);
        let mut table = HashMap::new();
        table.insert(
            root.clone(),
            Program {
                statements: vec![use_stmt("ghost")],
            },
        );
        let parse = stub_parser(table);
        let err = load_graph(&root, &parse).unwrap_err();
        match err {
            LoadError::ModuleNotFound { module, searched } => {
                assert_eq!(module, "ghost");
                assert_eq!(searched, dir.path().join("ghost.ldx"));
            }
            other => panic!("expected ModuleNotFound, got {other:?}"),
        }
    }

    #[test]
    fn cycle_is_detected() {
        // a imports b, b imports a.
        let (dir, root) = write_temp_tree(&[("a.ldx", "import b;"), ("b.ldx", "import a;")]);
        let a_path = dir.path().join("a.ldx");
        let b_path = dir.path().join("b.ldx");
        let mut table = HashMap::new();
        // root is a.ldx here (name "").
        table.insert(
            root.clone(),
            Program {
                statements: vec![use_stmt("b")],
            },
        );
        table.insert(
            a_path,
            Program {
                statements: vec![use_stmt("b")],
            },
        );
        table.insert(
            b_path,
            Program {
                statements: vec![use_stmt("a")],
            },
        );
        let parse = stub_parser(table);
        let err = load_graph(&root, &parse).unwrap_err();
        assert!(
            matches!(err, LoadError::Cycle(_)),
            "expected cycle, got {err:?}"
        );
    }

    // Minimal temp-dir shim so the loader's unit tests do not depend on an
    // external `tempfile` crate (offline, prefer-owning-core). Creates a unique
    // directory under the system temp dir and removes it on drop.
    mod tempdir_shim {
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicU64, Ordering};

        static COUNTER: AtomicU64 = AtomicU64::new(0);

        pub struct TempDir {
            path: PathBuf,
        }

        impl TempDir {
            pub fn new() -> Self {
                let n = COUNTER.fetch_add(1, Ordering::Relaxed);
                let pid = std::process::id();
                let mut path = std::env::temp_dir();
                path.push(format!("ldx_modload_test_{pid}_{n}"));
                std::fs::create_dir_all(&path).unwrap();
                TempDir { path }
            }
            pub fn path(&self) -> &Path {
                &self.path
            }
        }

        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.path);
            }
        }
    }
}
