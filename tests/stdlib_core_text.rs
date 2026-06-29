//! core.text CPB Phase 1 acceptance tests.
//!
//! Uses the REAL shipped `lib/core/text.ldx` by pointing LOGICODEX_STD at the
//! repo's `lib/` dir. Behaviour-level e2e: compile a main.ldx that imports
//! core.text, run it, assert the printed output.

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
        dir.push(format!("ldx_coretext_{name}_{}_{}", std::process::id(), uniq));
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
fn core_text_empty_helpers() {
    let out = run_main(
        "import core.text;\n\
         PAPAR core.text.is_empty_text_i64(\"\");\n\
         PAPAR core.text.is_empty_text_i64(\"abc\");\n\
         PAPAR core.text.not_empty_text_i64(\"abc\");\n\
         PAPAR core.text.not_empty_text_i64(\"\");\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "0", "1", "0"]);
}

#[test]
fn core_text_with_core_assert() {
    let out = run_main(
        "import core.text;\n\
         import core.assert;\n\
         PAPAR core.assert.eq_i64(core.text.is_empty_text_i64(\"\"), 1);\n\
         PAPAR core.assert.eq_i64(core.text.is_empty_text_i64(\"abc\"), 0);\n\
         PAPAR core.assert.eq_i64(core.text.not_empty_text_i64(\"abc\"), 1);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "1", "1"]);
}

#[test]
fn core_text_typed_binding_and_return_smoke() {
    let out = run_main(
        "import core.text;\n\
         public function id_text(x: String) -> String begin\n\
             return x;\n\
         end\n\
         let value: String = id_text(\"abc\");\n\
         PAPAR core.text.not_empty_text_i64(value);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1"]);
}

#[test]
fn core_text_emptiness_relationship_helpers() {
    let out = run_main(
        "import core.text;\n\
         PAPAR core.text.same_emptiness_i64(\"\", \"\");\n\
         PAPAR core.text.same_emptiness_i64(\"\", \"abc\");\n\
         PAPAR core.text.same_emptiness_i64(\"abc\", \"\");\n\
         PAPAR core.text.same_emptiness_i64(\"abc\", \"xyz\");\n\
         PAPAR core.text.select_by_empty_i64(\"\", 7, 9);\n\
         PAPAR core.text.select_by_empty_i64(\"abc\", 7, 9);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "0", "0", "1", "7", "9"]);
}

#[test]
fn core_text_emptiness_helpers_with_core_assert() {
    let out = run_main(
        "import core.text;\n\
         import core.assert;\n\
         PAPAR core.assert.eq_i64(core.text.same_emptiness_i64(\"abc\", \"xyz\"), 1);\n\
         PAPAR core.assert.eq_i64(core.text.same_emptiness_i64(\"\", \"abc\"), 0);\n\
         PAPAR core.assert.eq_i64(core.text.select_by_empty_i64(\"\", 7, 9), 7);\n\
         PAPAR core.assert.eq_i64(core.text.select_by_empty_i64(\"abc\", 7, 9), 9);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["1", "1", "1", "1"]);
}
