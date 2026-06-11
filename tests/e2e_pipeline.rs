//! End-to-end phase-gate tests.
//!
//! These tests drive the *real* `logicodex` binary over `.ldx` fixtures and
//! assert on exit codes and program output. They intentionally touch ONLY the
//! stable CLI surface (`compile` / `check`) and observable program behaviour —
//! never internal Rust APIs (AST shapes, lowering functions, registry types).
//!
//! Rationale: the legacy unit/integration tests drifted because they were
//! coupled to internal types that evolve every refactor. Behaviour-level e2e
//! tests are drift-resistant: they only break when the *language's observable
//! behaviour* changes, which is exactly what a phase gate should guard.
//!
//! This file is the phase-gate baseline. Every future change must keep it green.

use std::path::PathBuf;
use std::process::Command;

/// Path to the compiled binary (cargo sets this for integration tests).
fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_logicodex")
}

/// Write a fixture to a unique temp file and return its path.
fn fixture(name: &str, src: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("ldx_e2e_{name}.ldx"));
    std::fs::write(&p, src).expect("write fixture");
    p
}

/// Run `logicodex check <fixture>`; return (exit_code, combined stdout+stderr).
fn check(name: &str, src: &str) -> (i32, String) {
    let path = fixture(name, src);
    let out = Command::new(bin())
        .arg("check")
        .arg(&path)
        .output()
        .expect("spawn logicodex check");
    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), combined)
}

/// Compile a fixture to a native executable, run it, return (exit_code, stdout).
fn compile_and_run(name: &str, src: &str) -> (i32, String) {
    let path = fixture(name, src);
    let compile = Command::new(bin())
        .arg("compile")
        .arg("--emit-ir")
        .arg(&path)
        .output()
        .expect("spawn logicodex compile");
    assert!(
        compile.status.success(),
        "compile failed for {name}:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let exe = path.with_extension("");
    let run = Command::new(&exe).output().expect("run compiled exe");
    (
        run.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&run.stdout).into_owned(),
    )
}

// ----- core semantics: compile + run -----------------------------------------

#[test]
fn arithmetic_print() {
    // NOTE: a program's exit code currently carries the last binding's value
    // (a codegen quirk), so we assert on the PRINTED stdout, not the exit code.
    let (_code, out) = compile_and_run("arith", "BINA x: I32 = 5; PAPAR x + 37;\n");
    assert_eq!(out.trim(), "42", "5 + 37 printed to stdout");
}

#[test]
fn fixed_width_i8_wraps() {
    // I8 stores wrap to 8 bits: 300 -> 300-256 = 44.
    let (_code, out) = compile_and_run("i8wrap", "BINA a: I8 = 300; PAPAR a;\n");
    assert_eq!(out.trim(), "44", "300 wrapped to I8");
}

// ----- capability vocabulary check: `check` exit semantics -------------------

#[test]
fn capability_malformed_gate_is_error() {
    let (code, out) = check(
        "cap_bad",
        "service Web { port: 8080, requires: Foo, handler: H, policy: Block }\n",
    );
    assert_ne!(code, 0, "malformed gate must fail check");
    assert!(
        out.contains("malformed capability"),
        "expected malformed-capability diagnostic, got:\n{out}"
    );
}

#[test]
fn capability_unknown_gate_is_warning_but_passes() {
    let (code, out) = check(
        "cap_warn",
        "service Api { port: 9090, requires: Net.Admin, handler: H, policy: Block }\n",
    );
    assert_eq!(code, 0, "unknown-but-wellformed gate should still pass");
    assert!(
        out.contains("not in the standard capability vocabulary"),
        "expected vocabulary warning, got:\n{out}"
    );
}

#[test]
fn capability_valid_gate_passes_silently() {
    let (code, out) = check(
        "cap_ok",
        "service Dev { port: 1, requires: Net.Send, handler: H, policy: Block }\n",
    );
    assert_eq!(code, 0, "in-vocabulary gate should pass");
    assert!(
        !out.contains("not in the standard capability vocabulary"),
        "valid gate must not warn, got:\n{out}"
    );
}
