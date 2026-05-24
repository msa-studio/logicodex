// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
mod ast;
mod codegen;
mod lexer;
mod os;
mod parser;
mod semantic;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser as ClapParser, Subcommand};
use codegen::{CodegenOptions, LlvmCompiler, MemoryIntegrityPlan, PhysicalMemoryAccessPlan};
use lexer::{Lexer, Lexicon};
use os::target::CompilationTarget;
use parser::Parser;
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
             [ LOGICODEX COMPILER v1.21-alpha ]
             [ PRACTICAL LLVM COMPILER CORE  ]
=========================================================
Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)"#;

const LOGICODEX_LONG_VERSION: &str = r#"=========================================================
  _                 _               _                 
 | |    ___   __ _ (_)  ___  ___   __| |  ___ __  __  
 | |   / _ \ / _` || | / __|/ _ \ / _` | / _ \ \/ /  
 | |__| (_) | (_| || || (__| (_) | (_| ||  __/ >  <   
 |_____\___/ \__, ||_| \___|\___/ \__,_| \___|/_/\_\  
             |___/                                    
             [ LOGICODEX COMPILER v1.21-alpha ]
             [ PRACTICAL LLVM COMPILER CORE  ]
=========================================================
Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
logicodex 1.21-alpha
Security Roadmap: specification baseline and practical severity model"#;

#[derive(Debug, ClapParser)]
#[command(
    name = "logicodex",
    version = "1.21-alpha",
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
        #[arg(long, default_value = "native", value_parser = ["native", "host", "freestanding"], help = "Select host/native OS linkage or experimental freestanding object generation")]
        target: String,
    },
    Check {
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
        }) => compile(&file, output, &dict, emit_ir, object_only, secure, &target),
        Some(Commands::Check { file, dict }) => {
            parse_and_analyze(&file, &dict)?;
            println!("{}: semantic validation succeeded", file.display());
            Ok(())
        }
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
) -> Result<()> {
    ensure_ldx_source(file)?;
    let target = CompilationTarget::parse(target_name)?;
    let program = parse_and_analyze_for_target(file, dict, target_name, secure)?;
    let output_path = output.unwrap_or_else(|| default_output(file, object_only));
    let object_path = if object_only {
        output_path.clone()
    } else {
        output_path.with_extension("o")
    };

    let artifact = LlvmCompiler::compile_to_object(
        &program,
        &object_path,
        &CodegenOptions {
            module_name: module_name(file),
            emit_ir,
            secure,
            target,
        },
    )?;
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

fn write_security_attestation_plan(output_path: &Path) -> Result<()> {
    let mut plan_path = output_path.to_path_buf();
    plan_path.set_extension("security.md");
    let content = format!(
        "# Logicodex Runtime Memory Integrity Verification Plan\n\nTarget artifact: `{}`\n\nSecurity roadmap: v1.21-alpha specification baseline and practical severity model.\n\nThe `--secure` compilation path currently records the security roadmap for the target program. The long-term objective is to prototype cryptographic digest insertion over immutable executable regions, add verifier stubs, and define target-appropriate fail-stop behavior after threat models, tests, and overhead measurements exist. A future mismatch response should be implemented conservatively: hosted targets should prefer ordinary process termination, while freestanding targets should use halt or reset behavior only when explicitly configured and documented.\n\nMemory integrity plan: `{:?}`\n",
        output_path.display(),
        MemoryIntegrityPlan::hardened_default()
    );
    fs::write(&plan_path, content).with_context(|| {
        format!(
            "failed to write security attestation plan {}",
            plan_path.display()
        )
    })?;
    println!(
        "Security attestation plan written to {}",
        plan_path.display()
    );
    Ok(())
}

fn write_freestanding_plan(output_path: &Path) -> Result<()> {
    let mut plan_path = output_path.to_path_buf();
    plan_path.set_extension("freestanding.md");
    let access_plan = PhysicalMemoryAccessPlan::freestanding_default();
    let content = format!(
        "# Logicodex Freestanding Target Plan\n\nTarget artifact: `{}`\n\nCompilation profile: v1.21-alpha specification baseline and practical severity model.\n\nThe `--target freestanding` path is experimental and emits a raw object intended for future bootloader, kernel, hypervisor, or firmware integration work. The backend selects a freestanding LLVM target triple, static relocation, kernel code model, and `_start` entry symbol. Platform startup objects such as Linux crt0 or Windows subsystem entry points are intentionally excluded.\n\nPhysical memory access plan: `{:?}`\n\nThe planned `*int` raw pointer representation is reserved for memory-mapped I/O, including VGA text memory at `0xB8000` and serial UART ports such as `0x3F8`, under explicit backend safety gates. This is a freestanding or kernel-authority operation: hosted Linux or Windows processes with virtual memory paging and ASLR cannot directly manipulate physical addresses without kernel-space mapping such as `/dev/mem` or Ring-0 driver mediation.\n",
        output_path.display(),
        access_plan
    );
    fs::write(&plan_path, content).with_context(|| {
        format!(
            "failed to write freestanding target plan {}",
            plan_path.display()
        )
    })?;
    println!(
        "Freestanding target plan written to {}",
        plan_path.display()
    );
    Ok(())
}

fn parse_and_analyze(file: &Path, dict: &Path) -> Result<ast::Program> {
    parse_and_analyze_for_target(file, dict, "native", false)
}

fn parse_and_analyze_for_target(
    file: &Path,
    dict: &Path,
    target_name: &str,
    secure: bool,
) -> Result<ast::Program> {
    ensure_ldx_source(file)?;
    let source = fs::read_to_string(file)
        .with_context(|| format!("failed to read Logicodex source file {}", file.display()))?;
    let lexicon = Lexicon::from_path(dict)
        .with_context(|| format!("failed to load dictionary {}", dict.display()))?;
    let tokens = Lexer::new(&source, &lexicon).tokenize()?;
    let mut parser = Parser::new(tokens);
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
