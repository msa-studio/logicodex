//! core.prelude CPB Phase 1 acceptance tests.
//!
//! Uses the REAL shipped `lib/core/prelude.ldx` by pointing LOGICODEX_STD at the
//! repo's `lib/` dir. Behaviour-level e2e: compile a main.ldx that imports
//! core.prelude, run it, assert printed output.

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
        dir.push(format!("ldx_coreprelude_{name}_{}_{}", std::process::id(), uniq));
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

fn run_main(src: &str) -> Vec<String> {
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
        "compile failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let exe = root.with_extension("");
    let run = Command::new(&exe).output().expect("run compiled exe");

    assert!(
        run.stderr.is_empty(),
        "run emitted stderr:\nstdout:\n{}\nstderr:\n{}\nstatus:\n{:?}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
        run.status
    );

    String::from_utf8_lossy(&run.stdout)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

#[test]
fn core_prelude_scalar_helpers() {
    let got = run_main(
        "import core.prelude;\n\
         PAPAR core.prelude.id_i64(42);\n\
         PAPAR core.prelude.zero_i64();\n\
         PAPAR core.prelude.one_i64();\n\
         PAPAR core.prelude.truthy_i64(5);\n\
         PAPAR core.prelude.truthy_i64(-3);\n\
         PAPAR core.prelude.truthy_i64(0);\n\
         PAPAR core.prelude.fallback_i64(0, 9);\n\
         PAPAR core.prelude.fallback_i64(7, 9);\n",
    );

    assert_eq!(got, vec!["42", "0", "1", "1", "1", "0", "9", "7"]);
}

#[test]
fn core_prelude_with_core_assert() {
    let got = run_main(
        "import core.prelude;\n\
         import core.assert;\n\
         PAPAR core.assert.eq_i64(core.prelude.zero_i64(), 0);\n\
         PAPAR core.assert.eq_i64(core.prelude.one_i64(), 1);\n\
         PAPAR core.assert.eq_i64(core.prelude.fallback_i64(0, 8), 8);\n",
    );

    assert_eq!(got, vec!["1", "1", "1"]);
}
