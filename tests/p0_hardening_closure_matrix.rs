use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn check_fail(name: &str, source: &str) -> String {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();

    let path = std::env::temp_dir().join(format!(
        "logicodex_p0_closure_{name}_{}_{}.ldx",
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

    assert!(
        !output.status.success(),
        "{name}: expected check failure, got success\n{rendered}"
    );

    rendered
}

#[test]
fn p0_hardening_closure_matrix_has_codes_and_nonzero_spans() {
    let cases: &[(&str, &str, &str)] = &[
        (
            "unknown_name",
            "UnknownName",
            r###"function main() -> I64 begin
 return missing_name;
end
"###,
        ),
        (
            "unknown_function",
            "UnknownFunction",
            r###"function main() -> I64 begin
 return missing_fn();
end
"###,
        ),
        (
            "unknown_type",
            "UnknownType",
            r###"function main() -> MissingType begin
 return 1;
end
"###,
        ),
        (
            "wrong_enum_qualifier",
            "EnumTypeMismatch",
            r###"enum Status begin
 Ready;
end

enum Other begin
 Ready;
end

function main() -> I64 begin
 let s: Status = Other::Ready;
 return 1;
end
"###,
        ),
        (
            "missing_enum_variant",
            "UnknownEnumVariant",
            r###"enum Status begin
 Ready;
end

function main() -> I64 begin
 let s: Status = Status::Missing;
 return 1;
end
"###,
        ),
        (
            "return_bool",
            "TypeMismatch",
            r###"function main() -> I64 begin
 return true;
end
"###,
        ),
        (
            "let_type_mismatch",
            "TypeMismatch",
            r###"function main() -> I64 begin
 let x: I64 = true;
 return 1;
end
"###,
        ),
        (
            "if_non_bool",
            "TypeMismatch",
            r###"function main() -> I64 begin
 if 1 begin
  return 1;
 end
 return 0;
end
"###,
        ),
        (
            "array_index_bool",
            "TypeMismatch",
            r###"function main() -> I64 begin
 let xs: [I64; 2] = [1, 2];
 return xs[true];
end
"###,
        ),
        (
            "assignment_type_mismatch",
            "TypeMismatch",
            r###"function main() -> I64 begin
 let x: I64 = 1;
 x = true;
 return x;
end
"###,
        ),
        (
            "call_arg_count_mismatch",
            "TypeMismatch",
            r###"function add(a: I64, b: I64) -> I64 begin
 return a + b;
end

function main() -> I64 begin
 return add(1);
end
"###,
        ),
        (
            "call_arg_type_mismatch",
            "TypeMismatch",
            r###"function add(a: I64, b: I64) -> I64 begin
 return a + b;
end

function main() -> I64 begin
 return add(1, true);
end
"###,
        ),
        (
            "missing_return",
            "TypeMismatch",
            r###"function main() -> I64 begin
 let x = 1;
end
"###,
        ),
        (
            "field_on_integer",
            "TypeMismatch",
            r###"function main() -> I64 begin
 let x = 1;
 return x.y;
end
"###,
        ),
        (
            "division_by_zero",
            "DivisionByZero",
            r###"function main() -> I64 begin
 return 1 / 0;
end
"###,
        ),
    ];

    for (name, code, source) in cases {
        let output = check_fail(name, source);
        let expected_code = format!("code: {code}");

        assert!(
            output.contains(&expected_code),
            "{name}: expected {expected_code}\n{output}"
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
            !output.contains("ParserUnsupportedFeature"),
            "{name}: regressed to parser unsupported fallback\n{output}"
        );
        assert!(
            !output.contains("LLVM module verification failed"),
            "{name}: leaked to LLVM verifier\n{output}"
        );
        assert!(
            !output.contains("Incorrect number of arguments"),
            "{name}: leaked raw LLVM/call verifier error\n{output}"
        );
    }
}
