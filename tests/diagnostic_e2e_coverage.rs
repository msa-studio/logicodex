//! Diagnostic end-to-end coverage tests for semantic gate.

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
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let uniq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "ldx_diagnostic_e2e_{name}_{}_{uniq}",
            std::process::id()
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

fn check_fail(src: &str) -> String {
    let proj = Tmp::new("fail");
    let root = proj.file("main.ldx", src);

    let out = Command::new(bin())
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
        .arg("check")
        .arg(&root)
        .output()
        .expect("spawn check");

    assert!(
        !out.status.success(),
        "check unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn assert_exact_code(output: &str, expected: &str) {
    let codes: Vec<&str> = output
        .lines()
        .filter_map(|line| {
            line.strip_prefix("Error: code: ")
                .or_else(|| line.strip_prefix("code: "))
        })
        .collect();
    assert_eq!(
        codes,
        vec![expected],
        "unexpected diagnostic records: {output}"
    );
}
fn assert_nonzero_span(output: &str, expected_line: u64) {
    let marker = "span: file ";
    let start = output.find(marker).expect("missing span recovery");
    let tokens: Vec<&str> = output[start + marker.len()..]
        .split_whitespace()
        .take(3)
        .collect();
    let (file, begin_text, end_text) =
        if tokens[0].matches(':').count() >= 3 && tokens[0].contains('-') {
            let (file, coords) = tokens[0].split_once(':').expect("malformed span");
            let (begin, end) = coords.split_once('-').expect("malformed span coordinates");
            (file, begin.to_string(), end.to_string())
        } else if tokens.len() >= 3 && tokens[1].ends_with(':') {
            let (begin_line, end_part) = tokens[2]
                .split_once('-')
                .expect("malformed span coordinates");
            (
                tokens[0],
                format!("{}{}", tokens[1], begin_line),
                end_part.to_string(),
            )
        } else {
            panic!("malformed span record");
        };
    assert!(!file.is_empty(), "span file is empty");
    let parse = |part: &str| -> (u64, u64) {
        let values: Vec<u64> = part
            .split(':')
            .map(|value| value.parse().expect("non-numeric span"))
            .collect();
        assert_eq!(values.len(), 2, "malformed span coordinates: {part}");
        (values[0], values[1])
    };
    let begin = parse(&begin_text);
    let end = parse(&end_text);
    assert_eq!(begin.0, expected_line, "span begins on unexpected line");
    assert_eq!(end.0, expected_line, "span ends on unexpected line");
    assert!(begin.1 > 0, "span begins at zero column");
    assert!(end.1 > begin.1, "span does not advance columns");
}

fn check_ok(src: &str) {
    let proj = Tmp::new("ok");
    let root = proj.file("main.ldx", src);

    let out = Command::new(bin())
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
        .arg("check")
        .arg(&root)
        .output()
        .expect("spawn check");

    assert!(
        out.status.success(),
        "check failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn type_mismatch_has_diagnostic_code_and_span() {
    let output = check_fail(
        "function main() -> I64 begin\n    let x: I64 = \"hello\";\n    return 0;\nend\n",
    );
    assert_exact_code(&output, "TypeMismatch");
    assert_nonzero_span(&output, 2);
}

#[test]
fn unknown_name_has_diagnostic_code_and_span() {
    let output = check_fail("function main() -> I64 begin\n    return undefined_var;\nend\n");
    assert_exact_code(&output, "UnknownName");
    assert_nonzero_span(&output, 2);
}

#[test]
fn unknown_function_has_diagnostic_code_and_span() {
    let output = check_fail("function main() -> I64 begin\n    return undefined_func(1);\nend\n");
    assert_exact_code(&output, "UnknownFunction");
    assert_nonzero_span(&output, 2);
}

#[test]
fn call_arity_mismatch_has_diagnostic_code_and_span() {
    let output = check_fail(
        "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\n\nfunction main() -> I64 begin\n    return add(1);\nend\n",
    );
    assert_exact_code(&output, "TypeMismatch");
    assert_nonzero_span(&output, 6);
}

#[test]
fn missing_return_has_diagnostic_code_and_span() {
    let output = check_fail("function foo() -> I64 begin\n    let x = 1;\nend\n");
    assert_exact_code(&output, "TypeMismatch");
    assert_nonzero_span(&output, 1);
}

#[test]
fn valid_assignment_and_return_still_pass() {
    check_ok("function main() -> I64 begin\n    let x: I64 = 42;\n    return x;\nend\n");
}

#[test]
fn valid_function_call_still_passes() {
    check_ok(
        "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\n\nfunction main() -> I64 begin\n    return add(10, 20);\nend\n",
    );
}
