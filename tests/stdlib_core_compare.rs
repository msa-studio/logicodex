//! core.compare Stage 0 acceptance tests.
//!
//! Uses the REAL shipped `lib/core/compare.ldx` by pointing LOGICODEX_STD at the
//! repo's `lib/` dir. Behaviour-level e2e: compile a main.ldx that imports
//! core.compare, run it, assert the printed output.

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
            "ldx_corecompare_{name}_{}_{}",
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
fn core_compare_compare_i64() {
    let out = run_main(
        "import core.compare;\n\
         PAPAR core.compare.compare_i64(3, 7);\n\
         PAPAR core.compare.compare_i64(5, 5);\n\
         PAPAR core.compare.compare_i64(9, 2);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["-1", "0", "1"], "compare_i64 outputs");
}

#[test]
fn core_compare_order_predicates() {
    let out = run_main(
        "import core.compare;\n\
         PAPAR core.compare.lt_i64(3, 7);\n\
         PAPAR core.compare.lt_i64(5, 5);\n\
         PAPAR core.compare.lte_i64(5, 5);\n\
         PAPAR core.compare.lte_i64(8, 5);\n\
         PAPAR core.compare.gt_i64(9, 2);\n\
         PAPAR core.compare.gt_i64(5, 5);\n\
         PAPAR core.compare.gte_i64(5, 5);\n\
         PAPAR core.compare.gte_i64(2, 9);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(
        got,
        vec!["1", "0", "1", "0", "1", "0", "1", "0"],
        "core.compare ordering predicate outputs"
    );
}

// Cross-module dogfood: core.compare + core.assert together.
#[test]
fn core_compare_with_core_assert() {
    let out = run_main(
        "import core.compare;\n\
         import core.assert;\n\
         PAPAR core.assert.eq_i64(core.compare.compare_i64(3, 7), -1);\n\
         PAPAR core.assert.eq_i64(core.compare.gte_i64(5, 5), 1);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "1"]);
}
