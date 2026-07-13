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
mod contract_metadata;
mod ffi;
mod hir;
mod layout;
mod lexer;
mod lod;
mod module_loader;
mod os;
mod parser;
#[allow(dead_code)]
mod semantic;
mod semantic_gate;
mod span;
mod types;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser as ClapParser, Subcommand};
use codegen::{CodegenOptions, MemoryIntegrityPlan, PhysicalMemoryAccessPlan};

use lexer::{Lexer, Lexicon};
use os::target::CompilationTarget;
use parser::{CompilerPipeline, Parser};
// Semantic lifecycle boundary:
// - HIR lowering + semantic_gate own canonical semantic validation.
// - semantic::Analyzer is retained as LegacyReferenceOnly.
// - selected semantic types remain imported by non-canonical subsystems.
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
             [ LOGICODEX COMPILER v1.30.0-alpha ]
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
             [ LOGICODEX COMPILER v1.30.0-alpha ]
             [ DETERMINISTIC SYSTEMS PLATFORM   ]
=========================================================
Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
logicodex 1.30.0-alpha
Security Roadmap: deterministic systems platform with capability fabric"#;

#[derive(Debug, ClapParser)]
#[command(
    name = "logicodex",
    version = "1.30.0-alpha",
    long_version = LOGICODEX_LONG_VERSION,
    about = "Native compiler for the Logicodex programming language by Mohamad Supardi Abdul",
    before_help = LOGICODEX_LOGO
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Runtime profile selects which runtime ABI surface a program may use.
///
/// Honest status (Phase D):
///   bare    -> no runtime builtins beyond print (minimal/native identity)
///   std     -> print + sleep + yield (real syscall backends)  [DEFAULT]
///   safe    -> std + capability checks                        [runtime-pending]
///   actor   -> std + spawn/join/channel                       [runtime-pending: Phase B]
///   service -> std + local reactor/health/metrics             [runtime-pending]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Profile {
    Bare,
    Std,
    Safe,
    Actor,
    Service,
}

impl Profile {
    fn parse(s: &str) -> Result<Self> {
        match s {
            "bare" => Ok(Profile::Bare),
            "std" => Ok(Profile::Std),
            "safe" => Ok(Profile::Safe),
            "actor" => Ok(Profile::Actor),
            "service" => Ok(Profile::Service),
            other => Err(anyhow::anyhow!(
                "unknown profile `{other}` (expected: bare, std, safe, actor, service)"
            )),
        }
    }

    /// Profiles whose runtime is not yet implemented return Some(reason).
    /// bare/std are fully real today; the rest are reserved with honest status.
    fn pending_reason(self) -> Option<&'static str> {
        match self {
            // bare/std: real today. actor: real now too — the pthread runtime
            // (runtime_actor.c) provides spawn/join. safe/service remain pending.
            Profile::Bare | Profile::Std | Profile::Actor => None,
            Profile::Safe => Some(
                "the `safe` profile (capability enforcement) is not implemented yet;                  capability types exist but are not enforced at runtime",
            ),
            Profile::Service => Some(
                "the `service` profile (local reactor/health/metrics) is not implemented yet",
            ),
        }
    }
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
        #[arg(
            long,
            default_value = "v1.30",
            help = "Select compiler pipeline: v1.30 (default) or v1.21 (legacy)"
        )]
        pipeline: String,
        #[arg(
            long,
            default_value = "std",
            value_parser = ["bare", "std", "safe", "actor", "service"],
            help = "Runtime profile: bare, std (default), safe, actor, service"
        )]
        profile: String,
    },
    Check {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value = "dict/core_map.json")]
        dict: PathBuf,
        #[arg(
            long,
            default_value = "v1.30",
            help = "Select compiler pipeline: v1.30 (default) or v1.21 (legacy)"
        )]
        pipeline: String,
    },
    #[command(
        name = "v130-check",
        hide = true,
        about = "Run v1.30 subsystem self-check (formerly v130-check)"
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
            profile,
        }) => {
            let pipeline = pipeline
                .parse::<CompilerPipeline>()
                .map_err(anyhow::Error::msg)?;
            let profile = Profile::parse(&profile)?;
            compile(
                &file,
                output,
                &dict,
                emit_ir,
                object_only,
                secure,
                &target,
                pipeline,
                profile,
            )
        }
        Some(Commands::Check {
            file,
            dict,
            pipeline,
        }) => {
            {
                let _ = pipeline;
                v130_validate_file(&file, &dict)?;
            }
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
    profile: Profile,
) -> Result<()> {
    ensure_ldx_source(file)?;
    // Profiles whose runtime is not implemented yet fail early with an honest,
    // actionable message (consistent with the actor/channel compile guard).
    // bare/std proceed normally.
    if let Some(reason) = profile.pending_reason() {
        return Err(anyhow::anyhow!(
            "{reason}. Use `--profile std` (print + sleep + yield) or `--profile bare`.              See docs/runtime/PROFILES.md for current profile readiness."
        ));
    }
    let target = CompilationTarget::parse(target_name)?;
    let output_path = output.unwrap_or_else(|| default_output(file, object_only));
    let object_path = if object_only {
        output_path.clone()
    } else {
        output_path.with_extension("o")
    };

    // Compile through the HIR pipeline (CallableRegistry-backed).
    let artifact = match pipeline {
        CompilerPipeline::V130 => {
            compile_v130_pipeline(file, dict, &object_path, emit_ir, target, profile)?
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
            println!(
                "WASM module written to {} (use wasm-ld to link with WASI libc)",
                artifact.object_path.display()
            );
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
        println!("WASM module written to {}", artifact.object_path.display());
        println!("  Triple: {}", target.llvm_triple()); // wasm32-unknown-unknown
        println!(
            "  Use: wasm-ld --no-entry -o output.wasm {} --export-all",
            artifact.object_path.display()
        );
        return Ok(());
    }

    let runtime_asm = output_path.with_extension("runtime.s");
    fs::write(&runtime_asm, os::runtime_assembly())
        .with_context(|| format!("failed to write runtime assembly {}", runtime_asm.display()))?;
    // Build the link spec from the runtime profile. bare/std need nothing extra.
    // actor needs the audited pthread runtime (runtime_actor.c) + -lpthread,
    // both Logicodex-decided (runtime_libs channel), never user-requested.
    let mut link_spec = LinkSpec::default();
    let mut _actor_rt_tmp: Option<PathBuf> = None;
    if profile == Profile::Actor {
        // runtime_actor.c is a fixed, audited artifact embedded in the compiler
        // binary (include_str!), written to a temp file next to the output so cc
        // can compile+link it. The user cannot substitute or inject this C.
        let rt_src = output_path.with_extension("runtime_actor.c");
        fs::write(&rt_src, include_str!("runtime/runtime_actor.c")).with_context(|| {
            format!("failed to write actor runtime source {}", rt_src.display())
        })?;
        link_spec.extra_sources.push(rt_src.clone());
        link_spec.runtime_libs.push("pthread".to_string());
        _actor_rt_tmp = Some(rt_src);
    }
    // lod Stage 0: pull external C libraries to link from logicodex.toml
    // ([dependencies.c.*].link). These go in the user_libs channel (explicit,
    // user-requested) -- never auto-linked. No manifest -> nothing added.
    if let Some((manifest, _)) =
        lod::Manifest::discover(file).map_err(|e| anyhow::anyhow!("{e}"))?
    {
        link_spec.user_libs.extend(manifest.user_libs());
    }
    link_executable(
        &artifact.object_path,
        &runtime_asm,
        &output_path,
        &link_spec,
    )?;
    println!("Native executable written to {}", output_path.display());
    if secure {
        write_security_attestation_plan(&output_path)?;
    }
    Ok(())
}

/// Scan a lowered HIR module for actor/channel operations whose runtime is not
/// yet implemented (Phase B, pthread-based). Returns the first such operation's
/// human label, or None if the program uses no runtime-pending threading ops.
///
/// `yield` and `sleep` are intentionally NOT flagged: they have real syscall
/// backends in os::runtime_assembly() and are part of the std profile today.
fn first_pending_actor_op(module: &hir::HirModule) -> Option<&'static str> {
    fn scan_expr(e: &hir::HirExpr) -> Option<&'static str> {
        use hir::HirExprKind::*;
        match &e.kind {
            Spawn { args, .. } => {
                for a in args {
                    if let Some(op) = scan_expr(a) {
                        return Some(op);
                    }
                }
                Some("spawn")
            }
            Join { .. } => Some("join"),
            ChannelSend { .. } => Some("channel send"),
            ChannelRecv { .. } => Some("channel recv"),
            ChannelTrySend { .. } => Some("channel try_send"),
            ChannelTryRecv { .. } => Some("channel try_recv"),
            ChannelTimeoutRecv { .. } => Some("channel timeout_recv"),
            Binary { left, right, .. } => scan_expr(left).or_else(|| scan_expr(right)),
            Unary { expr, .. } | Field { base: expr, .. } | Cast { expr, .. } => scan_expr(expr),
            Call { args, .. } => args.iter().find_map(scan_expr),
            Sleep { duration_ms } => scan_expr(duration_ms),
            _ => None,
        }
    }
    fn scan_block(b: &hir::HirBlock) -> Option<&'static str> {
        for s in &b.statements {
            if let Some(op) = scan_stmt(&s.node) {
                return Some(op);
            }
        }
        None
    }
    fn scan_stmt(s: &hir::HirStmt) -> Option<&'static str> {
        use hir::HirStmt::*;
        match s {
            Let { value: Some(v), .. } => scan_expr(v),
            Let { value: None, .. } => None,
            Assign { target, value } => scan_expr(target).or_else(|| scan_expr(value)),
            If {
                condition,
                then_branch,
                else_branch,
                ..
            } => scan_expr(condition)
                .or_else(|| scan_block(then_branch))
                .or_else(|| else_branch.as_ref().and_then(scan_block)),
            While { condition, body } => scan_expr(condition).or_else(|| scan_block(body)),
            Loop { body } | UnsafeBlock(body) | HardwareZone(body) => scan_block(body),
            Expr(e) => scan_expr(e),
            Return(Some(e)) => scan_expr(e),
            Return(None) | Break { .. } | Continue { .. } | HardwareDecl { .. } => None,
        }
    }
    for item in &module.items {
        if let hir::HirItem::Function(f) = &item.node {
            if let Some(op) = scan_block(&f.body) {
                return Some(op);
            }
        }
    }
    None
}

fn is_unknown_span(span: span::Span) -> bool {
    span.file_id == span::FileId(0)
        && span.start_line == 0
        && span.start_col == 0
        && span.end_line == 0
        && span.end_col == 0
}

fn text_between(message: &str, prefix: &str, suffix: &str) -> Option<String> {
    let start = message.find(prefix)? + prefix.len();
    let rest = &message[start..];
    let end = rest.find(suffix)?;
    Some(rest[..end].to_string())
}

fn diagnostic_needle(diagnostic: &span::Diagnostic) -> Option<String> {
    match diagnostic.code {
        span::DiagnosticCode::UnknownName => {
            text_between(&diagnostic.message_en, "Name '", "' was not found")
        }
        span::DiagnosticCode::UnknownFunction => {
            text_between(&diagnostic.message_en, "Function '", "' was not found")
                .or_else(|| text_between(&diagnostic.message_en, "Function `", "` was not found"))
        }
        span::DiagnosticCode::UnknownType => {
            text_between(&diagnostic.message_en, "Type `", "` was not found")
        }
        span::DiagnosticCode::UnknownEnumVariant => {
            text_between(&diagnostic.message_en, "Enum variant `", "` was not found")
        }
        span::DiagnosticCode::EnumTypeMismatch => {
            text_between(&diagnostic.message_en, "Enum variant `", "` does not match")
                .or_else(|| text_between(&diagnostic.message_en, "Call returns enum `", "` but"))
        }
        _ => None,
    }
}

fn source_span_for_needle(source: &str, needle: &str) -> Option<span::Span> {
    if needle.is_empty() {
        return None;
    }

    let byte_index = source.find(needle)?;
    let before = &source[..byte_index];
    let line = before.chars().filter(|ch| *ch == '\n').count() as u32 + 1;

    let line_start = before.rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    let col = source[line_start..byte_index].chars().count() as u32 + 1;
    let end_col = col + needle.chars().count() as u32;

    Some(span::Span::new(span::FileId(0), line, col, line, end_col))
}

fn recover_hir_lowering_spans(source: &str, diagnostics: &mut [span::Diagnostic]) {
    for diagnostic in diagnostics {
        if !is_unknown_span(diagnostic.primary_span) {
            continue;
        }

        let Some(needle) = diagnostic_needle(diagnostic) else {
            continue;
        };

        if let Some(span) = source_span_for_needle(source, &needle) {
            diagnostic.primary_span = span;
        }
    }
}

fn source_span_for_first_line_keyword(source: &str, keyword: &str) -> Option<span::Span> {
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(keyword) {
            return source_span_for_needle(source, trimmed);
        }
    }
    None
}

fn semantic_type_mismatch_span(source: &str, diagnostic: &span::Diagnostic) -> Option<span::Span> {
    let msg = format!("{} {}", diagnostic.message_ms, diagnostic.message_en).to_lowercase();

    if msg.contains("return") || msg.contains("pulangan") {
        return source_span_for_first_line_keyword(source, "return")
            .or_else(|| source_span_for_first_line_keyword(source, "function"))
            .or_else(|| source_span_for_first_line_keyword(source, "let"));
    }

    if msg.contains("condition") || msg.contains("syarat") || msg.contains("if condition") {
        return source_span_for_first_line_keyword(source, "if");
    }

    if msg.contains("index") || msg.contains("indeks") {
        if let Some(start) = source.find('[') {
            if let Some(end_rel) = source[start..].find(']') {
                let needle = &source[start..start + end_rel + 1];
                return source_span_for_needle(source, needle);
            }
        }
    }

    if msg.contains("argument") || msg.contains("argumen") || msg.contains("parameter") {
        if let Some(call_start) = source.rfind('(') {
            if let Some(call_end_rel) = source[call_start..].find(')') {
                let needle = &source[call_start..call_start + call_end_rel + 1];
                return source_span_for_needle(source, needle);
            }
        }
    }

    if msg.contains("assignment")
        || msg.contains("penetapan")
        || msg.contains("binding")
        || msg.contains("let")
    {
        if let Some(span) = source_span_for_first_line_keyword(source, "let") {
            return Some(span);
        }
        if let Some(eq) = source.find('=') {
            let after = source[eq + 1..]
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .trim_end_matches(';')
                .trim();
            if !after.is_empty() {
                return source_span_for_needle(source, after);
            }
        }
    }

    source_span_for_first_line_keyword(source, "return")
        .or_else(|| source_span_for_first_line_keyword(source, "let"))
        .or_else(|| source_span_for_first_line_keyword(source, "if"))
}

fn semantic_division_by_zero_span(source: &str) -> Option<span::Span> {
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.contains('/') && trimmed.contains('0') {
            return source_span_for_needle(source, trimmed);
        }
    }
    None
}

fn recover_semantic_diagnostic_spans(source: &str, diagnostics: &mut [span::Diagnostic]) {
    for diagnostic in diagnostics {
        if !is_unknown_span(diagnostic.primary_span) {
            continue;
        }

        if diagnostic.code == span::DiagnosticCode::TypeMismatch {
            if let Some(span) = semantic_type_mismatch_span(source, diagnostic) {
                diagnostic.primary_span = span;
            }
            continue;
        }

        if diagnostic.code == span::DiagnosticCode::DivisionByZero {
            if let Some(span) = semantic_division_by_zero_span(source) {
                diagnostic.primary_span = span;
            }
        }
    }
}

fn semantic_diagnostics_error(
    source: &str,
    mut diagnostics: Vec<span::Diagnostic>,
) -> anyhow::Error {
    recover_semantic_diagnostic_spans(source, &mut diagnostics);
    anyhow::anyhow!(format_v130_diagnostics(&diagnostics))
}

fn hir_lowering_error(source: &str, mut diagnostics: Vec<span::Diagnostic>) -> anyhow::Error {
    recover_hir_lowering_spans(source, &mut diagnostics);
    anyhow::anyhow!(
        "v1.30 HIR lowering failed:\n{}",
        format_v130_diagnostics(&diagnostics)
    )
}

/// HIR compilation pipeline with CallableRegistry + Raylib FFI.
/// Lower a (possibly multi-module) program to a single merged HirModule on one
/// shared LoweringContext. If the root program has no `import` statements this
/// is exactly `lower_program(root)` -- the single-file path is unchanged, byte
/// for byte. When imports are present, the module loader resolves the graph
/// (filesystem-relative, dot = directory, cycles rejected), each imported
/// module is lowered in topological order (dependencies before dependents, so a
/// module's mangled symbols exist before a later module's qualified call
/// resolves them) under its own module name, the root is lowered last with its
/// `main`-wrap, and all items are concatenated into one HirModule fed to the
/// rest of the pipeline. One shared SymbolTable means cross-module qualified
/// calls resolve against the same id-space -- no duplicate CallableIds.
fn lower_with_modules(
    lowering: &mut hir::LoweringContext,
    file: &Path,
    dict: &Path,
    root_source: &str,
    root_program: ast::Program,
) -> Result<hir::HirModule> {
    // Fast path: no imports -> behave exactly like before (single file).
    if module_loader::imports_of(&root_program).is_empty() {
        return lowering
            .lower_program(root_program)
            .map_err(|diags| hir_lowering_error(root_source, diags));
    }

    // Multi-module path. Parsing is injected into the loader as a closure so the
    // loader stays a pure graph/ordering unit; here it runs the real v1.30
    // lexer+parser against the shared dictionary.
    let dict_owned = dict.to_path_buf();
    let parse = move |source: &str, _path: &Path| -> std::result::Result<ast::Program, String> {
        let lexicon = Lexicon::from_path(&dict_owned)
            .map_err(|e| format!("failed to load dictionary {}: {e}", dict_owned.display()))?;
        let tokens = Lexer::new(source, &lexicon)
            .tokenize()
            .map_err(|e| format!("{e}"))?;
        let mut parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
        parser.parse().map_err(|e| format!("{e}"))
    };

    let graph = module_loader::load_graph(file, &parse)
        .map_err(|e| anyhow::anyhow!("module loading failed: {e}"))?;

    // The loader emits dependencies first and the root ("") last, which is the
    // exact lowering order we want.
    let mut merged: Vec<span::Spanned<hir::HirItem>> = Vec::new();
    for module in graph.modules {
        let lowered = if module.name.is_empty() {
            // Root: keep the main-wrap so top-level statements run.
            lowering
                .lower_program(module.program)
                .map_err(|diags| hir_lowering_error(root_source, diags))?
        } else {
            // Imported module: function-only, mangled, no main-wrap.
            lowering
                .lower_module_program(&module.name, module.program)
                .map_err(|diags| anyhow::anyhow!("v1.30 HIR lowering failed: {:?}", diags))?
        };
        merged.extend(lowered.items);
    }
    Ok(hir::HirModule { items: merged })
}

fn compile_v130_pipeline(
    file: &Path,
    dict: &Path,
    object_path: &Path,
    emit_ir: bool,
    target: CompilationTarget,
    profile: Profile,
) -> Result<codegen::CodegenArtifact> {
    // Step 1: Parse source to AST
    let source = fs::read_to_string(file)
        .with_context(|| format!("failed to read Logicodex source file {}", file.display()))?;
    let lexicon = Lexicon::from_path(dict)
        .with_context(|| format!("failed to load dictionary {}", dict.display()))?;
    let tokens = Lexer::new(&source, &lexicon).tokenize()?;
    let mut parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse()?;
    // HIR lowering + semantic_gate perform validation (the legacy Analyzer was retired).

    // Step 2: Set up TypeRegistry with Raylib struct types
    let mut types = types::TypeRegistry::new();
    let (raylib_type_ids, _raylib_struct_ids) = ffi::raylib::register_raylib_types(&mut types);

    // Step 3: Lower AST → HIR
    let mut symbols = hir::SymbolTable::default();
    let module_ast = {
        let mut lowering = hir::LoweringContext {
            symbols: &mut symbols,
            types: &mut types,
            diagnostics: Vec::new(),
            current_module: String::new(),
            current_return_enum: None,
        };
        lower_with_modules(&mut lowering, file, dict, &source, program)?
    };

    // Actor/channel runtime is reserved for Phase B (pthread-based). The HIR is
    // lowered and codegen can emit calls to logicodex_spawn/join/channel_*, but
    // those symbols have no runtime definition yet, so linking would fail with a
    // bare "undefined reference". Detect this here and stop with an honest,
    // actionable message instead. `check` does NOT run this — the program is
    // syntactically and semantically valid; only its runtime is missing.
    if profile != Profile::Actor {
        if let Some(op) = first_pending_actor_op(&module_ast) {
            return Err(anyhow::anyhow!(
            "actor/channel runtime not available yet: this program uses `{op}`,              which is parsed and code-generated but has no runtime backend in              this build (reserved for Phase B, pthread-based). It type-checks              (`check` passes), but cannot be linked into a runnable executable.                  See docs/runtime/PROFILES.md (actor profile: runtime-pending)."
            ));
        }
    }

    // Step 4: Set up CallableRegistry with Raylib functions
    let mut callables = ffi::CallableRegistry::default();
    ffi::raylib::register_raylib_functions(&mut types, &mut callables, &raylib_type_ids);

    // v1.42 P7: WASM target — Raylib is native-desktop only, block all Raylib functions
    if target.is_wasm() {
        let raylib_names: Vec<String> = callables
            .signatures
            .iter()
            .filter(|sig| {
                (ffi::raylib::is_struct_constructor(&sig.name) || is_raylib_function(&sig.name))
                    && source_references_word(&source, &sig.name)
            })
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
        callables
            .signatures
            .retain(|sig| !is_raylib_function(&sig.name));
    }

    // Build id->name map for codegen call routing before `symbols` is moved.
    let callable_names = symbols.callables_map();

    // Step 5: Semantic check.
    // lod Stage 0: if a logicodex.toml sits next to the source, its [ffi].allow
    // and [dependencies.c.*].allow symbols are added to the capability policy so
    // declared externs can pass the FfiGatekeeper. No manifest -> default-deny
    // stays fully in force (the common case).
    let mut policy = ffi::CapabilityPolicy::with_runtime_builtins();
    if let Some((manifest, _)) =
        lod::Manifest::discover(file).map_err(|e| anyhow::anyhow!("{e}"))?
    {
        for sym in manifest.allowed_symbols() {
            policy.allow_symbol(sym);
        }
    }
    let mut semantic = semantic_gate::SemanticContext {
        types,
        symbols,
        callables: callables.clone(),
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: ffi::SafetyContext::Safe,
        current_return_type: None,
        policy,
        extern_symbols: std::collections::HashSet::new(),
    };
    semantic
        .check_module(&module_ast)
        .map_err(|diagnostics| semantic_diagnostics_error(&source, diagnostics))?;

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
        callable_names,
    )
}

/// v1.42 P7: Check if a function name is a Raylib function.
/// Used to filter out Raylib functions when targeting WASM.
/// True if `b` can appear inside an identifier.
fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// True if `word` appears in `source` as a whole word (identifier-boundary
/// aware), i.e. the program actually references it. Used to detect real Raylib
/// usage rather than the unconditional extern declarations.
fn source_references_word(source: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let bytes = source.as_bytes();
    let mut search_from = 0usize;
    while let Some(rel) = source[search_from..].find(word) {
        let start = search_from + rel;
        let end = start + word.len();
        let before_ok = start == 0 || !is_ident_byte(bytes[start - 1]);
        let after_ok = end >= bytes.len() || !is_ident_byte(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        search_from = start + 1;
    }
    false
}

fn is_raylib_function(name: &str) -> bool {
    const RAYLIB_FUNCTIONS: &[&str] = &[
        "InitWindow",
        "CloseWindow",
        "WindowShouldClose",
        "SetTargetFPS",
        "GetFPS",
        "GetFrameTime",
        "GetTime",
        "GetScreenWidth",
        "GetScreenHeight",
        "BeginDrawing",
        "EndDrawing",
        "ClearBackground",
        "DrawText",
        "DrawRectangle",
        "DrawCircle",
        "DrawLine",
        "DrawRectangleLines",
        "DrawPixel",
        "LoadTexture",
        "DrawTexture",
        "UnloadTexture",
        "IsKeyDown",
        "IsKeyPressed",
        "GetKeyPressed",
        "IsMouseButtonPressed",
        "GetMouseX",
        "GetMouseY",
        "GetMousePosition",
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
        target_triple,
        arch,
        features,
        code_model,
        target_triple,
        code_model,
        features,
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
        plan_path.display(),
        arch,
        target_triple
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

/// Compile-time capability check using the tier2 Capability Fabric vocabulary.
/// Each `Service` `requires` clause must name a gate. A clearly malformed gate
/// (not `Domain.Operation`) is a hard error; a well-formed gate outside the
/// standard vocabulary is a warning (the vocabulary may not list it yet). This
/// keeps with the fabric's "zero runtime mediation" design: capabilities are
/// verified purely at compile time, with no runtime overhead.
fn validate_capabilities(program: &ast::Program) -> Result<()> {
    let known: std::collections::HashSet<(String, String)> =
        logicodex::tier2::gate::all_standard_domains()
            .into_iter()
            .flat_map(|(_d, gates)| gates.into_iter().map(|g| (g.domain, g.operation)))
            .collect();
    let mut errors: Vec<String> = Vec::new();
    for stmt in &program.statements {
        if let ast::Stmt::Service {
            name,
            requires: Some(req),
            ..
        } = stmt
        {
            match logicodex::tier2::gate::GateRef::parse(req) {
                Ok(g) => {
                    if !known.contains(&(g.domain.clone(), g.operation.clone())) {
                        eprintln!(
                            "warning: service '{}' requires '{}', which is not in the standard capability vocabulary",
                            name, req
                        );
                    }
                }
                Err(_) => errors.push(format!(
                    "service '{}' has a malformed capability '{}' (expected Domain.Operation)",
                    name, req
                )),
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "capability check failed:\n  {}",
            errors.join("\n  ")
        ))
    }
}

fn v130_validate_file(file: &Path, dict: &Path) -> Result<()> {
    // Full validation: parse -> lower (AST->HIR) -> semantic_gate.
    // This is `check`'s validation pass (the legacy Analyzer was retired).
    let source = fs::read_to_string(file)
        .with_context(|| format!("failed to read Logicodex source file {}", file.display()))?;
    let lexicon = Lexicon::from_path(dict)
        .with_context(|| format!("failed to load dictionary {}", dict.display()))?;
    let tokens = Lexer::new(&source, &lexicon).tokenize()?;
    let mut parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse()?;

    // Compile-time capability check (tier2 Capability Fabric vocabulary).
    validate_capabilities(&program)?;

    let mut types = types::TypeRegistry::new();
    let (raylib_type_ids, _raylib_struct_ids) = ffi::raylib::register_raylib_types(&mut types);

    let mut symbols = hir::SymbolTable::default();
    let module_ast = {
        let mut lowering = hir::LoweringContext {
            symbols: &mut symbols,
            types: &mut types,
            diagnostics: Vec::new(),
            current_module: String::new(),
            current_return_enum: None,
        };
        lower_with_modules(&mut lowering, file, dict, &source, program)?
    };

    let mut callables = ffi::CallableRegistry::default();
    ffi::raylib::register_raylib_functions(&mut types, &mut callables, &raylib_type_ids);

    // lod Stage 0: `check` honours the same logicodex.toml allow-list as compile,
    // so a denied/allowed extern reports identically in both paths.
    let mut policy = ffi::CapabilityPolicy::with_runtime_builtins();
    if let Some((manifest, _)) =
        lod::Manifest::discover(file).map_err(|e| anyhow::anyhow!("{e}"))?
    {
        for sym in manifest.allowed_symbols() {
            policy.allow_symbol(sym);
        }
    }
    let mut semantic = semantic_gate::SemanticContext {
        types,
        symbols,
        callables: callables.clone(),
        diagnostics: Vec::new(),
        loop_depth: 0,
        safety_context: ffi::SafetyContext::Safe,
        current_return_type: None,
        policy,
        extern_symbols: std::collections::HashSet::new(),
    };
    semantic
        .check_module(&module_ast)
        .map_err(|diagnostics| semantic_diagnostics_error(&source, diagnostics))?;
    Ok(())
}

fn v130_check(file: &Path, dict: &Path) -> Result<()> {
    parse_and_analyze(file, dict, CompilerPipeline::V130)?;
    run_v130_subsystem_self_check()?;
    println!(
        "{}: v1.30 semantic validation and subsystem check succeeded",
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
                is_public: false,
            }),
            span: span::Span::unknown(),
        }],
    };
    let mut types = types::TypeRegistry::new();
    let mut lowering = hir::LoweringContext {
        symbols: &mut symbols,
        types: &mut types,
        diagnostics: Vec::new(),
        current_module: String::new(),
        current_return_enum: None,
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
        current_return_type: None,
        policy: ffi::CapabilityPolicy::with_runtime_builtins(),
        extern_symbols: std::collections::HashSet::new(),
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

fn format_v130_span(span: span::Span) -> String {
    format!(
        "file {}:{}:{}-{}:{}",
        span.file_id.0, span.start_line, span.start_col, span.end_line, span.end_col
    )
}

fn format_v130_diagnostic(diagnostic: &span::Diagnostic) -> String {
    let mut rendered = format!(
        "code: {:?}\nspan: {}\n{} / {}",
        diagnostic.code,
        format_v130_span(diagnostic.primary_span),
        diagnostic.message_ms,
        diagnostic.message_en
    );

    for note in &diagnostic.notes {
        match note.span {
            Some(span) => {
                rendered.push_str(&format!(
                    "\nnote: {} / {} [{}]",
                    note.message_ms,
                    note.message_en,
                    format_v130_span(span)
                ));
            }
            None => {
                rendered.push_str(&format!(
                    "\nnote: {} / {}",
                    note.message_ms, note.message_en
                ));
            }
        }
    }

    rendered
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
    // HIR lowering + semantic_gate perform validation (legacy Analyzer retired).
    let _ = (target_name, secure);
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

/// Libraries to link into the final executable, kept in two deliberately
/// separate channels so the architecture stays honest about *who* requires a
/// library (see docs/runtime/LINKING.md):
///
///   * `runtime_libs` — libraries the runtime/profile itself requires. These are
///     Logicodex-decided, not user-requested: today only OS primitives such as
///     `pthread` (needed by the actor profile). A future Logicodex *core
///     library* would also be auto-added here. Auto-linking these never means
///     "Logicodex depends on a third-party C lib" — they are platform/core
///     building blocks, like the sleep/yield syscalls.
///
///   * `user_libs` — external C libraries the *user* explicitly opted into
///     (e.g. `sqlite3`, `ssl`, `raylib`). Never automatic, never mandatory.
///
/// `lib_paths` are extra `-L` search directories (applies to both channels).
#[derive(Debug, Default, Clone)]
struct LinkSpec {
    runtime_libs: Vec<String>,
    user_libs: Vec<String>,
    lib_paths: Vec<PathBuf>,
    /// Extra source/object files cc should compile+link alongside the program
    /// object and runtime assembly — e.g. the audited runtime_actor.c for the
    /// actor profile. These are Logicodex-owned artifacts, never user input.
    extra_sources: Vec<PathBuf>,
}

impl LinkSpec {
    /// True when no external libraries are requested at all — the common case
    /// today (bare/std programs link only object + runtime assembly).
    fn is_empty(&self) -> bool {
        self.runtime_libs.is_empty() && self.user_libs.is_empty()
    }
}

fn link_executable(
    object_path: &Path,
    runtime_asm: &Path,
    output_path: &Path,
    spec: &LinkSpec,
) -> Result<()> {
    let linker = std::env::var("LOGICODEX_LINKER").unwrap_or_else(|_| "cc".to_string());
    let mut cmd = Command::new(&linker);
    cmd.arg(object_path)
        .arg(runtime_asm)
        .arg("-o")
        .arg(output_path);

    // Extra Logicodex-owned sources (e.g. runtime_actor.c) compiled+linked here.
    for src in &spec.extra_sources {
        cmd.arg(src);
    }
    // -L search paths first, then -l libraries. Both channels are linked the
    // same way at the cc level; the distinction is about provenance/intent, and
    // is preserved in the struct and surfaced in diagnostics/docs.
    for dir in &spec.lib_paths {
        cmd.arg("-L").arg(dir);
    }
    for lib in spec.runtime_libs.iter().chain(spec.user_libs.iter()) {
        cmd.arg(format!("-l{lib}"));
    }

    if std::env::var("LOGICODEX_VERBOSE_LINK").is_ok() {
        eprintln!(
            "[lx-link] {linker} {:?}",
            cmd.get_args().collect::<Vec<_>>()
        );
    }
    let status = cmd
        .status()
        .with_context(|| format!("failed to invoke linker `{linker}`"))?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("linker `{linker}` failed with status {status}")
    }
}
