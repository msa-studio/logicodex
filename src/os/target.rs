// =========================================================================
// Project: Logicodex Language Engine (Phase 1 MVP Upgrade)
// Version: v1.0.1-alpha (Internal Security & OS Freestanding Test)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use anyhow::{anyhow, Result};
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple};
use inkwell::OptimizationLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationTarget {
    Native,
    Freestanding,
}

impl CompilationTarget {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "native" => Ok(Self::Native),
            "freestanding" => Ok(Self::Freestanding),
            other => Err(anyhow!("unsupported Logicodex target `{other}`; expected `native` or `freestanding`")),
        }
    }

    pub fn entry_symbol(self) -> &'static str {
        match self {
            Self::Native => "main",
            Self::Freestanding => "_start",
        }
    }

    pub fn is_freestanding(self) -> bool {
        matches!(self, Self::Freestanding)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputKind {
    Object,
    FreestandingObject,
}

pub fn build_target_machine(kind: OutputKind) -> Result<TargetMachine> {
    Target::initialize_all(&InitializationConfig::default());
    let triple = match kind {
        OutputKind::Object => TargetMachine::get_default_triple(),
        OutputKind::FreestandingObject => TargetTriple::create("x86_64-unknown-none"),
    };
    let cpu = match kind {
        OutputKind::Object => "generic",
        OutputKind::FreestandingObject => "x86-64",
    };
    let features = match kind {
        OutputKind::Object => "",
        OutputKind::FreestandingObject => "+soft-float",
    };
    let reloc = match kind {
        OutputKind::Object => RelocMode::PIC,
        OutputKind::FreestandingObject => RelocMode::Static,
    };
    let code_model = match kind {
        OutputKind::Object => CodeModel::Default,
        OutputKind::FreestandingObject => CodeModel::Kernel,
    };
    let target = Target::from_triple(&triple).map_err(|e| anyhow!("failed to load LLVM target for {}: {e}", triple.as_str().to_string_lossy()))?;
    target.create_target_machine(&triple, cpu, features, OptimizationLevel::Aggressive, reloc, code_model)
        .ok_or_else(|| anyhow!("failed to create LLVM target machine for {}", triple.as_str().to_string_lossy()))
}
