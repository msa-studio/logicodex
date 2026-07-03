use std::fs;
use std::process::Command;

#[test]
fn modulo_operator_smoke_and_precedence() {
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workdir =
        std::env::temp_dir().join(format!("logicodex_modulo_operator_{}", std::process::id()));

    let _ = fs::remove_dir_all(&workdir);
    fs::create_dir_all(&workdir).expect("create temp dir");

    let source = workdir.join("main.ldx");
    fs::write(
        &source,
        r#"
PAPAR 7 % 3;
PAPAR 8 % 4;
PAPAR 10 % 6;
PAPAR 2 + 7 % 3;
PAPAR 2 * 7 % 5;
"#,
    )
    .expect("write source");

    let compiler = env!("CARGO_BIN_EXE_logicodex");

    let compile = Command::new(compiler)
        .arg("compile")
        .arg(&source)
        .env("LOGICODEX_STD", repo_root.join("lib"))
        .output()
        .expect("run compiler");

    assert!(
        compile.status.success(),
        "compile failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        compile.status,
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let executable = workdir.join("main");
    let run = Command::new(&executable)
        .output()
        .expect("run compiled modulo program");

    let stdout = String::from_utf8_lossy(&run.stdout);
    let tokens: Vec<&str> = stdout.split_whitespace().collect();

    assert_eq!(tokens, vec!["1", "0", "4", "3", "4"]);
}
