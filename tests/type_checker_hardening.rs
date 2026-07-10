//! P0 type checker hardening regression tests.

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
            "ldx_type_checker_{name}_{}_{}",
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
fn local_function_call_argument_count_mismatch_fails() {
    let output = check_fail(
        "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\nPAPAR add(1);\n",
    );

    assert!(
        output.contains("argument count mismatch") || output.contains("bilangan argumen"),
        "expected argument count diagnostic, got:\n{output}"
    );
}

#[test]
fn local_function_call_argument_type_mismatch_fails() {
    let output =
        check_fail("function id(a: I64) -> I64 begin\n    return a;\nend\nPAPAR id(true);\n");

    assert!(
        output.contains("argument 1 type mismatch") || output.contains("jenis argumen"),
        "expected argument type diagnostic, got:\n{output}"
    );
}

#[test]
fn local_function_call_return_type_flows_to_declared_assignment() {
    let output = check_fail(
        "function flag() -> Bool begin\n    return true;\nend\nlet x: I64 = flag();\nPAPAR 1;\n",
    );

    assert!(
        output.contains("Binding type mismatch") || output.contains("Jenis ikatan"),
        "expected declared assignment mismatch using actual call return type, got:\n{output}"
    );
}

#[test]
fn local_function_call_with_matching_args_still_passes() {
    check_ok(
        "function add(a: I64, b: I64) -> I64 begin\n    return a + b;\nend\nPAPAR add(1, 2);\n",
    );
}

#[test]
fn function_return_expression_type_mismatch_fails() {
    let output = check_fail("function bad() -> I64 begin\n    return true;\nend\nPAPAR 1;\n");

    assert!(
        output.contains("Return type mismatch") || output.contains("Jenis pulangan"),
        "expected return type mismatch diagnostic, got:\n{output}"
    );
}

#[test]
fn function_return_expression_with_matching_type_still_passes() {
    check_ok("function flag() -> Bool begin\n    return true;\nend\nPAPAR 1;\n");
}

#[test]
fn assignment_type_mismatch_fails() {
    let output = check_fail("let x: I64 = 1;\nx = true;\nPAPAR 1;\n");

    assert!(
        output.contains("Assignment type mismatch") || output.contains("Jenis tugasan"),
        "expected assignment type mismatch diagnostic, got:\n{output}"
    );
}

#[test]
fn assignment_type_match_still_passes() {
    check_ok("let x: I64 = 1;\nx = 2;\nPAPAR x;\n");
}

#[test]
fn arithmetic_bool_operands_fail() {
    let output = check_fail("PAPAR true + false;\n");

    assert!(
        output.contains("Invalid binary operator operand types")
            || output.contains("operand operator binari"),
        "expected arithmetic operator operand diagnostic, got:\n{output}"
    );
}

#[test]
fn logical_integer_operands_fail() {
    let output = check_fail("PAPAR 1 && 2;\n");

    assert!(
        output.contains("Invalid binary operator operand types")
            || output.contains("operand operator binari"),
        "expected logical operator operand diagnostic, got:\n{output}"
    );
}

#[test]
fn unary_not_integer_operand_fails() {
    let output = check_fail("PAPAR !1;\n");

    assert!(
        output.contains("Invalid unary operator operand type")
            || output.contains("operand operator unari"),
        "expected unary operator operand diagnostic, got:\n{output}"
    );
}

#[test]
fn arithmetic_integer_operands_still_pass() {
    check_ok("PAPAR 1 + 2 * 3;\n");
}

#[test]
fn logical_bool_operands_still_pass() {
    check_ok("PAPAR true && false;\n");
}

#[test]
fn literal_assignment_target_fails() {
    let output = check_fail("1 = 2;\nPAPAR 1;\n");

    assert!(
        output.contains("Assignment target is not writable") || output.contains("Sasaran tugasan"),
        "expected non-writable assignment target diagnostic, got:\n{output}"
    );
}

#[test]
fn call_result_assignment_target_fails() {
    let output =
        check_fail("function id(a: I64) -> I64 begin\n    return a;\nend\nid(1) = 2;\nPAPAR 1;\n");

    assert!(
        output.contains("Assignment target is not writable") || output.contains("Sasaran tugasan"),
        "expected non-writable call-result assignment target diagnostic, got:\n{output}"
    );
}

#[test]
fn binary_expression_assignment_target_fails() {
    let output = check_fail("(1 + 2) = 3;\nPAPAR 1;\n");

    assert!(
        output.contains("Assignment target is not writable") || output.contains("Sasaran tugasan"),
        "expected non-writable binary-expression assignment target diagnostic, got:\n{output}"
    );
}

#[test]
fn index_non_array_base_fails() {
    let output = check_fail("let x: I64 = 1;\nPAPAR x[0];\n");

    assert!(
        output.contains("Index base must be a local fixed array") || output.contains("Asas indeks"),
        "expected index base diagnostic, got:\n{output}"
    );
}

#[test]
fn index_non_integer_index_fails() {
    let output = check_fail("let xs: [I64; 3] = [1, 2, 3];\nPAPAR xs[true];\n");

    assert!(
        output.contains("Array index must be an integer") || output.contains("Indeks tatasusunan"),
        "expected index type diagnostic, got:\n{output}"
    );
}

#[test]
fn mixed_array_literal_element_types_fail() {
    let output = check_fail("let xs: [I64; 3] = [1, true, 3];\nPAPAR 1;\n");

    assert!(
        output.contains("Array literal element") || output.contains("elemen literal tatasusunan"),
        "expected array literal element diagnostic, got:\n{output}"
    );
}

#[test]
fn fixed_array_index_read_and_write_still_pass() {
    check_ok("let xs: [I64; 3] = [1, 2, 3];\nxs[1] = 99;\nPAPAR xs[1];\n");
}

#[test]
fn non_unit_function_missing_return_fails() {
    let output = check_fail("function bad() -> I64 begin\n    let x: I64 = 1;\nend\nPAPAR 1;\n");

    assert!(
        output.contains("guaranteed return path") || output.contains("laluan return yang dijamin"),
        "expected missing return diagnostic, got:\n{output}"
    );
}

#[test]
fn non_unit_function_with_return_still_passes() {
    check_ok("function good() -> I64 begin\n    return 1;\nend\nPAPAR good();\n");
}

#[test]
fn unit_function_without_return_still_passes() {
    check_ok("function side_effect() begin\n    let x: I64 = 1;\nend\nPAPAR 1;\n");
}

#[test]
fn non_unit_function_with_explicit_return_still_passes() {
    check_ok(
        r#"
function main() -> I64 begin
    return 42;
end
"#,
    );
}
#[test]
fn exhaustive_result_match_satisfies_missing_return() {
    check_ok(
        "function unwrap(x: Result<I64, I64>, fallback: I64) -> I64 begin\n\
             match x begin\n\
                 Ok(v) => begin\n\
                     return v;\n\
                 end,\n\
                 Err(e) => begin\n\
                     return fallback;\n\
                 end\n\
             end\n\
         end\n\
         PAPAR unwrap(Ok(7), 9);\n",
    );
}

#[test]
fn non_exhaustive_result_match_still_fails_missing_return() {
    let output = check_fail(
        "function unwrap_bad(x: Result<I64, I64>) -> I64 begin\n\
             match x begin\n\
                 Ok(v) => begin\n\
                     return v;\n\
                 end\n\
             end\n\
         end\n\
         PAPAR unwrap_bad(Ok(7));\n",
    );

    assert!(
        output.contains("guaranteed return path") || output.contains("laluan return yang dijamin"),
        "expected missing return diagnostic, got:\n{output}"
    );
}

#[test]
fn field_access_on_non_struct_base_fails() {
    let output = check_fail(
        "function bad() -> I64 begin\n\
             let x: I64 = 7;\n\
             return x.y;\n\
         end\n\
         PAPAR bad();\n",
    );

    assert!(
        output.contains("Field access base must be a struct")
            || output.contains("Asas akses medan mesti struct"),
        "expected non-struct field access diagnostic, got:\n{output}"
    );
}

#[test]
fn unknown_struct_field_fails() {
    let output = check_fail(
        "struct Point begin\n\
             x: I64;\n\
         end\n\
         function bad(p: Point) -> I64 begin\n\
             return p.y;\n\
         end\n\
         PAPAR 1;\n",
    );

    assert!(
        output.contains("Struct field `y` was not found")
            || output.contains("Medan struct `y` tidak ditemui"),
        "expected unknown struct field diagnostic, got:\n{output}"
    );
}

#[test]
fn known_struct_field_still_passes() {
    check_ok(
        "struct Point begin\n\
             x: I64;\n\
             y: I64;\n\
         end\n\
         function good(p: Point) -> I64 begin\n\
             return p.x;\n\
         end\n\
         PAPAR 1;\n",
    );
}

#[test]
fn duplicate_struct_field_fails() {
    let output = check_fail(
        r#"
struct Bad begin
    x: I64;
    x: I64;
end

PAPAR 1;
"#,
    );

    assert!(
        output.contains("Struct field `x` is declared more than once")
            || output.contains("Medan struct `x` diisytihar lebih daripada sekali"),
        "expected duplicate struct field diagnostic, got:\n{output}"
    );
}

#[test]
fn unknown_struct_field_type_fails() {
    let output = check_fail(
        r#"
struct Bad begin
    x: MissingType;
end

PAPAR 1;
"#,
    );

    assert!(
        output.contains("Struct field `Bad.x` uses an unknown type")
            || output.contains("Medan struct `Bad.x` menggunakan jenis yang tidak diketahui"),
        "expected unknown struct field type diagnostic, got:\n{output}"
    );
}

#[test]
fn recursive_struct_by_value_fails() {
    let output = check_fail(
        r#"
struct Node begin
    next: Node;
end

PAPAR 1;
"#,
    );

    assert!(
        output.contains("Struct field `Node.next` uses an unknown type")
            || output.contains("Struct `Node` contains recursive by-value field `next`")
            || output.contains("Medan struct `Node.next` menggunakan jenis yang tidak diketahui")
            || output.contains("mengandungi medan rekursif by-value"),
        "expected recursive/unknown struct field diagnostic, got:\n{output}"
    );
}

#[test]
fn valid_struct_layout_still_passes() {
    check_ok(
        r#"
struct Point begin
    x: I64;
    y: I64;
end

function get_x(p: Point) -> I64 begin
    return p.x;
end

PAPAR 1;
"#,
    );
}

#[test]
fn unknown_enum_variant_fails_in_hir_lowering() {
    let output = check_fail(
        r#"
function main() -> I64 begin
    return MissingEnum::MissingVariant;
end
"#,
    );

    assert!(
        output.contains("Enum variant `MissingEnum::MissingVariant` was not found")
            || output.contains("Varian enum `MissingEnum::MissingVariant` tidak ditemui")
            || output.contains("HIR lowering failed"),
        "expected unknown enum variant diagnostic, got:\n{output}"
    );
}

#[test]
fn unit_call_used_as_i64_return_fails() {
    let output = check_fail(
        r#"
function noop() -> Unit begin
end

function main() -> I64 begin
    return noop();
end
"#,
    );

    assert!(
        output.contains("returns Unit and cannot be used as a value")
            || output.contains("result type could not be resolved")
            || output.contains("result type is unknown")
            || output.contains("Return type mismatch")
            || output.contains("tidak boleh digunakan sebagai nilai")
            || output.contains("Jenis hasil panggilan"),
        "expected Unit/non-value call result diagnostic, got:\n{output}"
    );
}

#[test]
fn unit_call_statement_still_passes() {
    check_ok(
        r#"
function noop() -> Unit begin
end

function main() -> I64 begin
    noop();
    return 1;
end
"#,
    );
}

#[test]
fn tail_i64_expression_requires_explicit_return() {
    let output = check_fail(
        r#"
function main() -> I64 begin
    42;
end
"#,
    );

    assert!(
        output.contains("must have a guaranteed return path")
            || output.contains("mesti mempunyai laluan return yang dijamin"),
        "expected missing return diagnostic for tail expression, got:\n{output}"
    );
}

#[test]
fn tail_i64_call_requires_explicit_return() {
    let output = check_fail(
        r#"
function id(x: I64) -> I64 begin
    return x;
end

function main() -> I64 begin
    id(42);
end
"#,
    );

    assert!(
        output.contains("must have a guaranteed return path")
            || output.contains("mesti mempunyai laluan return yang dijamin"),
        "expected missing return diagnostic for tail call, got:\n{output}"
    );
}

#[test]
fn tail_struct_constructor_requires_explicit_return() {
    let output = check_fail(
        r#"
struct Point begin
    x: I64;
    y: I64;
end

function mk() -> Point begin
    Point(1, 2);
end

function main() -> I64 begin
    return 1;
end
"#,
    );

    assert!(
        output.contains("must have a guaranteed return path")
            || output.contains("mesti mempunyai laluan return yang dijamin"),
        "expected missing return diagnostic for tail constructor, got:\n{output}"
    );
}

#[test]
fn explicit_i64_return_still_passes() {
    check_ok(
        r#"
function main() -> I64 begin
    return 42;
end
"#,
    );
}

#[test]
fn explicit_struct_constructor_return_still_passes() {
    check_ok(
        r#"
struct Point begin
    x: I64;
    y: I64;
end

function mk() -> Point begin
    return Point(1, 2);
end

function main() -> I64 begin
    return 1;
end
"#,
    );
}

#[test]
fn unit_function_returning_value_fails() {
    let output = check_fail(
        r#"
function main() -> Unit begin
    return 1;
end
"#,
    );

    assert!(
        output.contains("Unit function cannot return a value")
            || output.contains("Fungsi Unit tidak boleh memulangkan nilai")
            || output.contains("Return type is unknown")
            || output.contains("Jenis pulangan tidak diketahui")
            || output.contains("Return type mismatch"),
        "expected Unit/unknown return value diagnostic, got:\n{output}"
    );
}

#[test]
fn unit_function_omitted_return_still_passes() {
    check_ok(
        r#"
function main() -> Unit begin
    let x: I64 = 1;
end
"#,
    );
}

#[test]
fn wrong_enum_qualifier_for_existing_variant_fails() {
    let output = check_fail(
        r#"
enum Status begin
    Ready;
    Done;
end

function main() -> I64 begin
    let s: Status = MissingEnum::Ready;
    return 1;
end
"#,
    );

    assert!(
        output.contains("Enum variant `MissingEnum::Ready` was not found")
            || output.contains("Varian enum `MissingEnum::Ready` tidak ditemui")
            || output.contains("HIR lowering failed"),
        "expected wrong enum qualifier diagnostic, got:\n{output}"
    );
}
