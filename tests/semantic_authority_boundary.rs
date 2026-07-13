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
