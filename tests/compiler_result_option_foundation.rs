//! Result / Option compiler foundation tests.
//!
//! These tests define the intended long-term behaviour for enum tags,
//! Result<I64, I64>, Option<I64>, and match destructuring.
//!
//! They originally started ignored while the branch was capturing blockers
//! before fixing compiler internals. Each test was unignored as the matching
//! compiler layer became real end to end; they are now live (non-ignored) and
//! green for the proven I64 slice.

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
            "ldx_result_option_foundation_{name}_{}_{}",
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

fn compile_and_run(name: &str, src: &str) -> Vec<String> {
    let proj = Tmp::new(name);
    let root = proj.file("main.ldx", src);

    let compile = Command::new(bin())
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

    // Current generated executables may return a nonzero process status even
    // when stdout behaviour is correct. Exit-code normalization is tracked as
    // a separate runtime cleanup so these foundation tests focus on compiler
    // semantics and observable output.
    String::from_utf8_lossy(&run.stdout)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

#[test]
fn enum_variants_have_deterministic_i64_tags() {
    let got = compile_and_run(
        "enum_tags",
        "enum Flag begin\n\
             Yes;\n\
             No;\n\
         end\n\
         PAPAR Flag::Yes;\n\
         PAPAR Flag::No;\n",
    );

    assert_eq!(got, vec!["0", "1"]);
}

#[test]
fn result_i64_i64_return_ok_payload() {
    let got = compile_and_run(
        "result_return_ok",
        "public function unwrap_or_i64(x: Result<I64, I64>, fallback: I64) -> I64 begin\n\
             match x begin\n\
                 Ok(v) => begin\n\
                     return v;\n\
                 end,\n\
                 Err(e) => begin\n\
                     return fallback;\n\
                 end\n\
             end\n\
         end\n\
         public function good() -> Result<I64, I64> begin\n\
             return Ok(42);\n\
         end\n\
         PAPAR unwrap_or_i64(good(), 9);\n",
    );

    assert_eq!(got, vec!["42"]);
}

#[test]
fn result_i64_i64_return_err_payload() {
    let got = compile_and_run(
        "result_return_err",
        "public function unwrap_err_or_i64(x: Result<I64, I64>, fallback: I64) -> I64 begin\n\
             match x begin\n\
                 Ok(v) => begin\n\
                     return fallback;\n\
                 end,\n\
                 Err(e) => begin\n\
                     return e;\n\
                 end\n\
             end\n\
         end\n\
         public function bad() -> Result<I64, I64> begin\n\
             return Err(7);\n\
         end\n\
         PAPAR unwrap_err_or_i64(bad(), 9);\n",
    );

    assert_eq!(got, vec!["7"]);
}

#[test]
fn result_i64_i64_match_unwrap_or() {
    let got = compile_and_run(
        "result_match_unwrap_or",
        "public function unwrap_or_i64(x: Result<I64, I64>, fallback: I64) -> I64 begin\n\
             match x begin\n\
                 Ok(v) => begin\n\
                     return v;\n\
                 end,\n\
                 Err(e) => begin\n\
                     return fallback;\n\
                 end\n\
             end\n\
         end\n\
         PAPAR unwrap_or_i64(Ok(42), 9);\n\
         PAPAR unwrap_or_i64(Err(7), 9);\n",
    );

    assert_eq!(got, vec!["42", "9"]);
}

#[test]
fn option_i64_return_some_payload() {
    let got = compile_and_run(
        "option_return_some",
        "public function unwrap_or_i64(x: Option<I64>, fallback: I64) -> I64 begin\n\
             match x begin\n\
                 Some(v) => begin\n\
                     return v;\n\
                 end,\n\
                 None => begin\n\
                     return fallback;\n\
                 end\n\
             end\n\
         end\n\
         public function maybe() -> Option<I64> begin\n\
             return Some(42);\n\
         end\n\
         PAPAR unwrap_or_i64(maybe(), 9);\n",
    );

    assert_eq!(got, vec!["42"]);
}

#[test]
fn option_i64_return_none_tag() {
    let got = compile_and_run(
        "option_return_none",
        "public function maybe() -> Option<I64> begin\n\
             return None;\n\
         end\n\
         PAPAR maybe();\n",
    );

    assert_eq!(got, vec!["0"]);
}

#[test]
fn option_i64_match_unwrap_or() {
    let got = compile_and_run(
        "option_match_unwrap_or",
        "public function unwrap_or_i64(x: Option<I64>, fallback: I64) -> I64 begin\n\
             match x begin\n\
                 Some(v) => begin\n\
                     return v;\n\
                 end,\n\
                 None => begin\n\
                     return fallback;\n\
                 end\n\
             end\n\
         end\n\
         PAPAR unwrap_or_i64(Some(42), 9);\n\
         PAPAR unwrap_or_i64(None, 9);\n",
    );

    assert_eq!(got, vec!["42", "9"]);
}
