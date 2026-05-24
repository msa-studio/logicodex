use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn logicodex_bin() -> &'static str {
    env!("CARGO_BIN_EXE_logicodex")
}

fn write_temp_source(name: &str, content: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "logicodex_{name}_{}_{}.ldx",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    fs::write(&path, content).expect("failed to write temporary Logicodex source");
    path
}

fn token_kinds(source: &str) -> Vec<String> {
    let path = write_temp_source("tokens", source);
    let output = Command::new(logicodex_bin())
        .args(["tokens", path.to_str().expect("valid UTF-8 temp path")])
        .output()
        .expect("failed to run logicodex tokens command");

    assert!(
        output.status.success(),
        "logicodex tokens failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split('\t').next())
        .filter(|kind| !kind.is_empty() && *kind != "Newline" && *kind != "Eof")
        .map(str::to_owned)
        .collect()
}

#[test]
fn malay_bina_and_expert_let_map_to_same_token_kind() {
    let malay = token_kinds("MULA\nBINA x = 1\nTAMAT\n");
    let expert = token_kinds("{\nlet x = 1\n}\n");

    assert!(malay.contains(&"Let".to_string()));
    assert!(expert.contains(&"Let".to_string()));
}

#[test]
fn malay_papar_and_expert_print_map_to_same_token_kind() {
    let malay = token_kinds("MULA\nBINA x = 1\nPAPAR x\nTAMAT\n");
    let expert = token_kinds("{\nlet x = 1\nprint x\n}\n");

    assert!(malay.contains(&"Print".to_string()));
    assert!(expert.contains(&"Print".to_string()));
}

#[test]
fn malay_start_end_and_expert_braces_map_to_same_token_kinds() {
    let malay = token_kinds("MULA\nTAMAT\n");
    let expert = token_kinds("{\n}\n");

    assert_eq!(malay, vec!["Start".to_string(), "End".to_string()]);
    assert_eq!(expert, vec!["Start".to_string(), "End".to_string()]);
}

#[test]
fn shipped_beginner_and_expert_examples_pass_semantic_check() {
    for example in ["examples/hello.ldx", "examples/matematik.ldx"] {
        let output = Command::new(logicodex_bin())
            .args(["check", example])
            .output()
            .expect("failed to run logicodex check command");

        assert!(
            output.status.success(),
            "logicodex check failed for {example}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
