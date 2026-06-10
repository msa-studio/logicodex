// =========================================================================
// Logicodex v1.36.0-alpha — Capability Translation Layer: CTL Mapper (Fasa B)
//
// "Auto-generate WIT from CapabilityGraph — 6 domain mappings."
//
// The CTL Mapper is the bridge between Logicodex's capability-native world
// and the WASM ecosystem. It reads a CapabilityGraph IR and produces:
//   1. WIT (WebAssembly Interface Types) — for WASM component linking
//   2. Populated CapabilityRef.wit_mapping — for runtime host resolution
//   3. Host reactor stubs — for HW gates that need host mediation
//
// 6 Domain Mappings (Logicodex → WASI):
//   Storage  → wasi:filesystem
//   Net      → wasi:sockets
//   UI       → wasi:cli/stdout
//   HW       → HostReactor (NOT standard WASI — host-mediated)
//   Audio    → wasi:io/custom (or web-audio via custom interface)
//   Crypto   → wasi:crypto
//
// Design: "Project INTO, not borrow FROM"
//   - Logicodex domains are primary; WASI is a projection target
//   - HW gates are NEVER directly exposed to WASM guest
//   - All HW access → Host Reactor → safe host-side implementation
// =========================================================================

use super::capability_ir::{
    CapabilityGraph, CapabilityRef,
};


// ─── WitDomain ───
/// Represents a WIT interface domain — the WASI-side projection
/// of a Logicodex capability domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitDomain {
    /// wasi:filesystem — file read, write, seek, sync
    WasiFilesystem,
    /// wasi:sockets — TCP/UDP bind, connect, send, recv
    WasiSockets,
    /// wasi:cli — stdout, stderr, stdin, environment
    WasiCli,
    /// Host Reactor — NOT standard WASI. Hardware access is
    /// mediated by the host reactor. WASM guest sees this as
    /// an import that resolves to host-side implementation.
    HostReactor,
    /// wasi:io/custom — audio streams, or custom host interface
    WasiIoCustom,
    /// wasi:crypto — hash, encrypt, decrypt, random
    WasiCrypto,
    /// Unknown domain — not in the 6 standard mappings
    Unknown(String),
}

impl WitDomain {
    /// Parse a Logicodex domain string into its WIT equivalent.
    pub fn from_logicodex_domain(domain: &str) -> Self {
        match domain {
            "Storage" => WitDomain::WasiFilesystem,
            "Net" => WitDomain::WasiSockets,
            "UI" => WitDomain::WasiCli,
            "HW" => WitDomain::HostReactor,
            "Audio" => WitDomain::WasiIoCustom,
            "Crypto" => WitDomain::WasiCrypto,
            other => WitDomain::Unknown(other.to_string()),
        }
    }

    /// Format as WIT package:interface string.
    pub fn wit_package_interface(&self) -> String {
        match self {
            WitDomain::WasiFilesystem => "wasi:filesystem".to_string(),
            WitDomain::WasiSockets => "wasi:sockets".to_string(),
            WitDomain::WasiCli => "wasi:cli".to_string(),
            WitDomain::HostReactor => "logicodex:host-reactor".to_string(),
            WitDomain::WasiIoCustom => "wasi:io/custom".to_string(),
            WitDomain::WasiCrypto => "wasi:crypto".to_string(),
            WitDomain::Unknown(s) => format!("logicodex:{}", s.to_lowercase()),
        }
    }

    /// Format as WIT world import string.
    pub fn wit_world_import(&self) -> String {
        self.wit_package_interface()
    }

    /// Check if this domain requires host reactor mediation.
    pub fn is_host_reactor(&self) -> bool {
        matches!(self, WitDomain::HostReactor)
    }

    /// Check if this is a known (mappable) domain.
    pub fn is_known(&self) -> bool {
        !matches!(self, WitDomain::Unknown(_))
    }
}

// ─── WitOperation ───
/// Maps a Logicodex operation to its WIT equivalent within a domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitOperation {
    pub logicodex_op: String,
    pub wit_op: String,
    pub params: Vec<(String, String)>, // (param_name, param_type)
    pub returns: Option<String>,
}

impl WitOperation {
    pub fn new(logicodex_op: impl Into<String>, wit_op: impl Into<String>) -> Self {
        Self {
            logicodex_op: logicodex_op.into(),
            wit_op: wit_op.into(),
            params: Vec::new(),
            returns: None,
        }
    }

    pub fn with_param(mut self, name: impl Into<String>, ty: impl Into<String>) -> Self {
        self.params.push((name.into(), ty.into()));
        self
    }

    pub fn with_return(mut self, ty: impl Into<String>) -> Self {
        self.returns = Some(ty.into());
        self
    }

    /// Format as WIT function signature.
    pub fn wit_signature(&self) -> String {
        let params = if self.params.is_empty() {
            "()".to_string()
        } else {
            format!(
                "({})",
                self.params
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        match &self.returns {
            Some(ret) => format!("  {}: func{} → {};", self.wit_op, params, ret),
            None => format!("  {}: func{};", self.wit_op, params),
        }
    }
}

// ─── Domain operation mappings ───
/// Get all known WIT operations for a Logicodex domain.
pub fn get_wit_operations(domain: &str) -> Vec<WitOperation> {
    match domain {
        "Storage" => vec![
            WitOperation::new("Baca", "read")
                .with_param("path", "string")
                .with_param("offset", "u64")
                .with_return("result<list<u8>, error>"),
            WitOperation::new("Tulis", "write")
                .with_param("path", "string")
                .with_param("data", "list<u8>")
                .with_return("result<u64, error>"),
            WitOperation::new("Padam", "delete")
                .with_param("path", "string")
                .with_return("result<unit, error>"),
        ],
        "Net" => vec![
            WitOperation::new("Send", "tcp-send")
                .with_param("socket", "u32")
                .with_param("data", "list<u8>")
                .with_return("result<u64, error>"),
            WitOperation::new("Recv", "tcp-recv")
                .with_param("socket", "u32")
                .with_param("max-len", "u32")
                .with_return("result<list<u8>, error>"),
            WitOperation::new("Raw", "udp-bind") // Raw → UDP (safest equivalent)
                .with_param("addr", "string")
                .with_param("port", "u16")
                .with_return("result<u32, error>"),
        ],
        "UI" => vec![
            WitOperation::new("Papar", "stdout")
                .with_param("text", "string")
                .with_return("result<unit, error>"),
            WitOperation::new("Input", "stdin")
                .with_return("result<string, error>"),
        ],
        "HW" => vec![
            // HW operations are host-reactor mediated
            WitOperation::new("GPIO", "gpio-control")
                .with_param("pin", "u32")
                .with_param("mode", "string")
                .with_return("result<u32, error>"),
            WitOperation::new("Timer", "timer-set")
                .with_param("micros", "u64")
                .with_return("result<u32, error>"),
            WitOperation::new("DMA", "dma-transfer")
                .with_param("src", "u32")
                .with_param("dst", "u32")
                .with_param("len", "u32")
                .with_return("result<u32, error>"),
        ],
        "Audio" => vec![
            WitOperation::new("Main", "play")
                .with_param("buffer", "list<f32>")
                .with_return("result<unit, error>"),
            WitOperation::new("Rakam", "record")
                .with_param("duration-ms", "u32")
                .with_return("result<list<f32>, error>"),
        ],
        "Crypto" => vec![
            WitOperation::new("Hash", "hash")
                .with_param("data", "list<u8>")
                .with_param("algorithm", "string")
                .with_return("result<list<u8>, error>"),
            WitOperation::new("Encrypt", "encrypt")
                .with_param("data", "list<u8>")
                .with_param("key", "list<u8>")
                .with_param("algorithm", "string")
                .with_return("result<list<u8>, error>"),
        ],
        _ => Vec::new(),
    }
}

// ─── CtlMapper ───
/// The Capability Translation Layer Mapper.
/// Auto-generates WIT and populates WIT mappings for a CapabilityGraph.
#[derive(Debug)]
pub struct CtlMapper {
    /// Mappings applied
    mappings_applied: usize,
    /// HW gates detected (require host reactor)
    hw_gates: Vec<CapabilityRef>,
    /// Unknown domains encountered
    unknown_domains: Vec<String>,
    /// Manual overrides: domain.operation → custom WIT string
    overrides: std::collections::HashMap<String, String>,
}

impl CtlMapper {
    /// Create a new CTL Mapper.
    pub fn new() -> Self {
        Self {
            mappings_applied: 0,
            hw_gates: Vec::new(),
            unknown_domains: Vec::new(),
            overrides: std::collections::HashMap::new(),
        }
    }

    /// Add a manual WIT mapping override.
    pub fn add_override(&mut self, logicodex_canonical: impl Into<String>, wit_string: impl Into<String>) {
        self.overrides.insert(logicodex_canonical.into(), wit_string.into());
    }

    /// Map a single CapabilityRef to its WIT equivalent.
    pub fn map_capability(&mut self, cap: &mut CapabilityRef) {
        let key = cap.canonical();

        // 1. Check for manual override
        if let Some(override_wit) = self.overrides.get(&key) {
            cap.wit_mapping = Some(override_wit.clone());
            self.mappings_applied += 1;
            return;
        }

        // 2. Auto-map based on domain
        let wit_domain = WitDomain::from_logicodex_domain(&cap.domain);

        if wit_domain.is_host_reactor() {
            // HW gates: mark for host reactor mediation
            self.hw_gates.push(cap.clone());
            cap.wit_mapping = Some(format!(
                "logicodex:host-reactor/{}-{}: func(pin: u32, mode: string) → result<u32, error>",
                cap.domain.to_lowercase(),
                cap.operation.to_lowercase()
            ));
            self.mappings_applied += 1;
            return;
        }

        if !wit_domain.is_known() {
            self.unknown_domains.push(cap.domain.clone());
            cap.wit_mapping = Some(format!(
                "logicodex:custom/{}",
                cap.operation.to_lowercase()
            ));
            self.mappings_applied += 1;
            return;
        }

        // 3. Look up specific operation mapping
        let operations = get_wit_operations(&cap.domain);
        if let Some(op) = operations.iter().find(|o| o.logicodex_op == cap.operation) {
            let wit = format!(
                "{}/{}",
                wit_domain.wit_package_interface(),
                op.wit_op
            );
            cap.wit_mapping = Some(wit);
        } else {
            // Fallback: just use the operation name
            cap.wit_mapping = Some(format!(
                "{}/{}",
                wit_domain.wit_package_interface(),
                cap.operation.to_lowercase()
            ));
        }
        self.mappings_applied += 1;
    }

    /// Map all capabilities in a CapabilityGraph.
    pub fn map_graph(&mut self, graph: &mut CapabilityGraph) {
        // Map all service requires
        for (_, svc) in &mut graph.services {
            for cap in &mut svc.requires {
                self.map_capability(cap);
            }
            for cap in &mut svc.provides {
                self.map_capability(cap);
            }
        }

        // Map all shard allowed gates
        for (_, shard) in &mut graph.shards {
            for cap in &mut shard.allowed_gates {
                self.map_capability(cap);
            }
        }

        // Map all gate edges
        for gate in &mut graph.gates {
            self.map_capability(&mut gate.capability);
        }
    }

    /// Generate complete WIT document from a CapabilityGraph.
    /// Assumes map_graph() has already been called.
    pub fn generate_wit(&self, graph: &CapabilityGraph) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push("// WIT Auto-Generated by Logicodex CTL Mapper v1.36.0-alpha".to_string());
        lines.push("// Source: CapabilityGraph → WASM component interfaces".to_string());
        lines.push("// Philosophy: Project INTO, not borrow FROM".to_string());
        lines.push("".to_string());

        // Package declaration
        lines.push("package logicodex:generated;".to_string());
        lines.push("".to_string());

        // Collect all unique imported interfaces
        let mut interfaces: std::collections::HashMap<String, Vec<&CapabilityRef>> = std::collections::HashMap::new();
        for (_, svc) in &graph.services {
            for cap in &svc.requires {
                let pkg = WitDomain::from_logicodex_domain(&cap.domain).wit_package_interface();
                interfaces.entry(pkg).or_default().push(cap);
            }
        }
        for (_, shard) in &graph.shards {
            for cap in &shard.allowed_gates {
                let pkg = WitDomain::from_logicodex_domain(&cap.domain).wit_package_interface();
                interfaces.entry(pkg).or_default().push(cap);
            }
        }

        // World declaration
        let world_name = if graph.shards.len() == 1 {
            "logicodex-world".to_string()
        } else {
            format!("logicodex-world-{}-shards", graph.shards.len())
        };
        lines.push(format!("world {} {{", world_name));

        // Import section
        lines.push("  // === IMPORTS ===".to_string());
        for (_pkg, caps) in &interfaces {
            for cap in caps {
                if let Some(wit) = &cap.wit_mapping {
                    // Extract interface name from WIT mapping
                    let iface = wit.split('/').nth(1).unwrap_or("unknown");
                    lines.push(format!(
                        "  import {}: func() → result<string, error>; // {}",
                        iface, cap.canonical()
                    ));
                }
            }
        }

        // HW gates need special host reactor imports
        if !self.hw_gates.is_empty() {
            lines.push("  // Host Reactor imports (HW gates — host-mediated)".to_string());
            for cap in &self.hw_gates {
                lines.push(format!(
                    "  import host-reactor-{}: func(pin: u32, mode: string) → result<u32, error>; // {}",
                    cap.operation.to_lowercase(),
                    cap.canonical()
                ));
            }
        }

        lines.push("}}".to_string());
        lines.push("".to_string());

        // Interface definitions
        lines.push("// === INTERFACE DEFINITIONS ===".to_string());
        for (pkg, caps) in &interfaces {
            let iface_name = pkg.replace(':', "-").replace('/', "-");
            lines.push(format!("interface {} {{", iface_name));

            // Get unique operations for this interface
            let mut seen_ops: std::collections::HashSet<String> = std::collections::HashSet::new();
            for cap in caps {
                let domain = &cap.domain;
                let op_key = format!("{}.{}", domain, cap.operation);
                if seen_ops.insert(op_key.clone()) {
                    // Find the WIT operation details
                    let ops = get_wit_operations(domain);
                    if let Some(wit_op) = ops.iter().find(|o| o.logicodex_op == cap.operation) {
                        lines.push(wit_op.wit_signature());
                    } else {
                        // Fallback
                        lines.push(format!(
                            "  {}: func() → result<string, error>; // auto-generated",
                            cap.operation.to_lowercase()
                        ));
                    }
                }
            }
            lines.push("}".to_string());
            lines.push("".to_string());
        }

        lines.join("\n")
    }

    /// Generate host reactor stub code (Rust) for HW gates.
    pub fn generate_host_reactor_stub(&self) -> String {
        let mut lines = Vec::new();
        lines.push("// Host Reactor Stub — Auto-generated by CTL Mapper v1.36.0-alpha".to_string());
        lines.push("// These functions run on the HOST side, not in WASM guest.".to_string());
        lines.push("".to_string());
        lines.push("use logicodex_reactor::HostReactor;".to_string());
        lines.push("".to_string());

        for cap in &self.hw_gates {
            let fn_name = format!(
                "host_reactor_{}_{}",
                cap.domain.to_lowercase(),
                cap.operation.to_lowercase()
            );
            lines.push(format!(
                "/// Host-side implementation for {}.{}",
                cap.domain, cap.operation
            ));
            lines.push(format!(
                "pub fn {}(reactor: &mut HostReactor, pin: u32, mode: String) -> Result<u32, ReactorError> {{",
                fn_name
            ));
            lines.push(format!(
                "    // TODO: Implement {}.{} host-side logic",
                cap.domain, cap.operation
            ));
            lines.push("    reactor.with_hardware_zone(|hw| {".to_string());
            lines.push(format!(
                "        hw.{}(pin, &mode)",
                cap.operation.to_lowercase()
            ));
            lines.push("    })".to_string());
            lines.push("}".to_string());
            lines.push("".to_string());
        }

        lines.join("\n")
    }

    /// Get mapping statistics.
    pub fn stats(&self) -> CtlMappingStats {
        CtlMappingStats {
            mappings_applied: self.mappings_applied,
            hw_gates_detected: self.hw_gates.len(),
            unknown_domains: self.unknown_domains.clone(),
            overrides_used: self.overrides.len(),
        }
    }

    /// Check if any HW gates were detected.
    pub fn has_hw_gates(&self) -> bool {
        !self.hw_gates.is_empty()
    }

    /// Get list of HW gates (for host reactor setup).
    pub fn hw_gates(&self) -> &[CapabilityRef] {
        &self.hw_gates
    }
}

impl Default for CtlMapper {
    fn default() -> Self {
        Self::new()
    }
}

// ─── CtlMappingStats ───
/// Statistics about a CTL mapping operation.
#[derive(Debug, Clone)]
pub struct CtlMappingStats {
    pub mappings_applied: usize,
    pub hw_gates_detected: usize,
    pub unknown_domains: Vec<String>,
    pub overrides_used: usize,
}

impl CtlMappingStats {
    /// Check if mapping completed successfully.
    pub fn is_ok(&self) -> bool {
        self.unknown_domains.is_empty()
    }

    /// Format summary.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("CTL Mapping Summary:"));
        lines.push(format!("  Mappings applied: {}", self.mappings_applied));
        lines.push(format!("  HW gates (host reactor): {}", self.hw_gates_detected));
        lines.push(format!("  Overrides used: {}", self.overrides_used));
        if !self.unknown_domains.is_empty() {
            lines.push(format!(
                "  ⚠️  Unknown domains: {}",
                self.unknown_domains.join(", ")
            ));
        }
        lines.join("\n")
    }
}

// ─── Convenience: full pipeline ───
/// One-shot: take a CapabilityGraph, map it, and generate WIT.
pub fn map_and_generate_wit(graph: &mut CapabilityGraph) -> (String, CtlMappingStats) {
    let mut mapper = CtlMapper::new();
    mapper.map_graph(graph);
    let wit = mapper.generate_wit(graph);
    let stats = mapper.stats();
    (wit, stats)
}

/// One-shot with manual overrides.
pub fn map_and_generate_wit_with_overrides(
    graph: &mut CapabilityGraph,
    overrides: std::collections::HashMap<String, String>,
) -> (String, CtlMappingStats) {
    let mut mapper = CtlMapper::new();
    for (k, v) in overrides {
        mapper.add_override(k, v);
    }
    mapper.map_graph(graph);
    let wit = mapper.generate_wit(graph);
    let stats = mapper.stats();
    (wit, stats)
}
