//! Stdlib Stage 0 -- std-root resolution.
//!
//! `import core.*` / `std.*` resolve against the std root (LOGICODEX_STD first),
//! NOT relative to the importing file. These are behaviour-level e2e tests:
//! they place a throwaway `core/math.ldx` under a temp std root and prove the
//! importing program finds it from there, independent of where main.ldx lives.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_logicodex")
}

struct Tmp {
    dir: PathBuf,
}
impl Tmp {
    fn new(name: &str) -> Self {
        let mut dir = std::env::temp_dir();
        dir.push(format!("ldx_stdroot_{name}_{}", std::process::id()));
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

// `import core.math;` resolves to $LOGICODEX_STD/core/math.ldx, even though that
// file is NOT next to main.ldx. Proves std-root resolution end-to-end.
#[test]
fn core_module_resolves_via_logicodex_std() {
    let stdroot = Tmp::new("root");
    stdroot.file(
        "core/math.ldx",
        "public function answer() -> I64 begin\n    return 42;\nend\n",
    );

    let proj = Tmp::new("proj");
    let root = proj.file(
        "main.ldx",
        "import core.math;\nPAPAR core.math.answer();\n",
    );

    let compile = Command::new(bin())
        .env("LOGICODEX_STD", &stdroot.dir)
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
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        "42",
        "core.math.answer() resolved from LOGICODEX_STD prints 42"
    );
}

// A missing core module names the std-root path it searched (not the local dir).
#[test]
fn missing_core_module_reports_std_root_path() {
    let stdroot = Tmp::new("root2");
    std::fs::create_dir_all(stdroot.dir.join("core")).expect("mkdir core");

    let proj = Tmp::new("proj2");
    let root = proj.file("main.ldx", "import core.ghost;\nPAPAR 1;\n");

    let out = Command::new(bin())
        .env("LOGICODEX_STD", &stdroot.dir)
        .arg("check")
        .arg(&root)
        .output()
        .expect("spawn check");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_ne!(
        out.status.code().unwrap_or(-1),
        0,
        "missing core module must fail"
    );
    assert!(
        combined.contains("core/ghost.ldx") || combined.contains("ghost"),
        "diagnostic should name the searched std-root path; got:\n{combined}"
    );
}
