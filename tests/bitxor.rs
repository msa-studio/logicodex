//! BitXor `^` operator: full lex -> parse -> codegen -> run proof.
//!
//! Acceptance: 5^3=6, 8^1=9, 7^7=0; precedence & > ^ > | (C-like); composes
//! with shifts (mini xorshift). Integer operator only.

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
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let uniq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut dir = std::env::temp_dir();
        dir.push(format!("ldx_bitxor_{}_{}", std::process::id(), uniq));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        Tmp { dir }
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn run_main(src: &str) -> String {
    let proj = Tmp::new();
    let root = proj.dir.join("main.ldx");
    std::fs::write(&root, src).expect("write main.ldx");
    let compile = Command::new(bin())
        .arg("compile")
        .arg("--emit-ir")
        .arg(&root)
        .output()
        .expect("spawn compile");
    assert!(
        compile.status.success(),
        "compile failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let exe = root.with_extension("");
    let run = Command::new(&exe).output().expect("run compiled exe");
    String::from_utf8_lossy(&run.stdout).into_owned()
}

#[test]
fn bitxor_basic_values() {
    let out = run_main("PAPAR 5 ^ 3;\nPAPAR 8 ^ 1;\nPAPAR 7 ^ 7;\n");
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["6", "9", "0"], "5^3=6, 8^1=9, 7^7=0");
}

// `^` binds tighter than `|`: 1 | 2 ^ 3 == 1 | (2 ^ 3) == 1. Left-assoc same
// level would give (1|2)^3 == 0, so 1 proves the precedence.
#[test]
fn bitxor_tighter_than_bitor() {
    let out = run_main("PAPAR 1 | 2 ^ 3;\n");
    assert_eq!(out.split_whitespace().next(), Some("1"), "^ tighter than |");
}

// `&` binds tighter than `^`: 6 ^ 2 & 3 == 6 ^ (2 & 3) == 4. Left-assoc same
// level would give (6^2)&3 == 0, so 4 proves the precedence.
#[test]
fn bitand_tighter_than_bitxor() {
    let out = run_main("PAPAR 6 ^ 2 & 3;\n");
    assert_eq!(out.split_whitespace().next(), Some("4"), "& tighter than ^");
}

// Mini xorshift proves `^` composes with shifts. Shift binds tighter than `^`,
// so the parens are redundant but kept for clarity.
#[test]
fn bitxor_xorshift_composes_with_shifts() {
    let out = run_main(
        "let x: I64 = 1;\n\
         x = x ^ (x << 13);\n\
         x = x ^ (x >> 7);\n\
         x = x ^ (x << 17);\n\
         PAPAR x;\n",
    );
    assert_eq!(
        out.split_whitespace().next(),
        Some("1082269761"),
        "xorshift(1) final state"
    );
}
