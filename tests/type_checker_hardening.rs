//! P0 type checker hardening regression tests.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_logicodex")
}

struct Tmp {
    dir: PathBuf,
}

impl Tmp {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let uniq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "ldx_type_checker_{name}_{}_{}",
            std::process::id(),
            uniq
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        Tmp { dir }
    }

    fn file(&self, rel: &str, src: &str) -> PathBuf {
        let p = self.dir.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("mkdir parent");
        }
        std::fs::write(&p, src).expect("write");
        p
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn check_fail(src: &str) -> String {
    let proj = Tmp::new("fail");
    let root = proj.file("main.ldx", src);

    let out = Command::new(bin())
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
        .arg("check")
        .arg(&root)
        .output()
        .expect("spawn check");

    assert!(
        !out.status.success(),
        "check unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn check_ok(src: &str) {
    let proj = Tmp::new("ok");
    let root = proj.file("main.ldx", src);

    let out = Command::new(bin())
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
        .arg("check")
        .arg(&root)
        .output()
        .expect("spawn check");

    assert!(
        out.status.success(),
        "check failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn local_function_call_argument_count_mismatch_fails() {
    let output = check_fail(
        "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\nPAPAR add(1);\n",
    );

    assert!(
        output.contains("argument count mismatch") || output.contains("bilangan argumen"),
        "expected argument count diagnostic, got:\n{output}"
    );
}

#[test]
fn local_function_call_argument_type_mismatch_fails() {
    let output =
        check_fail("function id(a: I64) -> I64 begin\n    return a;\nend\nPAPAR id(true);\n");

    assert!(
        output.contains("argument 1 type mismatch") || output.contains("jenis argumen"),
        "expected argument type diagnostic, got:\n{output}"
    );
}

#[test]
fn local_function_call_return_type_flows_to_declared_assignment() {
    let output = check_fail(
        "function flag() -> Bool begin\n    return true;\nend\nlet x: I64 = flag();\nPAPAR 1;\n",
    );

    assert!(
        output.contains("Binding type mismatch") || output.contains("Jenis ikatan"),
        "expected declared assignment mismatch using actual call return type, got:\n{output}"
    );
}

#[test]
fn local_function_call_with_matching_args_still_passes() {
    check_ok(
        "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\nPAPAR add(1, 2);\n",
    );
}

#[test]
fn function_return_expression_type_mismatch_fails() {
    let output = check_fail("function bad() -> I64 begin\n    return true;\nend\nPAPAR 1;\n");

    assert!(
        output.contains("Return type mismatch") || output.contains("Jenis pulangan"),
        "expected return type mismatch diagnostic, got:\n{output}"
    );
}

#[test]
fn function_return_expression_with_matching_type_still_passes() {
    check_ok("function flag() -> Bool begin\n    return true;\nend\nPAPAR 1;\n");
}

#[test]
fn assignment_type_mismatch_fails() {
    let output = check_fail("let x: I64 = 1;\nx = true;\nPAPAR 1;\n");

    assert!(
        output.contains("Assignment type mismatch") || output.contains("Jenis tugasan"),
        "expected assignment type mismatch diagnostic, got:\n{output}"
    );
}

#[test]
fn assignment_type_match_still_passes() {
    check_ok("let x: I64 = 1;\nx = 2;\nPAPAR x;\n");
}
