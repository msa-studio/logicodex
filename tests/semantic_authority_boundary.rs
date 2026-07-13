#[test]
fn canonical_semantic_authority_is_explicit() {
    let main_source = include_str!("../src/main.rs");
    let codegen_source = include_str!("../src/codegen.rs");
    let gate_source = include_str!("../src/semantic_gate.rs");
    let lifecycle_doc = include_str!("../docs/architecture/semantic-lifecycle-status.md");

    assert!(
        main_source.contains("semantic_gate::SemanticContext"),
        "main compiler paths must use the canonical HIR semantic gate"
    );
    assert!(
        codegen_source.contains("crate::semantic_gate::validate_module"),
        "codegen must retain its defensive semantic boundary"
    );
    assert!(
        gate_source.contains("ACTIVE / CANONICAL MEANING AUTHORITY"),
        "canonical semantic authority marker is missing"
    );
    assert!(
        lifecycle_doc.contains("semantic.rs::Analyzer")
            && lifecycle_doc.contains("LegacyReferenceOnly")
            && lifecycle_doc.contains("semantic.rs::SemanticError")
            && lifecycle_doc.contains("Active"),
        "semantic lifecycle classification record is incomplete"
    );
}

#[test]
fn retired_ast_analyzer_is_not_wired_into_main_or_codegen() {
    let main_source = include_str!("../src/main.rs");
    let codegen_source = include_str!("../src/codegen.rs");

    assert!(
        !main_source.contains("Analyzer::"),
        "retired AST Analyzer must not be wired into the main compiler"
    );
    assert!(
        !codegen_source.contains("Analyzer::"),
        "retired AST Analyzer must not be wired into codegen"
    );
}

#[test]
fn future_reserved_semantic_subsystem_stays_outside_production_wiring() {
    use std::fs;
    use std::path::{Path, PathBuf};

    fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(directory).expect("read source directory") {
            let path = entry.expect("read source entry").path();

            if path.is_dir() {
                collect_rust_files(&path, files);
            } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    let excluded = [
        source_root.join("semantic/type_checker.rs"),
        source_root.join("semantic/coercion.rs"),
        source_root.join("semantic/registry.rs"),
    ];

    let forbidden = [
        "TypeChecker::new",
        "semantic::type_checker::TypeChecker",
        "type_checker::TypeChecker",
        "CoercionEngine::new",
        "semantic::coercion::CoercionEngine",
        "TypeInspector::new",
        "semantic::registry::TypeInspector",
    ];

    let mut source_files = Vec::new();
    collect_rust_files(&source_root, &mut source_files);

    for path in source_files {
        if excluded.contains(&path) {
            continue;
        }

        let source = fs::read_to_string(&path).expect("read Rust source file");

        for symbol in forbidden {
            assert!(
                !source.contains(symbol),
                "FutureReserved semantic symbol `{symbol}` is wired in {}",
                path.display()
            );
        }
    }
}
