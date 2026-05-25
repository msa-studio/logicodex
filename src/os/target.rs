// =========================================================================
// Project: Logicodex Language Engine (Phase 2 Deployment Integration)
// Version: v1.21-alpha (Specification Baseline & Practical Severity Roadmap)
// Architect & Creator: Mohamad Supardi Abdul (mymsastudio@gmail.com)
// Copyright (c) 2026. All Rights Reserved.
// Licensed under permissive dual-license: MIT & Apache License 2.0
// =========================================================================
use anyhow::{anyhow, Result};
use inkwell::targets::{
    CodeModel, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use inkwell::OptimizationLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationTarget {
    Native,
    Freestanding,
    /// v1.40: WebAssembly target — generates .wasm via LLVM WASM backend
    Wasm,
}

impl CompilationTarget {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "native" | "host" => Ok(Self::Native),
            "freestanding" => Ok(Self::Freestanding),
            "wasm" | "wasm32" => Ok(Self::Wasm),
            other => Err(anyhow!(
                "unsupported Logicodex target `{other}`; expected `native`, `host`, `freestanding`, or `wasm`"
            )),
        }
    }

    pub fn entry_symbol(self) -> &'static str {
        match self {
            Self::Native => "main",
            Self::Freestanding => "_start",
            Self::Wasm => "_start",
        }
    }

    pub fn is_freestanding(self) -> bool {
        matches!(self, Self::Freestanding)
    }

    /// Check if this target is WebAssembly.
    pub fn is_wasm(self) -> bool {
        matches!(self, Self::Wasm)
    }

    /// LLVM target triple string.
    pub fn llvm_triple(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Freestanding => "x86_64-unknown-none",
            Self::Wasm => "wasm32-unknown-unknown",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputKind {
    Object,
    FreestandingObject,
    /// v1.40: WebAssembly module — .wasm output
    WasmModule,
}

pub fn build_target_machine(kind: OutputKind) -> Result<TargetMachine> {
    Target::initialize_all(&InitializationConfig::default());
    let triple = match kind {
        OutputKind::Object => TargetMachine::get_default_triple(),
        OutputKind::FreestandingObject => TargetTriple::create("x86_64-unknown-none"),
        OutputKind::WasmModule => TargetTriple::create("wasm32-unknown-unknown"),
    };
    let cpu = match kind {
        OutputKind::Object => "generic",
        OutputKind::FreestandingObject => "x86-64",
        OutputKind::WasmModule => "generic",
    };
    let features = match kind {
        OutputKind::Object => "",
        OutputKind::FreestandingObject => "+soft-float",
        // v1.40: WASM features — bulk-memory for memcpy, mutable-globals
        OutputKind::WasmModule => "+bulk-memory,+mutable-globals,+sign-ext",
    };
    let reloc = match kind {
        OutputKind::Object => RelocMode::PIC,
        OutputKind::FreestandingObject => RelocMode::Static,
        // v1.40: WASM uses static PIC for embedded data
        OutputKind::WasmModule => RelocMode::Static,
    };
    let code_model = match kind {
        OutputKind::Object => CodeModel::Default,
        OutputKind::FreestandingObject => CodeModel::Kernel,
        // v1.40: WASM doesn't use code models
        OutputKind::WasmModule => CodeModel::Default,
    };
    let opt_level = match kind {
        // v1.40: WASM uses size optimization (Oz) for smaller bundles
        OutputKind::WasmModule => OptimizationLevel::Default,
        _ => OptimizationLevel::Aggressive,
    };
    let target = Target::from_triple(&triple).map_err(|e| {
        anyhow!(
            "failed to load LLVM target for {}: {e}",
            triple.as_str().to_string_lossy()
        )
    })?;
    target
        .create_target_machine(
            &triple,
            cpu,
            features,
            opt_level,
            reloc,
            code_model,
        )
        .ok_or_else(|| {
            anyhow!(
                "failed to create LLVM target machine for {}",
                triple.as_str().to_string_lossy()
            )
        })
}
