//! Collections Foundation Stage 0 target tests.
//!
//! These tests define the smallest generic compiler capability needed before
//! `core.collections` can honestly exist as a contract-backed stdlib module.
//!
//! Scope intentionally limited:
//! - fixed local arrays
//! - I64 elements only at first
//! - array literal construction
//! - index read
//! - index assignment
//! - no heap Vec/List
//! - no Buffer runtime/profile semantics
//! - no slice passing yet

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
    fn new(name: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let uniq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "ldx_collections_{name}_{}_{}",
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

fn compile_and_run(src: &str) -> Vec<String> {
    let proj = Tmp::new("proj");
    let root = proj.file("main.ldx", src);

    let compile = Command::new(bin())
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
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
fn fixed_i64_array_literal_index_read() {
    let got = compile_and_run(
        "let xs: [I64; 3] = [10, 20, 30];\n\
         PAPAR xs[0];\n\
         PAPAR xs[1];\n\
         PAPAR xs[2];\n",
    );

    assert_eq!(got, vec!["10", "20", "30"]);
}

#[test]
fn fixed_i64_array_index_assignment() {
    let got = compile_and_run(
        "let xs: [I64; 3] = [10, 20, 30];\n\
         xs[1] = 99;\n\
         PAPAR xs[0];\n\
         PAPAR xs[1];\n\
         PAPAR xs[2];\n",
    );

    assert_eq!(got, vec!["10", "99", "30"]);
}
