//! Span recovery verification tests for diagnostic end-to-end flow.

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
            "ldx_diagnostic_span_{name}_{}_{uniq}",
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

    assert!(!out.status.success());

    format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
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

#[test]
fn type_mismatch_span_is_nonzero() {
    let output = check_fail(
        "function main() -> I64 begin\n    let x: I64 = \"hello\";\n    return 0;\nend\n",
    );

    assert_nonzero_span(&output, 2);
}

/// Verify that unknown name errors include nonzero spans
#[test]
fn unknown_name_span_is_nonzero() {
    let output = check_fail("function main() -> I64 begin\n    return undefined_var;\nend\n");

    assert_nonzero_span(&output, 2);
}

/// Verify that call argument mismatch errors include nonzero spans
#[test]
fn call_argument_mismatch_span_is_nonzero() {
    let output = check_fail(
        "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\n\nfunction main() -> I64 begin\n    return add(1, \"wrong\");\nend\n",
    );

    assert_nonzero_span(&output, 6);
}

/// Verify that missing return errors include nonzero spans
#[test]
fn missing_return_span_is_nonzero() {
    let output = check_fail("function foo() -> I64 begin\n    let x = 1;\nend\n");

    assert_nonzero_span(&output, 1);
}
