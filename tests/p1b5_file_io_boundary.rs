use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_source(name: &str, source: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();

    let path = std::env::temp_dir().join(format!(
        "logicodex_p1b5_file_io_boundary_{name}_{}_{}.ldx",
        std::process::id(),
        unique
    ));

    fs::write(&path, source).expect("write test source");
    path
}

fn run_logicodex(args: &[&str], path: &std::path::Path) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .args(args)
        .arg(path)
        .output()
        .expect("run logicodex");

    let mut rendered = String::new();
    rendered.push_str(&String::from_utf8_lossy(&output.stdout));
    rendered.push_str(&String::from_utf8_lossy(&output.stderr));

    (output.status.success(), rendered)
}

fn assert_check_fails(name: &str, source: &str, expected: &str) {
    let path = write_temp_source(name, source);

    let (ok, output) = run_logicodex(&["check"], &path);
    let _ = fs::remove_file(&path);

    assert!(!ok, "{name}: expected check failure, got success\n{output}");
    assert!(
        output.contains(expected),
        "{name}: expected marker {expected:?}\n{output}"
    );
}

fn assert_compile_ok(name: &str, source: &str) {
    let path = write_temp_source(name, source);

    let (ok, output) = run_logicodex(&["compile", "--emit-ir"], &path);
    let _ = fs::remove_file(&path);

    assert!(ok, "{name}: expected compile success\n{output}");
}

#[test]
fn p1b5_std_file_and_std_io_are_not_public_modules_yet() {
    assert_check_fails(
        "std_file_import_missing",
        r#"
import std.file;

function main() -> I64 begin
    return 1;
end
"#,
        "module `std.file` not found",
    );

    assert_check_fails(
        "std_io_import_missing",
        r#"
import std.io;

function main() -> I64 begin
    return 1;
end
"#,
        "module `std.io` not found",
    );
}

#[test]
fn p1b5_papar_is_builtin_output_not_std_io_api() {
    assert_compile_ok(
        "papar_builtin_output_smoke",
        r#"
PAPAR 42;
"#,
    );

    assert_check_fails(
        "std_io_print_not_claimed",
        r#"
import std.io;

function main() -> I64 begin
    return std.io.print_i64(42);
end
"#,
        "module `std.io` not found",
    );
}

#[test]
fn p1b5_filehandle_is_not_a_contract_backed_file_api_yet() {
    assert_check_fails(
        "filehandle_named_type_probe",
        r#"
function main() -> I64 begin
    let h: FileHandle = 0;
    return 1;
end
"#,
        "code: TypeMismatch",
    );
}

#[test]
fn p1b5_contract_files_are_intentionally_absent_until_policy_exists() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    assert!(
        !root.join("lib/std/file.std.toml").exists(),
        "std.file contract must not appear before P1-B5 policy/API is implemented"
    );
    assert!(
        !root.join("lib/std/io.std.toml").exists(),
        "std.io contract must not appear before P1-B5 policy/API is implemented"
    );
}
