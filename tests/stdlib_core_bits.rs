//! core.bits Stage 0 acceptance tests.
//!
//! Uses the REAL shipped `lib/core/bits.ldx` by pointing LOGICODEX_STD at the
//! repo's `lib/` dir. Behaviour-level e2e: compile a main.ldx that imports
//! core.bits, run it, assert the printed output.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_logicodex")
}

// The real stdlib root shipped in this repo: <repo>/lib.
fn std_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib")
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
            "ldx_corebits_{name}_{}_{}",
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

fn run_main(src: &str) -> String {
    let proj = Tmp::new("proj");
    let root = proj.file("main.ldx", src);
    let compile = Command::new(bin())
        .env("LOGICODEX_STD", std_root())
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
fn core_bits_binary_ops() {
    let out = run_main(
        "import core.bits;\n\
         PAPAR core.bits.bit_and_i64(6, 3);\n\
         PAPAR core.bits.bit_or_i64(4, 1);\n\
         PAPAR core.bits.bit_xor_i64(7, 3);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["2", "5", "4"], "core.bits binary op outputs");
}

#[test]
fn core_bits_flag_helpers() {
    let out = run_main(
        "import core.bits;\n\
         PAPAR core.bits.has_flag_i64(6, 2);\n\
         PAPAR core.bits.has_flag_i64(4, 2);\n\
         PAPAR core.bits.set_flag_i64(4, 2);\n\
         PAPAR core.bits.toggle_flag_i64(6, 2);\n\
         PAPAR core.bits.toggle_flag_i64(4, 2);\n\
         PAPAR core.bits.clear_flag_i64(7, 2);\n\
         PAPAR core.bits.clear_flag_i64(4, 2);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(
        got,
        vec!["1", "0", "6", "4", "6", "5", "4"],
        "core.bits flag helper outputs"
    );
}

// Cross-module dogfood: core.bits + core.assert together.
#[test]
fn core_bits_with_core_assert() {
    let out = run_main(
        "import core.bits;\n\
         import core.assert;\n\
         PAPAR core.assert.eq_i64(core.bits.set_flag_i64(4, 2), 6);\n\
         PAPAR core.assert.eq_i64(core.bits.clear_flag_i64(7, 2), 5);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "1"]);
}
