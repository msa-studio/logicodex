//! Diagnostics fail-fast regression tests.
//!
//! Unsupported compiler/codegen paths must not silently compile into successful
//! default values such as `0`.

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
            "ldx_diag_fail_fast_{name}_{}_{}",
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

fn compile_fail(src: &str) -> String {
    let proj = Tmp::new("proj");
    let root = proj.file("main.ldx", src);

    let out = Command::new(bin())
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
        !out.status.success(),
        "compile unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn array_literal_expression_does_not_silently_lower_to_zero() {
    let output = compile_fail("PAPAR [1, 2, 3];\n");

    assert!(
        output.contains("array literal") || output.contains("ArrayLiteral"),
        "expected array literal fail-fast diagnostic, got:\n{output}"
    );

    assert!(
        output.contains("unsupported codegen path") || output.contains("not supported"),
        "expected unsupported-path wording, got:\n{output}"
    );
}
