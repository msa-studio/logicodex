//! core.assert Stage 0 acceptance tests.
//!
//! Uses the REAL shipped `lib/core/assert.ldx` (and `lib/core/math.ldx` for the
//! cross-module dogfood) by pointing LOGICODEX_STD at the repo's `lib/` dir.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_logicodex")
}

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
        dir.push(format!("ldx_coreassert_{name}_{}_{}", std::process::id(), uniq));
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
fn assert_eq_i64_true_and_false() {
    let out = run_main(
        "import core.assert;\n\
         PAPAR core.assert.eq_i64(5, 5);\n\
         PAPAR core.assert.eq_i64(5, 6);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "0"], "eq_i64 true then false");
}

#[test]
fn assert_is_true_predicate() {
    let out = run_main(
        "import core.assert;\n\
         PAPAR core.assert.is_true(1);\n\
         PAPAR core.assert.is_true(0);\n\
         PAPAR core.assert.is_true(-3);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "0", "1"], "is_true: nonzero->1, zero->0");
}

// Cross-module dogfood: core.assert + core.math together, the DESIGN example.
#[test]
fn assert_eq_of_core_math_abs() {
    let out = run_main(
        "import core.math;\n\
         import core.assert;\n\
         PAPAR core.assert.eq_i64(core.math.abs_i64(-5), 5);\n",
    );
    assert_eq!(out.split_whitespace().next(), Some("1"),
        "eq_i64(abs_i64(-5), 5) is true");
}
