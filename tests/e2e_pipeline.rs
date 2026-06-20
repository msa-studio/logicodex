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

// ----- runtime ABI builtins (Phase D: sleep + yield) -------------------------
#[test]
fn runtime_sleep_and_yield() {
    // YIELD() -> sched_yield, SLEEP(ms) -> nanosleep. Both are real syscalls
    // backed by os::runtime_assembly(); they must not break the program. We
    // assert on stdout (the two PAPARs straddling the calls), not timing.
    let src = "PAPAR 1;\nYIELD();\nSLEEP(50);\nPAPAR 2;\n";
    let (_code, out) = compile_and_run("sleep_yield", src);
    let lines: Vec<&str> = out.trim().lines().map(|l| l.trim()).collect();
    assert_eq!(lines, vec!["1", "2"], "PAPARs around YIELD/SLEEP both run");
}

// ----- actor runtime is reserved (Phase B): check passes, compile bails ------
#[test]
fn actor_spawn_check_passes_but_compile_is_pending() {
    let src = "ACTOR pekerja MULA\n    PAPAR 1;\nTAMAT\nSPAWN pekerja();\n";
    // check: the program is syntactically + semantically valid.
    let (code, _out) = check("actor_pending", src);
    assert_eq!(code, 0, "actor program type-checks (check passes)");
    // compile: must fail with an honest runtime-pending message, not a raw
    // linker "undefined reference".
    use std::process::Command;
    let path = fixture("actor_pending_compile", src);
    let out = Command::new(bin())
        .arg("compile")
        .arg(&path)
        .output()
        .expect("spawn compile");
    assert!(
        !out.status.success(),
        "compile must fail (no actor runtime yet)"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("runtime not available") && stderr.contains("spawn"),
        "expected honest actor-pending message, got:\n{stderr}"
    );
}

// ----- actor runtime is real under --profile actor (Phase B): spawn + join ---
#[test]
fn actor_spawn_join_runs_in_a_real_thread() {
    // With --profile actor the compiler links the audited pthread runtime
    // (runtime_actor.c). SPAWN runs the actor body in a real OS thread; JOIN
    // (by handle, ABI-1) makes main wait for it, so output is deterministic:
    // the actor prints 99, then main prints 1.
    use std::process::Command;
    let src =
        "ACTOR pekerja MULA\n    PAPAR 99;\nTAMAT\nSPAWN pekerja();\nJOIN pekerja;\nPAPAR 1;\n";
    let path = fixture("actor_spawn_join", src);
    let compile = Command::new(bin())
        .arg("compile")
        .arg("--profile")
        .arg("actor")
        .arg(&path)
        .output()
        .expect("spawn compile --profile actor");
    assert!(
        compile.status.success(),
        "compile --profile actor failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let exe = path.with_extension("");
    let run = Command::new(&exe).output().expect("run actor exe");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        "99\n1",
        "actor spawn+join is deterministic: actor 99 then main 1"
    );
}
// ----- channel B.1: same-scope SPSC works; cross-actor is a clear error ------
#[test]
fn channel_same_scope_send_recv_works() {
    // Channel::baru(N) creates a handle; ch.send/ch.recv are by-handle. Within
    // a single scope (here: main creates, sends, and receives) the SPSC channel
    // round-trips a value. Requires --profile actor (links the pthread runtime).
    use std::process::Command;
    let src = "BINA ch: Channel<Penghantar, Penerima, I64> = Channel::baru(4);\nch.send(123);\nBINA x: I64 = ch.recv();\nPAPAR x;\n";
    let path = fixture("channel_same_scope", src);
    let compile = Command::new(bin())
        .arg("compile")
        .arg("--profile")
        .arg("actor")
        .arg(&path)
        .output()
        .expect("spawn compile --profile actor");
    assert!(
        compile.status.success(),
        "compile --profile actor failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let exe = path.with_extension("");
    let run = Command::new(&exe).output().expect("run channel exe");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        "123",
        "same-scope channel round-trips the value"
    );
}
#[test]
fn channel_cross_actor_fails_with_clear_diagnostic() {
    // A channel declared in main but used inside an actor body needs capture
    // (Channel B.1b), which is not implemented. The compiler must reject this
    // at check time with a clear message — never silently lower to handle 0
    // and deadlock at runtime.
    let src = "BINA ch: Channel<Penghantar, Penerima, I64> = Channel::baru(2);\nACTOR pengeluar MULA\n    ch.send(10);\nTAMAT\nSPAWN pengeluar();\n";
    let (code, out) = check("channel_cross_actor", src);
    assert_ne!(code, 0, "cross-actor channel use must fail check");
    assert!(
        out.contains("Cross-actor channel capture is not implemented"),
        "expected a clear cross-actor diagnostic, got:\n{out}"
    );
}
#[test]
fn channel_actor_capture_cross_thread_works() {
    // B.1b: an actor declares an explicit channel parameter and SPAWN passes the
    // channel handle. The actor sends through the captured handle; main receives
    // across the thread boundary. Capacity 2 < 3 sends also exercises blocking.
    use std::process::Command;
    let src = "ACTOR pengeluar(ch: Channel<Penghantar, Penerima, I64>) MULA\n    ch.send(10);\n    ch.send(20);\n    ch.send(30);\nTAMAT\nBINA ch: Channel<Penghantar, Penerima, I64> = Channel::baru(2);\nSPAWN pengeluar(ch);\nBINA satu: I64 = ch.recv();\nPAPAR satu;\nBINA dua: I64 = ch.recv();\nPAPAR dua;\nBINA tiga: I64 = ch.recv();\nPAPAR tiga;\nJOIN pengeluar;\n";
    let path = fixture("channel_actor_capture", src);
    let compile = Command::new(bin())
        .arg("compile")
        .arg("--profile")
        .arg("actor")
        .arg(&path)
        .output()
        .expect("spawn compile --profile actor");
    assert!(
        compile.status.success(),
        "capture compile --profile actor failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let exe = path.with_extension("");
    let run = Command::new(&exe).output().expect("run capture exe");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        "10\n20\n30",
        "actor sends via captured channel handle; main receives across threads"
    );
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
