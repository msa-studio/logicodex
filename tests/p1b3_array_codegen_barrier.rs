use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_source(name: &str, source: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();

    let path = std::env::temp_dir().join(format!(
        "logicodex_p1b3b_array_barrier_{name}_{}_{}.ldx",
        std::process::id(),
        unique
    ));

    fs::write(&path, source).expect("write test source");
    path
}

fn run_check(path: &std::path::Path) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .arg("check")
        .arg(path)
        .output()
        .expect("run logicodex check");

    let mut rendered = String::new();
    rendered.push_str(&String::from_utf8_lossy(&output.stdout));
    rendered.push_str(&String::from_utf8_lossy(&output.stderr));

    (output.status.success(), rendered)
}

fn run_compile_emit_ir(path: &std::path::Path) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_logicodex"))
        .arg("compile")
        .arg(path)
        .arg("--emit-ir")
        .output()
        .expect("run logicodex compile");

    let mut rendered = String::new();
    rendered.push_str(&String::from_utf8_lossy(&output.stdout));
    rendered.push_str(&String::from_utf8_lossy(&output.stderr));

    (output.status.success(), rendered)
}

fn assert_check_ok_but_compile_barrier(name: &str, source: &str, expected: &str) {
    let path = write_temp_source(name, source);

    let (check_ok, check_output) = run_check(&path);
    assert!(
        check_ok,
        "{name}: expected semantic check success before known codegen barrier\n{check_output}"
    );

    let (compile_ok, compile_output) = run_compile_emit_ir(&path);
    let _ = fs::remove_file(&path);

    assert!(
        !compile_ok,
        "{name}: expected current codegen barrier until array param/return ABI is implemented"
    );
    assert!(
        compile_output.contains(expected),
        "{name}: expected barrier marker {expected:?}\n{compile_output}"
    );
}

#[test]
fn p1b3b_array_param_and_return_remain_codegen_barriers() {
    assert_check_ok_but_compile_barrier(
        "array_param_unused",
        r#"
function len3(xs: [I64; 3]) -> I64 begin
    return 3;
end

function main() -> I64 begin
    let xs: [I64; 3] = [10, 20, 30];
    return len3(xs);
end
"#,
        "Call parameter type does not match function signature",
    );

    assert_check_ok_but_compile_barrier(
        "array_param_index_read",
        r#"
function first(xs: [I64; 3]) -> I64 begin
    return xs[0];
end

function main() -> I64 begin
    let xs: [I64; 3] = [10, 20, 30];
    return first(xs);
end
"#,
        "Call parameter type does not match function signature",
    );

    assert_check_ok_but_compile_barrier(
        "array_return_value",
        r#"
function make() -> [I64; 3] begin
    return [10, 20, 30];
end

function main() -> I64 begin
    let xs: [I64; 3] = make();
    return xs[1];
end
"#,
        "array literals are only supported as typed local initializers",
    );
}
