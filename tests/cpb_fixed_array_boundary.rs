use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_check(name: &str, source: &str) -> (bool, String) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();

    let path = std::env::temp_dir().join(format!(
        "logicodex_cpb_fixed_array_{name}_{}_{}.ldx",
        std::process::id(),
        unique
    ));

    fs::write(&path, source).expect("write test source");

    let output = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .arg("check")
        .arg(&path)
        .output()
        .expect("run logicodex check");

    let mut rendered = String::new();
    rendered.push_str(&String::from_utf8_lossy(&output.stdout));
    rendered.push_str(&String::from_utf8_lossy(&output.stderr));

    let _ = fs::remove_file(&path);

    (output.status.success(), rendered)
}

fn assert_check_ok(name: &str, source: &str) {
    let (ok, output) = run_check(name, source);
    assert!(ok, "{name}: expected check success\n{output}");
}

fn assert_check_type_mismatch(name: &str, source: &str) {
    let (ok, output) = run_check(name, source);

    assert!(!ok, "{name}: expected check failure, got success\n{output}");
    assert!(
        output.contains("code: TypeMismatch"),
        "{name}: expected TypeMismatch diagnostic\n{output}"
    );
    assert!(
        output.contains("span: file"),
        "{name}: expected source span\n{output}"
    );
    assert!(
        !output.contains("span: file 0:0:0-0:0"),
        "{name}: diagnostic still has unknown span\n{output}"
    );
    assert!(
        !output.contains("LLVM module verification failed"),
        "{name}: leaked to LLVM verifier\n{output}"
    );
}

#[test]
fn cpb_fixed_array_subset_is_semantically_proven() {
    assert_check_ok(
        "fixed_array_literal_index",
        r#"
function main() -> I64 begin
    let xs: [I64; 3] = [10, 20, 30];
    return xs[1];
end
"#,
    );

    assert_check_ok(
        "fixed_array_index_assignment",
        r#"
function main() -> I64 begin
    let xs: [I64; 3] = [10, 20, 30];
    xs[1] = 99;
    return xs[1];
end
"#,
    );

    assert_check_type_mismatch(
        "array_literal_length_mismatch",
        r#"
function main() -> I64 begin
    let xs: [I64; 3] = [10, 20];
    return 1;
end
"#,
    );

    assert_check_type_mismatch(
        "array_literal_element_mismatch",
        r#"
function main() -> I64 begin
    let xs: [I64; 2] = [10, true];
    return 1;
end
"#,
    );

    assert_check_type_mismatch(
        "array_index_bool",
        r#"
function main() -> I64 begin
    let xs: [I64; 2] = [10, 20];
    return xs[true];
end
"#,
    );
}

#[test]
fn cpb_slice_parameter_syntax_is_not_a_full_slice_api_claim() {
    assert_check_ok(
        "slice_param_syntax_probe",
        r#"
function first(xs: []I64) -> I64 begin
    return 1;
end

function main() -> I64 begin
    return 1;
end
"#,
    );
}
