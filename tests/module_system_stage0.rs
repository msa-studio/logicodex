//! Module System Stage 0 -- acceptance criteria as behaviour-level e2e tests.
//!
//! These drive the real `logicodex` binary over multi-file `.ldx` fixtures and
//! assert on exit codes, program output, and diagnostics -- never internal Rust
//! APIs. They are the phase gate for the module system: each test is one of the
//! Stage 0 acceptance criteria from docs/modules/DESIGN.md.
//!
//! Stage 0 scope: function-only modules, `import`, `public`/private, qualified
//! cross-module calls, same-module unqualified calls. NOT in scope (and checked
//! to fail honestly): cross-module struct/enum/type, selective import, aliases.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_logicodex")
}

/// A throwaway project directory under the system temp dir. Multi-file module
/// programs need their files side by side (the loader resolves imports relative
/// to the root file), so each test gets its own directory.
struct Project {
    dir: PathBuf,
}

impl Project {
    fn new(name: &str) -> Self {
        let mut dir = std::env::temp_dir();
        dir.push(format!("ldx_mod_accept_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create project dir");
        Project { dir }
    }

    /// Write a file (e.g. "math.ldx") into the project and return its path.
    fn file(&self, rel: &str, src: &str) -> PathBuf {
        let path = self.dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(&path, src).expect("write file");
        path
    }

    /// Run `logicodex check <root>`; return (exit_code, stdout+stderr).
    /// The dictionary default is `dict/core_map.json`, resolved from the cargo
    /// working directory (the crate root), exactly like the other e2e tests.
    fn check(&self, root: &PathBuf) -> (i32, String) {
        let out = Command::new(bin())
            .arg("check")
            .arg(root)
            .output()
            .expect("spawn logicodex check");
        let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
        combined.push_str(&String::from_utf8_lossy(&out.stderr));
        (out.status.code().unwrap_or(-1), combined)
    }

    /// Compile the root file to a native exe, run it, return (exit_code, stdout).
    fn compile_and_run(&self, root: &PathBuf) -> (i32, String) {
        let compile = Command::new(bin())
            .arg("compile")
            .arg("--emit-ir")
            .arg(root)
            .output()
            .expect("spawn logicodex compile");
        assert!(
            compile.status.success(),
            "compile failed:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let exe = root.with_extension("");
        let run = Command::new(&exe).output().expect("run compiled exe");
        (
            run.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&run.stdout).into_owned(),
        )
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// ---- Criterion 1: single-file programs still compile and run unchanged ------
#[test]
fn c1_single_file_still_works() {
    let p = Project::new("c1");
    let root = p.file(
        "main.ldx",
        "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\nPAPAR add(2, 3);\n",
    );
    let (_code, out) = p.compile_and_run(&root);
    assert_eq!(out.trim(), "5", "single-file add(2,3) prints 5");
}

// ---- Criterion 2: import math; math.add(2,3) works --------------------------
#[test]
fn c2_cross_module_qualified_call_works() {
    let p = Project::new("c2");
    p.file(
        "math.ldx",
        "public function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\n",
    );
    let root = p.file("main.ldx", "import math;\nPAPAR math.add(2, 3);\n");
    let (_code, out) = p.compile_and_run(&root);
    assert_eq!(out.trim(), "5", "math.add(2,3) across files prints 5");
}

// ---- Criterion 3: a non-public function cannot be called from outside -------
#[test]
fn c3_private_function_denied() {
    let p = Project::new("c3");
    // `add` is NOT public -> calling math.add from main must be denied.
    p.file(
        "math.ldx",
        "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\n",
    );
    let root = p.file("main.ldx", "import math;\nPAPAR math.add(2, 3);\n");
    let (code, out) = p.check(&root);
    assert_ne!(code, 0, "private call must fail");
    assert!(
        out.contains("is private") || out.contains("persendirian"),
        "diagnostic names privacy; got:\n{out}"
    );
}

// ---- Criterion 4: a missing module reports the exact path searched ----------
#[test]
fn c4_missing_module_reports_path() {
    let p = Project::new("c4");
    let root = p.file("main.ldx", "import ghost;\nPAPAR ghost.foo(1);\n");
    let (code, out) = p.check(&root);
    assert_ne!(code, 0, "missing module must fail");
    assert!(
        out.contains("ghost.ldx") && (out.contains("not found") || out.contains("tidak")),
        "diagnostic names the searched path; got:\n{out}"
    );
}

// ---- Criterion 5: cyclic imports fail clearly (no hang) ---------------------
#[test]
fn c5_import_cycle_detected() {
    let p = Project::new("c5");
    p.file("a.ldx", "import b;\nPAPAR 1;\n");
    p.file("b.ldx", "import a;\nPAPAR 2;\n");
    let root = p.dir.join("a.ldx");
    let (code, out) = p.check(&root);
    assert_ne!(code, 0, "cycle must fail");
    assert!(
        out.contains("cycle") || out.contains("kitar"),
        "diagnostic names the cycle; got:\n{out}"
    );
}

// ---- Criterion 6: same-named functions in different modules don't collide ---
#[test]
fn c6_same_name_across_modules_no_collision() {
    let p = Project::new("c6");
    p.file(
        "math.ldx",
        "public function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\n",
    );
    p.file(
        "stats.ldx",
        "public function add(a: I64, b: I64) -> I64 begin\n    return a + b + 100;\nend\n",
    );
    // math.add(2,3)=5, stats.add(2,3)=105; print the second so collision would
    // surface as a wrong value (5 instead of 105) rather than a link error.
    let root = p.file(
        "main.ldx",
        "import math;\nimport stats;\nPAPAR stats.add(2, 3);\n",
    );
    let (_code, out) = p.compile_and_run(&root);
    assert_eq!(
        out.trim(),
        "105",
        "stats.add resolves independently of math.add (mangling prevents collision)"
    );
}

// ---- Criterion (option A): public function may call a private same-module one
#[test]
fn ca_public_calls_private_same_module() {
    let p = Project::new("ca");
    p.file(
        "util.ldx",
        "function helper(n: I64) -> I64 begin\n    return n + 100;\nend\n\
         public function calc(x: I64) -> I64 begin\n    return helper(x);\nend\n",
    );
    let root = p.file("main.ldx", "import util;\nPAPAR util.calc(5);\n");
    let (_code, out) = p.compile_and_run(&root);
    assert_eq!(
        out.trim(),
        "105",
        "public calc() calls private same-module helper(): 5 -> 105"
    );
}

// ---- Stage 0 scope honesty: a struct in a module is rejected, not silent ----
#[test]
fn scope_struct_in_module_rejected() {
    let p = Project::new("struct");
    // Non-public struct: parser accepts the decl, lowering rejects it with the
    // Stage-0 function-only diagnostic (the public-struct case is rejected even
    // earlier, at the parser).
    p.file(
        "shapes.ldx",
        "struct Point begin x: I64; end\nfunction helper() -> I64 begin\n    return 1;\nend\n",
    );
    let root = p.file("main.ldx", "import shapes;\nPAPAR shapes.helper();\n");
    let (code, out) = p.check(&root);
    assert_ne!(code, 0, "struct in a module must fail in Stage 0");
    assert!(
        out.contains("Stage 0") || out.contains("not supported") || out.contains("tidak disokong"),
        "diagnostic names the Stage 0 limitation; got:\n{out}"
    );
}

// ---- Reserved namespace: user cannot define __ldx_* symbols -----------------
#[test]
fn reserved_namespace_user_symbol_denied() {
    let p = Project::new("reserved");
    let root = p.file(
        "main.ldx",
        "function __ldx_evil() -> I64 begin\n    return 1;\nend\nPAPAR 1;\n",
    );
    let (code, out) = p.check(&root);
    assert_ne!(code, 0, "reserved-prefix user symbol must fail");
    assert!(
        out.contains("reserved") || out.contains("terpelihara"),
        "diagnostic names the reserved prefix; got:\n{out}"
    );
}
