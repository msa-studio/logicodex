//! core.math Stage 0 acceptance tests.
//!
//! Uses the REAL shipped `lib/core/math.ldx` by pointing LOGICODEX_STD at the
//! repo's `lib/` dir. Behaviour-level e2e: compile a main.ldx that imports
//! core.math, run it, assert the printed output.

use std::path::PathBuf;
use std::process::Command;

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
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let uniq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "ldx_coremath_{name}_{}_{}",
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

// All eight first-wave functions, proven in one compiled program.
#[test]
fn core_math_first_wave() {
    let out = run_main(
        "import core.math;\n\
         PAPAR core.math.abs_i64(-5);\n\
         PAPAR core.math.min_i64(2, 7);\n\
         PAPAR core.math.max_i64(2, 7);\n\
         PAPAR core.math.clamp_i64(12, 0, 10);\n\
         PAPAR core.math.sign_i64(-3);\n\
         PAPAR core.math.is_even(4);\n\
         PAPAR core.math.is_odd(7);\n\
         PAPAR core.math.pow_i64(2, 10);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(
        got,
        vec!["5", "2", "7", "10", "-1", "1", "1", "1024"],
        "core.math first-wave outputs"
    );
}

// abs handles the non-negative branch too.
#[test]
fn core_math_abs_positive_and_zero() {
    let out = run_main(
        "import core.math;\n\
         PAPAR core.math.abs_i64(5);\n\
         PAPAR core.math.abs_i64(0);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(got, vec!["5", "0"]);
}

// clamp passes values already inside the range through unchanged.
#[test]
fn core_math_clamp_in_range() {
    let out = run_main("import core.math;\nPAPAR core.math.clamp_i64(5, 0, 10);\n");
    assert_eq!(out.split_whitespace().next(), Some("5"));
}

// Later Stage 0 helpers: factorial, gcd, and lcm.
#[test]
fn core_math_factorial_gcd_lcm_helpers() {
    let out = run_main(
        "import core.math;\n\
         PAPAR core.math.factorial_i64(0);\n\
         PAPAR core.math.factorial_i64(5);\n\
         PAPAR core.math.factorial_i64(-3);\n\
         PAPAR core.math.gcd_i64(54, 24);\n\
         PAPAR core.math.gcd_i64(-48, 18);\n\
         PAPAR core.math.gcd_i64(0, 9);\n\
         PAPAR core.math.lcm_i64(6, 8);\n\
         PAPAR core.math.lcm_i64(-6, 8);\n\
         PAPAR core.math.lcm_i64(0, 8);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(
        got,
        vec!["1", "120", "0", "6", "6", "9", "24", "24", "0"],
        "core.math factorial/gcd/lcm outputs"
    );
}

// Arithmetic and predicate convenience helpers.
#[test]
fn core_math_arithmetic_predicate_helpers() {
    let out = run_main(
        "import core.math;\n\
         PAPAR core.math.square_i64(7);\n\
         PAPAR core.math.square_i64(-4);\n\
         PAPAR core.math.cube_i64(3);\n\
         PAPAR core.math.cube_i64(-3);\n\
         PAPAR core.math.is_positive(5);\n\
         PAPAR core.math.is_positive(0);\n\
         PAPAR core.math.is_negative(-5);\n\
         PAPAR core.math.is_negative(0);\n\
         PAPAR core.math.between_i64(5, 1, 10);\n\
         PAPAR core.math.between_i64(0, 1, 10);\n\
         PAPAR core.math.between_i64(11, 1, 10);\n",
    );
    let got: Vec<&str> = out.split_whitespace().collect();
    assert_eq!(
        got,
        vec!["49", "16", "27", "-27", "1", "0", "1", "0", "1", "0", "0"],
        "core.math arithmetic/predicate helper outputs"
    );
}
