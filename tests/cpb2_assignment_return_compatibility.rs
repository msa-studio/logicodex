use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct FixtureRoot(PathBuf);
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Record {
    code: String,
    severity: String,
    span: String,
    ms: String,
    en: String,
}

fn artifacts(root: &Path) -> Vec<PathBuf> {
    fn walk(path: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if matches!(
                    p.extension().and_then(|x| x.to_str()),
                    Some("ll" | "bc" | "o" | "obj" | "exe")
                ) || p.file_name().and_then(|x| x.to_str()) == Some("main")
                {
                    out.push(p);
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

fn fixture(source: &str) -> (FixtureRoot, String) {
    let root = std::env::temp_dir().join(format!(
        "logicodex_cpb2b_{}_{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).expect("fresh fixture root");
    let file = root.join("main.ldx");
    std::fs::write(&file, source).expect("fixture source");
    (FixtureRoot(root), file.to_string_lossy().into_owned())
}

fn parse_records(text: &str) -> Vec<Record> {
    let lines: Vec<&str> = text.lines().collect();
    let mut records = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (severity, code) = if let Some(code) = lines[i].strip_prefix("Error: code: ") {
            ("Error", code)
        } else if let Some(code) = lines[i].strip_prefix("code: ") {
            ("Error", code)
        } else {
            i += 1;
            continue;
        };
        assert!(i + 2 < lines.len(), "truncated diagnostic: {text}");
        let span = lines[i + 1]
            .strip_prefix("span: ")
            .expect("diagnostic span")
            .to_string();
        let message = lines[i + 2];
        let (ms, en) = message
            .split_once(" / Error: ")
            .expect("bilingual diagnostic");
        records.push(Record {
            code: code.to_string(),
            severity: severity.to_string(),
            span,
            ms: ms.to_string(),
            en: format!("Error: {en}"),
        });
        i += 3;
    }
    records
}

fn check(source: &str, expected: &[Record], must_fail: bool) {
    let (root_guard, file) = fixture(source);
    let root = &root_guard.0;
    let output = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .current_dir(root)
        .env(
            "LOGICODEX_STD",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib"),
        )
        .arg("check")
        .arg("--dict")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dict/core_map.json"))
        .arg(&file)
        .output()
        .expect("logicodex check");
    assert_eq!(
        output.status.success(),
        !must_fail,
        "unexpected exit: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(parse_records(&text), expected, "raw diagnostics:\n{text}");
    assert!(
        artifacts(root).is_empty(),
        "rejected fixture produced backend artifacts: {:?}",
        artifacts(root)
    );
}

fn mismatch(code: &str, span: &str, ms: &str, en: &str) -> Record {
    Record {
        code: code.into(),
        severity: "Error".into(),
        span: span.into(),
        ms: ms.into(),
        en: en.into(),
    }
}

#[test]
fn assignment_mismatch_preserves_exact_record() {
    check(
        "fn main() -> I64 begin\n    let x: I64 = 1;\n    x = true;\n    return 0;\nend\n",
        &[mismatch(
            "TypeMismatch",
            "file 0:2:5-2:20",
            "Ralat: Jenis tugasan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)",
            "Error: Assignment type mismatch: expected Primitive(I64), got Primitive(Bool)",
        )],
        true,
    );
}

#[test]
fn assignment_match_is_accepted() {
    check(
        "fn main() -> I64 begin\n    let x: I64 = 1;\n    x = 2;\n    return x;\nend\n",
        &[],
        false,
    );
}

#[test]
fn binding_call_result_mismatch_preserves_exact_record() {
    check("fn flag() -> Bool begin\n    return true;\nend\nfn main() -> I64 begin\n    let x: I64 = flag();\n    return 0;\nend\n", &[mismatch("TypeMismatch", "file 0:5:5-5:25", "Ralat: Jenis ikatan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)", "Error: Binding type mismatch: expected Primitive(I64), got Primitive(Bool)")], true);
}

#[test]
fn return_mismatch_preserves_exact_record() {
    check("fn bad() -> I64 begin\n    return true;\nend\nfn main() -> I64 begin\n    return 0;\nend\n", &[mismatch("TypeMismatch", "file 0:2:5-2:17", "Ralat: Jenis pulangan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)", "Error: Return type mismatch: expected Primitive(I64), got Primitive(Bool)")], true);
}

#[test]
fn return_match_is_accepted() {
    check("fn flag() -> Bool begin\n    return true;\nend\nfn main() -> I64 begin\n    return 0;\nend\n", &[], false);
}

#[test]
fn unit_return_value_preserves_exact_record() {
    check("fn bad() -> Unit begin\n    return 1;\nend\nfn main() -> I64 begin\n    return 0;\nend\n", &[mismatch("TypeMismatch", "file 0:2:5-2:14", "Ralat: Fungsi Unit tidak boleh memulangkan nilai jenis Primitive(I64)", "Error: Unit function cannot return a value of type Primitive(I64)"), mismatch("TypeMismatch", "file 0:2:5-2:14", "Ralat: Jenis pulangan tidak sepadan: dijangka Primitive(Unit), diterima Primitive(I64)", "Error: Return type mismatch: expected Primitive(Unit), got Primitive(I64)")], true);
}

#[test]
fn call_result_used_as_value_preserves_exact_record() {
    check("fn noop() -> Unit begin\nend\nfn main() -> I64 begin\n    return noop();\nend\n", &[mismatch("TypeMismatch", "file 0:4:5-4:19", "Ralat: Panggilan `noop` memulangkan Unit dan tidak boleh digunakan sebagai nilai dalam return", "Error: Call `noop` returns Unit and cannot be used as a value in return"), mismatch("TypeMismatch", "file 0:4:5-4:19", "Ralat: Jenis pulangan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Unit)", "Error: Return type mismatch: expected Primitive(I64), got Primitive(Unit)")], true);
}

#[test]
fn rejected_fixture_emits_no_backend_artifacts() {
    check(
        "fn main() -> I64 begin\n    let x: I64 = 1;\n    x = true;\n    return 0;\nend\n",
        &[mismatch(
            "TypeMismatch",
            "file 0:2:5-2:20",
            "Ralat: Jenis tugasan tidak sepadan: dijangka Primitive(I64), diterima Primitive(Bool)",
            "Error: Assignment type mismatch: expected Primitive(I64), got Primitive(Bool)",
        )],
        true,
    );
}
