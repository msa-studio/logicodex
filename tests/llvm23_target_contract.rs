// =========================================================================
// LLVM 23 target contract regression tests.
//
// Freezes the owner-approved LLVM 23 x86_64/wasm32 data layout and WASM
// target_features contract, keeps raw versus normalized parity evidence
// distinct, and proves `src/os/target.rs` declares/selects the contract
// without computing layout (`LayoutEngine` remains the layout authority).
// =========================================================================

use inkwell::context::Context;
use inkwell::targets::{FileType, TargetMachine};
use logicodex::os::target::{
    build_target_machine, build_target_machine_with_arch, llvm23_contract, OutputKind, TargetArch,
};

// LLVM 15 control contract, retained as the raw-diff baseline.
const LLVM15_X86_64_DATA_LAYOUT: &str =
    "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128";
const LLVM15_WASM32_DATA_LAYOUT: &str =
    "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20";
const LLVM15_WASM32_TARGET_FEATURES: &str = "+bulk-memory,+mutable-globals,+sign-ext,";

// Enabled features LLVM 23 records in the WASM object `target_features` section.
const LLVM23_WASM32_OBJECT_FEATURES: [&str; 8] = [
    "bulk-memory",
    "bulk-memory-opt",
    "call-indirect-overlong",
    "multivalue",
    "mutable-globals",
    "nontrapping-fptoint",
    "reference-types",
    "sign-ext",
];

fn data_layout(machine: &TargetMachine) -> String {
    machine
        .get_target_data()
        .get_data_layout()
        .as_str()
        .to_str()
        .expect("data layout is UTF-8")
        .to_string()
}

/// Emit a minimal WASM module through the production target machine and
/// return (object bytes, post-emission IR).
fn emit_wasm_probe() -> (Vec<u8>, String) {
    let context = Context::create();
    let module = context.create_module("llvm23_target_contract");
    let i64_type = context.i64_type();
    let print = module.add_function(
        "logicodex_print_i64",
        context.void_type().fn_type(&[i64_type.into()], false),
        None,
    );
    let start = module.add_function("_start", context.void_type().fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(start, "entry"));
    builder
        .build_call(print, &[i64_type.const_int(42, false).into()], "")
        .expect("build call");
    builder.build_return(None).expect("build return");
    module.verify().expect("probe module verifies");

    let machine = build_target_machine(OutputKind::WasmModule).expect("wasm target machine");
    let object = machine
        .write_to_memory_buffer(&module, FileType::Object)
        .expect("emit wasm object")
        .as_slice()
        .to_vec();
    (object, module.print_to_string().to_string())
}

fn read_leb_u32(bytes: &[u8], cursor: &mut usize) -> u32 {
    let mut result = 0u32;
    let mut shift = 0;
    loop {
        let byte = bytes[*cursor];
        *cursor += 1;
        result |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return result;
        }
        shift += 7;
    }
}

fn read_name(bytes: &[u8], cursor: &mut usize) -> String {
    let len = read_leb_u32(bytes, cursor) as usize;
    let name = std::str::from_utf8(&bytes[*cursor..*cursor + len])
        .expect("wasm name is UTF-8")
        .to_string();
    *cursor += len;
    name
}

/// Decode the `target_features` custom section as `(prefix, name)` entries.
fn wasm_object_target_features(object: &[u8]) -> Vec<(char, String)> {
    assert_eq!(&object[..4], b"\0asm", "not a wasm object");
    let mut cursor = 8;
    while cursor < object.len() {
        let id = object[cursor];
        cursor += 1;
        let size = read_leb_u32(object, &mut cursor) as usize;
        let end = cursor + size;
        if id == 0 {
            let mut inner = cursor;
            if read_name(object, &mut inner) == "target_features" {
                let count = read_leb_u32(object, &mut inner);
                return (0..count)
                    .map(|_| {
                        let prefix = object[inner] as char;
                        inner += 1;
                        (prefix, read_name(object, &mut inner))
                    })
                    .collect();
            }
        }
        cursor = end;
    }
    panic!("wasm object has no target_features section");
}

#[test]
fn llvm23_x86_64_data_layout_contract_is_frozen() {
    let freestanding =
        build_target_machine_with_arch(OutputKind::FreestandingObject, TargetArch::X86_64)
            .expect("x86_64 freestanding target machine");
    assert_eq!(
        data_layout(&freestanding),
        llvm23_contract::X86_64_DATA_LAYOUT
    );

    if cfg!(target_arch = "x86_64") {
        let native = build_target_machine(OutputKind::Object).expect("native target machine");
        assert_eq!(data_layout(&native), llvm23_contract::X86_64_DATA_LAYOUT);
    }

    assert_eq!(
        llvm23_contract::X86_64_DATA_LAYOUT,
        LLVM15_X86_64_DATA_LAYOUT.replace("-i64:64-", "-i64:64-i128:128-"),
        "x86_64 drift must be exactly the approved i128:128 entry"
    );
}

#[test]
fn llvm23_wasm_data_layout_contract_is_frozen() {
    let machine = build_target_machine(OutputKind::WasmModule).expect("wasm target machine");
    assert_eq!(data_layout(&machine), llvm23_contract::WASM32_DATA_LAYOUT);

    let (_, ir) = emit_wasm_probe();
    assert!(
        ir.contains(&format!(
            "target datalayout = \"{}\"",
            llvm23_contract::WASM32_DATA_LAYOUT
        )),
        "emitted WASM IR must carry the frozen data layout:\n{ir}"
    );

    assert_eq!(
        llvm23_contract::WASM32_DATA_LAYOUT,
        LLVM15_WASM32_DATA_LAYOUT.replace("-i64:64-", "-i64:64-i128:128-"),
        "wasm32 drift must be exactly the approved i128:128 entry"
    );
}

#[test]
fn llvm23_wasm_target_features_contract_is_frozen() {
    assert_eq!(
        llvm23_contract::WASM32_REQUESTED_FEATURES,
        "+bulk-memory,+mutable-globals,+sign-ext",
        "the WASM feature request is unchanged; only LLVM-derived metadata drifts"
    );

    let (object, ir) = emit_wasm_probe();
    assert!(
        ir.contains(&format!(
            "\"target-features\"=\"{}\"",
            llvm23_contract::WASM32_TARGET_FEATURES
        )),
        "emitted WASM IR must carry the frozen target-features attribute:\n{ir}"
    );

    let features = wasm_object_target_features(&object);
    let expected: Vec<(char, String)> = LLVM23_WASM32_OBJECT_FEATURES
        .iter()
        .map(|name| ('+', name.to_string()))
        .collect();
    assert_eq!(features, expected, "wasm object target_features section");

    let enabled: Vec<&str> = llvm23_contract::WASM32_TARGET_FEATURES
        .split(',')
        .filter_map(|feature| feature.strip_prefix('+'))
        .collect();
    assert_eq!(enabled, LLVM23_WASM32_OBJECT_FEATURES);
}

#[derive(Debug, PartialEq, Eq)]
enum Drift {
    Identical,
    ApprovedTargetContractDrift,
    Protected,
}

fn approved_pairs() -> Vec<(String, String)> {
    vec![
        (
            format!("target datalayout = \"{LLVM15_X86_64_DATA_LAYOUT}\""),
            format!(
                "target datalayout = \"{}\"",
                llvm23_contract::X86_64_DATA_LAYOUT
            ),
        ),
        (
            format!("target datalayout = \"{LLVM15_WASM32_DATA_LAYOUT}\""),
            format!(
                "target datalayout = \"{}\"",
                llvm23_contract::WASM32_DATA_LAYOUT
            ),
        ),
        (
            format!("\"target-features\"=\"{LLVM15_WASM32_TARGET_FEATURES}\""),
            format!(
                "\"target-features\"=\"{}\"",
                llvm23_contract::WASM32_TARGET_FEATURES
            ),
        ),
    ]
}

fn classify(control: &str, candidate: &str) -> Drift {
    if control == candidate {
        Drift::Identical
    } else if approved_pairs()
        .iter()
        .any(|(old, new)| old == control && new == candidate)
    {
        Drift::ApprovedTargetContractDrift
    } else {
        Drift::Protected
    }
}

/// Normalized view: approved candidate values map back to the control value.
/// Returns a new artifact; the raw input is never rewritten.
fn normalize(candidate: &str) -> String {
    approved_pairs()
        .into_iter()
        .find(|(_, new)| new == candidate)
        .map(|(old, _)| old)
        .unwrap_or_else(|| candidate.to_string())
}

#[test]
fn raw_and_normalized_parity_artifacts_remain_distinct() {
    for (control, candidate) in approved_pairs() {
        assert_ne!(control, candidate, "raw diff must remain visible");
        assert_eq!(
            classify(&control, &candidate),
            Drift::ApprovedTargetContractDrift
        );

        let normalized = normalize(&candidate);
        assert_eq!(
            normalized, control,
            "normalized view absorbs approved drift"
        );
        assert_ne!(
            normalized, candidate,
            "normalized artifact must not replace the raw artifact"
        );
        assert_ne!(
            classify(&control, &candidate),
            Drift::Identical,
            "a normalized match never turns a raw diff into a raw PASS"
        );
    }

    let unapproved = [
        (
            format!("target datalayout = \"{LLVM15_X86_64_DATA_LAYOUT}\""),
            format!(
                "target datalayout = \"{}\"",
                LLVM15_X86_64_DATA_LAYOUT.replace("-i64:64-", "-i64:64-i128:64-")
            ),
        ),
        (
            format!("\"target-features\"=\"{LLVM15_WASM32_TARGET_FEATURES}\""),
            format!(
                "\"target-features\"=\"{}\"",
                llvm23_contract::WASM32_TARGET_FEATURES.replace("-simd128", "+simd128")
            ),
        ),
        (
            "define void @_start()".to_string(),
            "define fastcc void @_start()".to_string(),
        ),
    ];
    for (control, candidate) in unapproved {
        assert_eq!(classify(&control, &candidate), Drift::Protected);
        assert_eq!(
            normalize(&candidate),
            candidate,
            "protected drift is never normalized"
        );
    }
}

fn code_lines(source: &str) -> impl Iterator<Item = &str> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("//"))
}

#[test]
fn target_contract_does_not_compute_layout() {
    let target_source = include_str!("../src/os/target.rs");

    let forbidden = [
        "LayoutEngine",
        "TargetLayout",
        "compute_struct_layout",
        "crate::layout",
        "get_target_data",
        "TargetData",
        "set_data_layout",
        "get_abi_size",
        "get_abi_alignment",
        "get_store_size",
        "get_bit_size",
        "offset_of_element",
        "size_of",
        "align_of",
        "offset_of",
        "pointer_size",
        "alignment",
    ];
    for line in code_lines(target_source) {
        for symbol in forbidden {
            assert!(
                !line.contains(symbol),
                "src/os/target.rs must not compute layout; found `{symbol}` in `{line}`"
            );
        }
    }

    for constant in [
        "pub const X86_64_DATA_LAYOUT: &str",
        "pub const WASM32_DATA_LAYOUT: &str",
        "pub const WASM32_REQUESTED_FEATURES: &str",
        "pub const WASM32_TARGET_FEATURES: &str",
    ] {
        assert!(
            target_source.contains(constant),
            "target contract declaration `{constant}` is missing"
        );
    }
    assert!(
        target_source.contains("llvm23_contract::WASM32_REQUESTED_FEATURES,"),
        "WASM target machine must select the declared feature request"
    );
}

#[test]
fn target_contract_routes_layout_to_layout_engine() {
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

    let layout_source = include_str!("../src/layout.rs");
    assert!(layout_source.contains("pub struct LayoutEngine"));
    assert!(layout_source.contains("pub struct TargetLayout"));
    assert!(layout_source.contains("pub fn compute_struct_layout"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let layout_path = source_root.join("layout.rs");
    let mut source_files = Vec::new();
    collect_rust_files(&source_root, &mut source_files);

    let llvm_layout_queries = [
        "set_data_layout",
        "get_target_data",
        "get_abi_size",
        "get_abi_alignment",
        "get_store_size",
        "offset_of_element",
        "llvm23_contract::X86_64_DATA_LAYOUT",
        "llvm23_contract::WASM32_DATA_LAYOUT",
    ];

    for path in source_files {
        if path == layout_path {
            continue;
        }
        let source = fs::read_to_string(&path).expect("read Rust source file");
        for line in code_lines(&source) {
            assert!(
                !line.contains("fn compute_struct_layout"),
                "second layout authority defined in {}",
                path.display()
            );
            for symbol in llvm_layout_queries {
                assert!(
                    !line.contains(symbol),
                    "`{symbol}` bypasses LayoutEngine in {}",
                    path.display()
                );
            }
        }
    }
}
