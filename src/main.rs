// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.45.0-alpha (Deterministic Systems Platform)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
mod ast;
mod codegen;
mod codegen_contract;
mod ffi;
mod hir;
mod layout;
mod lexer;
mod os;
mod parser;
mod semantic;
mod semantic_gate;
mod span;
mod types;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser as ClapParser, Subcommand};
use codegen::{CodegenOptions, LlvmCompiler, MemoryIntegrityPlan, PhysicalMemoryAccessPlan};
use ffi::raylib;
use lexer::{Lexer, Lexicon};
use os::target::CompilationTarget;
use parser::{CompilerPipeline, Parser};
use semantic::Analyzer;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const LOGICODEX_LOGO: &str = r#"=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
             [ LOGICODEX COMPILER v1.45.0-alpha ]
             [ DETERMINISTIC SYSTEMS PLATFORM   ]
=========================================================
Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)"#;

const LOGICODEX_LONG_VERSION: &str = r#"=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
             [ LOGICODEX COMPILER v1.45.0-alpha ]
             [ DETERMINISTIC SYSTEMS PLATFORM   ]
=========================================================
Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
logicodex 1.45.0-alpha
Security Roadmap: deterministic systems platform with capability fabric"#;

#[derive(Debug, ClapParser)]
#[command(
    name = "logicodex",
    version = "1.45.0-alpha",
    long_version = LOGICODEX_LONG_VERSION,
    about = "Native compiler for the Logicodex programming language by Mohamad Supardi Abdul",
    before_help = LOGICODEX_LOGO
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Print the official Logicodex terminal logo")]
    Logo,
    Compile {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<PathBuf>,
        #[arg(long, default_value = "dict/core_map.json")]
        dict: PathBuf,
        #[arg(long)]
        emit_ir: bool,
        #[arg(long, help = "Stop after generating the native object file")]
        object_only: bool,
        #[arg(
            short = 's',
            long,
            help = "Emit the planned runtime memory-integrity attestation notes for the current security roadmap"
        )]
        secure: bool,
        #[arg(long, default_value = "native", value_parser = ["native", "host", "freestanding", "wasm"], help = "Select target: native, freestanding, or wasm (WebAssembly)")]
        target: String,
        #[arg(long, default_value = "v1.21", help = "Select compiler pipeline: v1.21 (stable) or v1.30 (experimental)")]
        pipeline: String,
    },
    Check {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value = "dict/core_map.json")]
        dict: PathBuf,
        #[arg(long, default_value = "v1.21", help = "Select compiler pipeline: v1.21 (stable) or v1.30 (experimental)")]
        pipeline: String,
    },
    #[command(
        name = "v130-check",
        hide = true,
        about = "Run dormant v1.30.0-alpha subsystem validation after the stable v1.21 semantic check"
    )]
    V130Check {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value = "dict/core_map.json")]
        dict: PathBuf,
    },
    Tokens {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value = "dict/core_map.json")]
        dict: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Logo) => {
            println!("{LOGICODEX_LOGO}");
            Ok(())
        }
        Some(Commands::Compile {
            file,
            output,
            dict,
            emit_ir,
            object_only,
            secure,
            target,
            pipeline,
        }) => {
            let pipeline = pipeline.parse::<CompilerPipeline>()?;
            compile(&file, output, &dict, emit_ir, object_only, secure, &target, pipeline)
        }
        Some(Commands::Check { file, dict, pipeline }) => {
            let pipeline = pipeline.parse::<CompilerPipeline>()?;
            parse_and_analyze(&file, &dict, pipeline)?;
            println!("{}: semantic validation succeeded", file.display());
            Ok(())
        }
        Some(Commands::V130Check { file, dict }) => v130_check(&file, &dict),
        Some(Commands::Tokens { file, dict }) => print_tokens(&file, &dict),
        None => {
            println!("{LOGICODEX_LOGO}\n");
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}

fn compile(
    file: &Path,
    output: Option<PathBuf>,
    dict: &Path,
    emit_ir: bool,
    object_only: bool,
    secure: bool,
    target_name: &str,
    pipeline: CompilerPipeline,
) -> Result<()> {
    ensure_ldx_source(file)?;
    let target = CompilationTarget::parse(target_name)?;
    let output_path = output.unwrap_or_else(|| default_output(file, object_only));
    let object_path = if object_only {
        output_path.clone()
    } else {
        output_path.with_extension("o")
    };

    // Sprint 3: Version-gated compilation — V130 uses HIR + CallableRegistry path
    let artifact = match pipeline {
        CompilerPipeline::V130 => {
            compile_v130_pipeline(file, dict, &object_path, emit_ir, target)?
        }
        CompilerPipeline::V121 => {
            let program = parse_and_analyze_for_target(file, dict, target_name, secure, pipeline)?;
            LlvmCompiler::compile_to_object(
                &program,
                &object_path,
                &CodegenOptions {
                    module_name: module_name(file),
                    emit_ir,
                    secure,
                    target,
                },
            )?
        }
    };
    if let Some(ir_path) = artifact.ir_path.as_ref() {
        println!("LLVM IR written to {}", ir_path.display());
    }

    if object_only {
        println!("Object file written to {}", artifact.object_path.display());
        if secure {
            write_security_attestation_plan(&output_path)?;
        }
        if target.is_freestanding() {
            write_freestanding_plan(&output_path)?;
        }
        if target.is_wasm() {
            println!("WASM module written to {} (use wasm-ld to link with WASI libc)",
                artifact.object_path.display());
        }
        return Ok(());
    }

    if target.is_freestanding() {
        println!(
            "Freestanding object written to {}",
            artifact.object_path.display()
        );
        write_freestanding_plan(&output_path)?;
        if secure {
            write_security_attestation_plan(&output_path)?;
        }
        return Ok(());
    }

    // v1.40: WASM target — emit .wasm via LLVM wasm32-unknown-unknown backend
    if target.is_wasm() {
        println!(
            "WASM module written to {}",
            artifact.object_path.display()
        );
        println!("  Triple: {}", target.llvm_triple()); // wasm32-unknown-unknown
        println!("  Use: wasm-ld --no-entry -o output.wasm {} --export-all",
            artifact.object_path.display());
        return Ok(());
    }

    let runtime_asm = output_path.with_extension("runtime.s");
    fs::write(&runtime_asm, os::runtime_assembly())
        .with_context(|| format!("failed to write runtime assembly {}", runtime_asm.display()))?;
    link_executable(&artifact.object_path, &runtime_asm, &output_path)?;
    println!("Native executable written to {}", output_path.display());
    if secure {
        write_security_attestation_plan(&output_path)?;
    }
    Ok(())
}

/// Sprint 3: v1.30 HIR compilation pipeline with CallableRegistry + Raylib FFI.
fn compile_v130_pipeline(
    file: &Path,
    dict: &Path,
    object_path: &Path,
    emit_ir: bool,
    target: CompilationTarget,
) -> Result<codegen::CodegenArtifact> {
    // Step 1: Parse source to AST
    let source = fs::read_to_string(file)
        .with_context(|| format!("failed to read Logicodex source file {}", file.display()))?;
    let lexicon = Lexicon::from_path(dict)
        .with_context(|| format!("failed to load dictionary {}", dict.display()))?;
    let tokens = Lexer::new(&source, &lexicon).tokenize()?;
    let mut parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse()?;
    Analyzer::analyze_for_target(&program, "native", false)?;

    // Step 2: Set up TypeRegistry with Raylib struct types
    let mut types = types::TypeRegistry::new();
    let (raylib_type_ids, raylib_struct_ids) = ffi::raylib::register_raylib_types(&mut types);

    // Step 3: Lower AST → HIR
    let mut symbols = hir::SymbolTable::default();
    let module_ast = {
        let mut lowering = hir::LoweringContext {
            symbols: &mut symbols,
            types: &mut types,
            diagnostics: Vec::new(),
        };
        lowering.lower_v121_program(program)?
    };

    // Step 4: Set up CallableRegistry with Raylib functions
    let mut callables = ffi::CallableRegistry::default();
    ffi::raylib::register_raylib_functions(&mut types, &mut callables, &raylib_struct_ids);

    // v1.42 P7: WASM target — Raylib is native-desktop only, block all Raylib functions
    if target.is_wasm() {
        let raylib_names: Vec<String> = callables.signatures.iter()
            .filter(|sig| ffi::raylib::is_struct_constructor(&sig.name) || is_raylib_function(&sig.name))
            .map(|sig| sig.name.clone())
            .collect();
        if !raylib_names.is_empty() {
            return Err(anyhow::anyhow!(
                "WASM target does not support Raylib graphics functions ({} found: {}). \
                 Use WebGL or Canvas API via the WASM host instead.",
                raylib_names.len(),
                raylib_names.join(", ")
            ));
        }
        // Remove all Raylib functions from the callable registry
        callables.signatures.retain(|sig| !is_raylib_function(&sig.name));
    }

    // Step 5: Semantic check
    let mut semantic = semantic_gate::SemanticContext {
        types,
        symbols,
        callables: callables.clone(),
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: ffi::SafetyContext::Safe,
    };
    semantic
        .check_module(&module_ast)
        .map_err(|diagnostics| anyhow::anyhow!(format_v130_diagnostics(&diagnostics)))?;

    // Step 6: HIR → LLVM object via compile_v130 with registries
    codegen::compile_v130(
        &module_ast,
        object_path,
        &CodegenOptions {
            module_name: module_name(file),
            emit_ir,
            secure: false,
            target,
        },
        callables,
        semantic.types,
    )
}

/// v1.42 P7: Check if a function name is a Raylib function.
/// Used to filter out Raylib functions when targeting WASM.
fn is_raylib_function(name: &str) -> bool {
    const RAYLIB_FUNCTIONS: &[&str] = &[
        "InitWindow", "CloseWindow", "WindowShouldClose", "SetTargetFPS",
        "GetFPS", "GetFrameTime", "GetTime", "GetScreenWidth", "GetScreenHeight",
        "BeginDrawing", "EndDrawing", "ClearBackground", "DrawText",
        "DrawRectangle", "DrawCircle", "DrawLine", "DrawRectangleLines", "DrawPixel",
        "LoadTexture", "DrawTexture", "UnloadTexture",
        "IsKeyDown", "IsKeyPressed", "GetKeyPressed",
        "IsMouseButtonPressed", "GetMouseX", "GetMouseY", "GetMousePosition",
    ];
    RAYLIB_FUNCTIONS.contains(&name)
}

fn write_security_attestation_plan(output_path: &Path) -> Result<()> {
    let mut plan_path = output_path.to_path_buf();
    plan_path.set_extension("security.md");

    // v1.38 G1: Compute a simple module integrity hash (placeholder for SHA-256)
    // In production, this would compute a cryptographic hash of the .text section
    let module_hash = compute_module_hash(output_path);

    let content = format!(
        "# Logicodex Runtime Memory Integrity Verification Plan\n\nTarget artifact: `{}`\nModule hash (placeholder): `{:x}`\n\nSecurity roadmap: v1.21-alpha specification baseline and practical severity model.\n\nThe `--secure` compilation path computes a module integrity hash and records the security roadmap. v1.38: Basic hash computation via simple folding (placeholder for SHA-256).\n\nMemory integrity plan: `{:?}`\n\n## Verification Steps\n1. Recompile with `--secure` flag\n2. Compare module hash against known-good value\n3. Hash mismatch → fail-stop (hosted: process termination, freestanding: halt)\n",
        output_path.display(),
        module_hash,
        MemoryIntegrityPlan::hardened_default()
    );
    fs::write(&plan_path, content).with_context(|| {
        format!(
            "failed to write security attestation plan {}",
            plan_path.display()
        )
    })?;
    println!(
        "Security attestation plan written to {} (module hash: {:x})",
        plan_path.display(),
        module_hash
    );
    Ok(())
}

/// v1.38 G1: Compute a simple module integrity hash.
/// Placeholder — production would use SHA-256 over the .text section.
fn compute_module_hash(path: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    // In production: read the compiled binary and hash its contents
    hasher.finish()
}

fn write_freestanding_plan(output_path: &Path) -> Result<()> {
    use logicodex::os::target::TargetArch;

    let mut plan_path = output_path.to_path_buf();
    plan_path.set_extension("freestanding.md");
    let access_plan = PhysicalMemoryAccessPlan::freestanding_default();

    // v1.44 G8: Auto-detect host architecture for freestanding target
    let arch = detect_host_arch();
    let target_triple = arch.llvm_triple();
    let features = arch.llvm_features();
    let code_model = format!("{:?}", arch.code_model());

    let content = format!(
        "# Logicodex Freestanding Target Plan\n\n\
         Target artifact: `{}`\n\
         LLVM target triple: `{}`\n\
         Architecture: `{:?}`\n\
         LLVM features: `{}`\n\
         Code model: `{}`\n\n\
         The `--target freestanding` path emits a raw object for bootloader, \
         kernel, hypervisor, or firmware integration. The backend selects:\n\
         - Target triple: `{}`\n\
         - Relocation: static\n\
         - Code model: `{}`\n\
         - Entry symbol: `_start`\n\
         - Features: `{}`\n\
         - Startup: `_start` (stack init, BSS zero, data copy)\n\
         - Allocator: bump allocator\n\
         - Output: UART 0x3F8 + VGA 0xB8000\n\n\
         Physical memory access plan: `{:?}`\n\n\
         Raw pointer representation (`*int`) is reserved for memory-mapped I/O \
         (VGA `0xB8000`, UART `0x3F8`) under explicit backend safety gates.\n",
        output_path.display(),
        target_triple, arch, features, code_model,
        target_triple, code_model, features,
        access_plan
    );
    fs::write(&plan_path, content).with_context(|| {
        format!(
            "failed to write freestanding target plan {}",
            plan_path.display()
        )
    })?;
    println!(
        "Freestanding target plan written to {} (arch: {:?}, triple: {})",
        plan_path.display(), arch, target_triple
    );
    Ok(())
}

/// v1.44 G8: Auto-detect the host CPU architecture for freestanding target.
fn detect_host_arch() -> logicodex::os::target::TargetArch {
    use logicodex::os::target::TargetArch;
    if cfg!(target_arch = "x86_64") {
        TargetArch::X86_64
    } else if cfg!(target_arch = "aarch64") {
        TargetArch::Aarch64
    } else if cfg!(target_arch = "riscv64") {
        TargetArch::Riscv64
    } else {
        TargetArch::X86_64 // Default fallback
    }
}

fn parse_and_analyze(file: &Path, dict: &Path, pipeline: CompilerPipeline) -> Result<ast::Program> {
    parse_and_analyze_for_target(file, dict, "native", false, pipeline)
}

fn v130_check(file: &Path, dict: &Path) -> Result<()> {
    parse_and_analyze(file, dict, CompilerPipeline::V121)?;
    run_v130_subsystem_self_check()?;
    println!(
        "{}: v1.21 semantic validation and dormant v1.30.0-alpha subsystem check succeeded",
        file.display()
    );
    Ok(())
}

fn run_v130_subsystem_self_check() -> Result<()> {
    let mut types = types::TypeRegistry::new();
    let ids = types.primitive_ids();
    let pointer_to_i64 = types.intern(types::TypeKind::Pointer {
        pointee: ids.i64_,
        mutability: types::Mutability::Immutable,
    });

    {
        let layout_engine = layout::LayoutEngine {
            types: &types,
            target: layout::TargetLayout::default(),
        };
        layout_engine
            .compute_struct_layout(layout::LayoutRequest {
                name: "V130Probe".to_string(),
                fields: vec![
                    layout::LayoutFieldRequest {
                        name: "tag".to_string(),
                        ty: ids.u8_,
                        span: span::Span::unknown(),
                    },
                    layout::LayoutFieldRequest {
                        name: "payload".to_string(),
                        ty: pointer_to_i64,
                        span: span::Span::unknown(),
                    },
                ],
                attributes: Vec::new(),
                span: span::Span::unknown(),
            })
            .map_err(|diagnostic| anyhow::anyhow!(format_v130_diagnostic(&diagnostic)))?;
    }

    let mut symbols = hir::SymbolTable::default();
    let module_ast = hir::ModuleAst {
        items: vec![span::Spanned {
            node: hir::ItemAst::Function(hir::FunctionAst {
                name: "main".to_string(),
                params: Vec::new(),
                return_type: Some(hir::TypeAst::Unit),
                body: hir::BlockAst {
                    statements: vec![span::Spanned {
                        node: hir::StmtAst::Loop {
                            body: hir::BlockAst {
                                statements: vec![span::Spanned {
                                    node: hir::StmtAst::Break,
                                    span: span::Span::unknown(),
                                }],
                            },
                        },
                        span: span::Span::unknown(),
                    }],
                },
                is_unsafe: false,
            }),
            span: span::Span::unknown(),
        }],
    };
    let mut types = types::TypeRegistry::new();
    let mut lowering = hir::LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
    };
    let hir_module = lowering
        .lower_module(module_ast)
        .map_err(|diagnostics| anyhow::anyhow!(format_v130_diagnostics(&diagnostics)))?;

    let mut semantic = semantic_gate::SemanticContext {
        types,
        symbols: hir::SymbolTable::default(),
        callables: ffi::CallableRegistry::default(),
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: ffi::SafetyContext::Safe,
    };
    semantic
        .check_module(&hir_module)
        .map_err(|diagnostics| anyhow::anyhow!(format_v130_diagnostics(&diagnostics)))?;

    Ok(())
}

fn format_v130_diagnostics(diagnostics: &[span::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(format_v130_diagnostic)
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_v130_diagnostic(diagnostic: &span::Diagnostic) -> String {
    format!("{} / {}", diagnostic.message_ms, diagnostic.message_en)
}

fn parse_and_analyze_for_target(
    file: &Path,
    dict: &Path,
    target_name: &str,
    secure: bool,
    pipeline: CompilerPipeline,
) -> Result<ast::Program> {
    ensure_ldx_source(file)?;
    let source = fs::read_to_string(file)
        .with_context(|| format!("failed to read Logicodex source file {}", file.display()))?;
    let lexicon = Lexicon::from_path(dict)
        .with_context(|| format!("failed to load dictionary {}", dict.display()))?;
    let tokens = Lexer::new(&source, &lexicon).tokenize()?;
    let mut parser = Parser::new(tokens).with_pipeline(pipeline);
    let program = parser.parse()?;
    Analyzer::analyze_for_target(&program, target_name, secure)?;
    Ok(program)
}

fn print_tokens(file: &Path, dict: &Path) -> Result<()> {
    ensure_ldx_source(file)?;
    let source = fs::read_to_string(file)
        .with_context(|| format!("failed to read Logicodex source file {}", file.display()))?;
    let lexicon = Lexicon::from_path(dict)
        .with_context(|| format!("failed to load dictionary {}", dict.display()))?;
    for token in Lexer::new(&source, &lexicon).tokenize()? {
        println!(
            "{:?}\t{}\t{}:{}",
            token.kind, token.lexeme, token.line, token.column
        );
    }
    Ok(())
}

fn ensure_ldx_source(file: &Path) -> Result<()> {
    if file.extension().and_then(|e| e.to_str()) == Some("ldx") {
        Ok(())
    } else {
        anyhow::bail!(
            "Logicodex source files must use the official .ldx extension: {}",
            file.display()
        )
    }
}

fn default_output(file: &Path, object_only: bool) -> PathBuf {
    let mut path = file.with_extension(if object_only {
        "o"
    } else if cfg!(target_os = "windows") {
        "exe"
    } else {
        ""
    });
    if !object_only && !cfg!(target_os = "windows") {
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("a.out");
        path.set_file_name(stem);
    }
    path
}

fn module_name(file: &Path) -> String {
    file.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("logicodex_module")
        .replace('-', "_")
}

fn link_executable(object_path: &Path, runtime_asm: &Path, output_path: &Path) -> Result<()> {
    let linker = std::env::var("LOGICODEX_LINKER").unwrap_or_else(|_| "cc".to_string());
    let status = Command::new(&linker)
        .arg(object_path)
        .arg(runtime_asm)
        .arg("-o")
        .arg(output_path)
        .status()
        .with_context(|| format!("failed to invoke linker `{linker}`"))?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("linker `{linker}` failed with status {status}")
    }
}
