use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_program(name: &str, source: &str) -> String {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();

    let tmpdir = std::env::temp_dir().join(format!(
        "logicodex_std_path_{name}_{}_{}",
        std::process::id(),
        unique
    ));
    fs::create_dir_all(&tmpdir).expect("create temp dir");

    let source_path = tmpdir.join("main.ldx");
    fs::write(&source_path, source).expect("write test source");

    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let stdlib_root = repo_root.join("lib");

    let compile = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .arg("compile")
        .arg("--emit-ir")
        .arg(&source_path)
        .env("LOGICODEX_STD", &stdlib_root)
        .current_dir(&repo_root)
        .output()
        .expect("run logicodex compile");

    let mut rendered = String::new();
    rendered.push_str(&String::from_utf8_lossy(&compile.stdout));
    rendered.push_str(&String::from_utf8_lossy(&compile.stderr));

    assert!(
        compile.status.success(),
        "{name}: compile failed\n{rendered}"
    );

    let exe_path = source_path.with_extension("");
    let run = Command::new(&exe_path)
        .current_dir(&tmpdir)
        .output()
        .expect("run compiled Logicodex program");

    let stdout = String::from_utf8_lossy(&run.stdout).to_string();
    let _ = fs::remove_dir_all(&tmpdir);

    stdout
}

fn assert_stdout_tokens(name: &str, expr: &str, expected: &[&str]) {
    let source = format!("import std.path;\nPAPAR {expr};\n");
    let stdout = run_program(name, &source);
    let tokens: Vec<&str> = stdout.split_whitespace().collect();

    assert_eq!(
        tokens, expected,
        "{name}: stdout oracle mismatch for {expr}\nstdout:\n{stdout}"
    );
}

#[test]
fn std_path_lexical_foundation_contract_cases() {
    assert_stdout_tokens("empty_path", "std.path.is_empty_path_i64(\"\")", &["1"]);
    assert_stdout_tokens(
        "nonempty_path",
        "std.path.is_empty_path_i64(\"src/main.ldx\")",
        &["0"],
    );
    assert_stdout_tokens(
        "not_empty_path",
        "std.path.not_empty_path_i64(\"src/main.ldx\")",
        &["1"],
    );
    assert_stdout_tokens(
        "not_empty_path_false",
        "std.path.not_empty_path_i64(\"\")",
        &["0"],
    );
    assert_stdout_tokens(
        "same_emptiness_empty",
        "std.path.same_emptiness_path_i64(\"\", \"\")",
        &["1"],
    );
    assert_stdout_tokens(
        "same_emptiness_mixed",
        "std.path.same_emptiness_path_i64(\"\", \"src/main.ldx\")",
        &["0"],
    );
    assert_stdout_tokens(
        "same_emptiness_nonempty",
        "std.path.same_emptiness_path_i64(\"src/main.ldx\", \"docs/readme.md\")",
        &["1"],
    );
    assert_stdout_tokens(
        "select_empty",
        "std.path.select_by_empty_path_i64(\"\", 7, 9)",
        &["7"],
    );
    assert_stdout_tokens(
        "select_nonempty",
        "std.path.select_by_empty_path_i64(\"src/main.ldx\", 7, 9)",
        &["9"],
    );
}
