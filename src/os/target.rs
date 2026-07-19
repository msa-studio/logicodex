// =========================================================================
// Project: Logicodex Language Engine
// Pipeline: single HIR compilation engine (.ldx -> AST -> HIR -> LLVM)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use anyhow::{anyhow, Result};
use inkwell::targets::{
    CodeModel, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use inkwell::OptimizationLevel;

/// CPU architecture for freestanding targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArch {
    X86_64,
    Aarch64,
    Riscv64,
}

impl TargetArch {
    pub fn llvm_triple(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64-unknown-none",
            Self::Aarch64 => "aarch64-unknown-none",
            Self::Riscv64 => "riscv64gc-unknown-none-elf",
        }
    }

    pub fn llvm_features(self) -> &'static str {
        match self {
            // x86_64 has SSE2 — no need for soft-float
            Self::X86_64 => "+sse2",
            // aarch64: default features include NEON
            Self::Aarch64 => "",
            // riscv64gc: includes IMAFD (integer, mult, atomics, float, double)
            Self::Riscv64 => "",
        }
    }

    pub fn code_model(self) -> CodeModel {
        match self {
            Self::X86_64 => CodeModel::Kernel,
            Self::Aarch64 => CodeModel::Small,
            Self::Riscv64 => CodeModel::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationTarget {
    Native,
    /// Freestanding with architecture selection
    Freestanding {
        arch: TargetArch,
    },
    /// WebAssembly target — generates .wasm via LLVM WASM backend
    Wasm,
}

impl CompilationTarget {
    pub fn parse(value: &str) -> Result<Self> {
        // Support freestanding-<arch> syntax
        if value.starts_with("freestanding-") {
            let arch_str = &value["freestanding-".len()..];
            let arch = match arch_str {
                "x86_64" | "x64" => TargetArch::X86_64,
                "aarch64" | "arm64" => TargetArch::Aarch64,
                "riscv64" | "riscv" | "rv64" => TargetArch::Riscv64,
                other => {
                    return Err(anyhow!(
                        "unsupported freestanding architecture `{other}`; \
                     expected `x86_64`, `aarch64`, or `riscv64`"
                    ))
                }
            };
            return Ok(Self::Freestanding { arch });
        }

        match value {
            "native" | "host" => Ok(Self::Native),
            "freestanding" => Ok(Self::Freestanding {
                arch: TargetArch::X86_64,
            }),
            "wasm" | "wasm32" => Ok(Self::Wasm),
            other => Err(anyhow!(
                "unsupported Logicodex target `{other}`; \
                 expected `native`, `host`, `freestanding`, `freestanding-x86_64`, \
                 `freestanding-aarch64`, `freestanding-riscv64`, or `wasm`"
            )),
        }
    }

    pub fn entry_symbol(self) -> &'static str {
        match self {
            Self::Native => "main",
            Self::Freestanding { .. } => "_start",
            Self::Wasm => "_start",
        }
    }

    pub fn is_freestanding(self) -> bool {
        matches!(self, Self::Freestanding { .. })
    }

    /// Check if this target is WebAssembly.
    pub fn is_wasm(self) -> bool {
        matches!(self, Self::Wasm)
    }

    /// LLVM target triple string.
    pub fn llvm_triple(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Freestanding { arch } => arch.llvm_triple(),
            Self::Wasm => "wasm32-unknown-unknown",
        }
    }

    /// Get the target architecture (for freestanding targets).
    pub fn arch(self) -> Option<TargetArch> {
        match self {
            Self::Freestanding { arch } => Some(arch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputKind {
    Object,
    FreestandingObject,
    /// WebAssembly module — .wasm output
    WasmModule,
}

pub fn build_target_machine(kind: OutputKind) -> Result<TargetMachine> {
    build_target_machine_with_arch(kind, TargetArch::X86_64)
}

/// Build target machine with architecture selection.
/// Supports x86_64 (default), aarch64, and riscv64 for freestanding targets.
pub fn build_target_machine_with_arch(kind: OutputKind, arch: TargetArch) -> Result<TargetMachine> {
    Target::initialize_all(&InitializationConfig::default());

    let (triple, cpu, features, reloc, code_model, opt_level) = match kind {
        OutputKind::Object => (
            TargetMachine::get_default_triple(),
            "generic",
            "",
            RelocMode::PIC,
            CodeModel::Default,
            OptimizationLevel::Aggressive,
        ),
        OutputKind::FreestandingObject => (
            TargetTriple::create(arch.llvm_triple()),
            match arch {
                TargetArch::X86_64 => "x86-64",
                TargetArch::Aarch64 => "generic",
                TargetArch::Riscv64 => "generic-rv64",
            },
            // Architecture-specific features (not +soft-float for x86_64)
            arch.llvm_features(),
            RelocMode::Static,
            arch.code_model(),
            OptimizationLevel::Aggressive,
        ),
        OutputKind::WasmModule => (
            TargetTriple::create("wasm32-unknown-unknown"),
            "generic",
            // WASM features — bulk-memory for memcpy, mutable-globals
            "+bulk-memory,+mutable-globals,+sign-ext",
            RelocMode::Static,
            CodeModel::Default,
            OptimizationLevel::Default,
        ),
    };

    let target = Target::from_triple(&triple).map_err(|e| {
        anyhow!(
            "failed to load LLVM target for {}: {e}",
            triple.as_str().to_string_lossy()
        )
    })?;
    target
        .create_target_machine(&triple, cpu, features, opt_level, reloc, code_model)
        .ok_or_else(|| {
            anyhow!(
                "failed to create LLVM target machine for {}",
                triple.as_str().to_string_lossy()
            )
        })
}
