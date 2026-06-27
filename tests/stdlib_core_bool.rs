//! core.bool Stage 0 acceptance tests.
//!
//! Uses the REAL shipped `lib/core/bool.ldx` by pointing LOGICODEX_STD at the
//! repo's `lib/` dir. Behaviour-level e2e: compile a main.ldx that imports
//! core.bool, run it, assert the printed output.

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
        dir.push(format!("ldx_corebool_{name}_{}_{}", std::process::id(), uniq));
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
fn core_bool_truthiness_helpers() {
    let out = run_main(
        "import core.bool;\n\
         PAPAR core.bool.truthy_i64(5);\n\
         PAPAR core.bool.truthy_i64(-3);\n\
         PAPAR core.bool.truthy_i64(0);\n\
         PAPAR core.bool.falsey_i64(0);\n\
         PAPAR core.bool.falsey_i64(9);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "1", "0", "1", "0"]);
}

#[test]
fn core_bool_logic_helpers() {
    let out = run_main(
        "import core.bool;\n\
         PAPAR core.bool.not_i64(0);\n\
         PAPAR core.bool.not_i64(4);\n\
         PAPAR core.bool.and_i64(1, -2);\n\
         PAPAR core.bool.and_i64(0, 7);\n\
         PAPAR core.bool.and_i64(7, 0);\n\
         PAPAR core.bool.or_i64(0, 0);\n\
         PAPAR core.bool.or_i64(8, 0);\n\
         PAPAR core.bool.or_i64(0, -8);\n\
         PAPAR core.bool.xor_i64(0, 0);\n\
         PAPAR core.bool.xor_i64(9, 0);\n\
         PAPAR core.bool.xor_i64(0, -9);\n\
         PAPAR core.bool.xor_i64(2, 3);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(
        got,
        vec!["1", "0", "1", "0", "0", "0", "1", "1", "0", "1", "1", "0"],
        "core.bool logic helper outputs"
    );
}

// Cross-module dogfood: core.bool + core.assert together.
#[test]
fn core_bool_with_core_assert() {
    let out = run_main(
        "import core.bool;\n\
         import core.assert;\n\
         PAPAR core.assert.eq_i64(core.bool.truthy_i64(-3), 1);\n\
         PAPAR core.assert.eq_i64(core.bool.xor_i64(2, 3), 0);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "1"]);
}
