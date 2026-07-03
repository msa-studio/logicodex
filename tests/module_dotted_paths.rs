//! Module System Stage 0.5 -- dotted module paths (`import a.b;` + `a.b.f()`).
//!
//! Behaviour-level e2e: dotted module names must parse end-to-end (import side
//! and call side) and resolve via the existing dotted-aware loader + mangling.
//! Uses a non-reserved dotted name (`geo.vec`) so this is independent of the
//! stdlib std-root resolution (a separate change).

use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_logicodex")
}

struct Project {
    dir: PathBuf,
}
impl Project {
    fn new(name: &str) -> Self {
        let mut dir = std::env::temp_dir();
        dir.push(format!("ldx_dotted_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create project dir");
        Project { dir }
    }
    fn file(&self, rel: &str, src: &str) -> PathBuf {
        let path = self.dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(&path, src).expect("write file");
        path
    }
    fn compile_and_run(&self, root: &PathBuf) -> (i32, String) {
        let compile = Command::new(bin())
            .arg("compile")
            .arg("--emit-ir")
            .arg(root)
            .output()
            .expect("spawn logicodex compile");
        assert!(
            compile.status.success(),
            "compile failed:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let exe = root.with_extension("");
        let run = Command::new(&exe).output().expect("run compiled exe");
        (
            run.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&run.stdout).into_owned(),
        )
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// Two-segment dotted module: `geo/vec.ldx`, imported as `geo.vec`, called as
// `geo.vec.add(2, 3)`. Proves dotted import + dotted qualified call.
#[test]
fn dotted_two_segment_import_and_call() {
    let p = Project::new("two_seg");
    p.file(
        "geo/vec.ldx",
        "public function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\n",
    );
    let root = p.file("main.ldx", "import geo.vec;\nPAPAR geo.vec.add(2, 3);\n");
    let (_code, out) = p.compile_and_run(&root);
    assert_eq!(
        out.trim(),
        "5",
        "geo.vec.add(2,3) across a dotted path prints 5"
    );
}

// Regression: single-segment modules must still work after the parser change.
#[test]
fn single_segment_still_works() {
    let p = Project::new("one_seg");
    p.file(
        "math.ldx",
        "public function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\n",
    );
    let root = p.file("main.ldx", "import math;\nPAPAR math.add(2, 3);\n");
    let (_code, out) = p.compile_and_run(&root);
    assert_eq!(out.trim(), "5", "single-segment math.add still prints 5");
}
