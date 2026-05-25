// =========================================================================
// Logicodex v1.40.0-alpha — WASM Codegen Backend Tests
//
// Tests: CompilationTarget::Wasm parsing, triple, LLVM WASM backend
// =========================================================================

use logicodex::os::target::CompilationTarget;

// ─── CompilationTarget::Wasm parsing ───

#[test]
fn wasm_target_parse_wasm() {
    let t = CompilationTarget::parse("wasm").unwrap();
    assert!(t.is_wasm());
    assert_eq!(t.entry_symbol(), "_start");
    assert_eq!(t.llvm_triple(), "wasm32-unknown-unknown");
}

#[test]
fn wasm_target_parse_wasm32() {
    let t = CompilationTarget::parse("wasm32").unwrap();
    assert!(t.is_wasm());
}

#[test]
fn wasm_target_not_native() {
    let t = CompilationTarget::parse("wasm").unwrap();
    assert!(!t.is_freestanding());
}

#[test]
fn native_target_still_works() {
    let t = CompilationTarget::parse("native").unwrap();
    assert!(!t.is_wasm());
    assert!(!t.is_freestanding());
}

#[test]
fn freestanding_target_still_works() {
    let t = CompilationTarget::parse("freestanding").unwrap();
    assert!(!t.is_wasm());
    assert!(t.is_freestanding());
}

// ─── CompilationTarget methods ───

#[test]
fn wasm_target_methods() {
    let t = CompilationTarget::Wasm;
    assert!(t.is_wasm());
    assert_eq!(t.entry_symbol(), "_start");
    assert!(!t.is_freestanding());
}

#[test]
fn all_targets_roundtrip() {
    for name in ["native", "host", "freestanding", "wasm", "wasm32"] {
        let t = CompilationTarget::parse(name).unwrap();
        // Should not panic
        let _ = t.entry_symbol();
        let _ = t.llvm_triple();
    }
}

#[test]
fn invalid_target_rejected() {
    let result = CompilationTarget::parse("invalid");
    assert!(result.is_err());
}
