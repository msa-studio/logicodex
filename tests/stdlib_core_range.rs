//! core.range Stage 0 acceptance tests.

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
        dir.push(format!(
            "ldx_corerange_{name}_{}_{}",
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
fn core_range_validity_and_span() {
    let out = run_main(
        "import core.range;\n\
         PAPAR core.range.is_valid_i64(2, 5);\n\
         PAPAR core.range.is_valid_i64(5, 2);\n\
         PAPAR core.range.span_i64(3, 7);\n\
         PAPAR core.range.span_i64(7, 3);\n\
         PAPAR core.range.span_i64(5, 5);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "0", "4", "0", "0"]);
}

#[test]
fn core_range_contains_helpers() {
    let out = run_main(
        "import core.range;\n\
         PAPAR core.range.contains_closed_i64(3, 7, 3);\n\
         PAPAR core.range.contains_closed_i64(3, 7, 7);\n\
         PAPAR core.range.contains_closed_i64(3, 7, 8);\n\
         PAPAR core.range.contains_closed_i64(7, 3, 5);\n\
         PAPAR core.range.contains_open_i64(3, 7, 5);\n\
         PAPAR core.range.contains_open_i64(3, 7, 3);\n\
         PAPAR core.range.contains_open_i64(3, 7, 7);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "1", "0", "0", "1", "0", "0"]);
}

#[test]
fn core_range_overlap_and_touch_helpers() {
    let out = run_main(
        "import core.range;\n\
         PAPAR core.range.overlaps_closed_i64(1, 5, 5, 9);\n\
         PAPAR core.range.overlaps_closed_i64(1, 5, 3, 9);\n\
         PAPAR core.range.overlaps_closed_i64(1, 4, 5, 9);\n\
         PAPAR core.range.overlaps_closed_i64(5, 1, 2, 3);\n\
         PAPAR core.range.touches_i64(1, 5, 5, 9);\n\
         PAPAR core.range.touches_i64(5, 9, 1, 5);\n\
         PAPAR core.range.touches_i64(1, 5, 4, 9);\n\
         PAPAR core.range.touches_i64(5, 1, 1, 5);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "1", "0", "0", "1", "1", "0", "0"]);
}

#[test]
fn core_range_with_core_assert() {
    let out = run_main(
        "import core.range;\n\
         import core.assert;\n\
         PAPAR core.assert.eq_i64(core.range.contains_closed_i64(3, 7, 5), 1);\n\
         PAPAR core.assert.eq_i64(core.range.overlaps_closed_i64(1, 4, 5, 9), 0);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "1"]);
}
