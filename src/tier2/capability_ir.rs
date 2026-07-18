// =========================================================================
// Logicodex v1.35.0-alpha — Capability Translation Layer: CapabilityGraph IR
//
// "Single Source of Truth" — language-agnostic capability representation.
// Project capability model INTO the WASM ecosystem, not borrow from it.
//
// CapabilityGraph unifies:
//   - v1.31 SemanticSummary (effects, inline_cost)
//   - v1.32 CapabilityTopology (gates, providers/consumers)
//   - v1.34 ShardTopology (shards, services, doors, budgets)
//
// Generates:
//   1. Native (ELF) — capability checks inlined by codegen
//   2. WASM (.wasm) — maps to WASI via CTL Mapper (Fasa B)
//   3. Audit (.cap) — human-readable capability manifest
//
// Guard Rail:
//   - WASM Guest = Unit Logik — NO direct hardware access
//   - All hardware access through Capability Gates → Host Reactor
//   - NO hidden scheduling — all async = explicit Reactor Events
// =========================================================================

use super::gate::{GateRef, GateType};
use super::metadata::{Capability, InlineCost, SemanticSummary};
use super::shard::ShardTopology;
use super::topology::CapabilityTopology;
use std::collections::HashMap;

// ─── CompileTarget ───
/// Output target untuk compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileTarget {
    /// ELF sahaja — native, capability checks inlined
    Native,
    /// WASM sahaja — sandboxed, maps ke WASI via CTL
    Wasm,
    /// Dual artifacts — ELF + WASM dari Capability Graph yang sama
    All,
}

impl CompileTarget {
    /// Parse daripada string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "native" | "elf" | "ELF" => Some(CompileTarget::Native),
            "wasm" | "WASM" | "webassembly" => Some(CompileTarget::Wasm),
            "all" | "dual" => Some(CompileTarget::All),
            _ => None,
        }
    }

    /// Format sebagai string.
    pub fn as_str(&self) -> &'static str {
        match self {
            CompileTarget::Native => "native",
            CompileTarget::Wasm => "wasm",
            CompileTarget::All => "all",
        }
    }
}

// ─── CapabilityRef ───
/// Rujukan keupayaan dengan optional WIT mapping (Fasa B).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityRef {
    /// Domain: Storage, Net, UI, HW, Audio, Crypto, DB
    pub domain: String,
    /// Operation: Baca, Send, GPIO, etc.
    pub operation: String,
    /// Jenis gate
    pub gate_type: GateType,
    /// WIT mapping — diisi oleh CTL Mapper (Fasa B)
    /// Contoh: "wasi:filesystem/read" untuk Storage.Baca
    pub wit_mapping: Option<String>,
}

impl CapabilityRef {
    pub fn new(
        domain: impl Into<String>,
        operation: impl Into<String>,
        gate_type: GateType,
    ) -> Self {
        Self {
            domain: domain.into(),
            operation: operation.into(),
            gate_type,
            wit_mapping: None,
        }
    }

    /// Format canonikal: "Domain.Operation"
    pub fn canonical(&self) -> String {
        format!("{}.{}", self.domain, self.operation)
    }

    /// Set WIT mapping (digunakan oleh CTL Mapper Fasa B).
    pub fn with_wit(mut self, wit: impl Into<String>) -> Self {
        self.wit_mapping = Some(wit.into());
        self
    }

    /// Check jika ini hardware gate (tidak dibenarkan dalam WASM).
    pub fn is_hardware(&self) -> bool {
        matches!(self.gate_type, GateType::Hardware)
    }
}

impl From<&GateRef> for CapabilityRef {
    fn from(g: &GateRef) -> Self {
        Self {
            domain: g.domain.clone(),
            operation: g.operation.clone(),
            gate_type: g.gate_type.clone(),
            wit_mapping: None,
        }
    }
}

// ─── IRServiceNode ───
/// Servis node dalam CapabilityGraph IR.
/// Unified: merges SemanticSummary + ServiceNode + capability info.
#[derive(Debug, Clone)]
pub struct IRServiceNode {
    pub id: u32,
    pub name: String,
    pub port: Option<u16>,
    /// Capability gates yang diperlukan
    pub requires: Vec<CapabilityRef>,
    /// Capability gates yang disediakan
    pub provides: Vec<CapabilityRef>,
    /// Handler function/actor name
    pub handler: String,
    /// Shard ID yang di-assign (None = belum assign)
    pub assigned_shard: Option<u32>,
    /// Policy backpressure
    pub policy: String,
    /// Capability effects
    pub effects: Capability,
    /// Inline cost
    pub inline_cost: InlineCost,
    /// Channels used (untuk actor)
    pub channels_used: Vec<String>,
}

impl IRServiceNode {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            port: None,
            requires: Vec::new(),
            provides: Vec::new(),
            handler: String::new(),
            assigned_shard: None,
            policy: "Block".to_string(),
            effects: Capability::default(),
            inline_cost: InlineCost::Medium,
            channels_used: Vec::new(),
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn with_handler(mut self, handler: impl Into<String>) -> Self {
        self.handler = handler.into();
        self
    }

    pub fn with_shard(mut self, shard_id: u32) -> Self {
        self.assigned_shard = Some(shard_id);
        self
    }

    pub fn require(mut self, cap: CapabilityRef) -> Self {
        self.requires.push(cap);
        self
    }

    pub fn provide(mut self, cap: CapabilityRef) -> Self {
        self.provides.push(cap);
        self
    }
}

// ─── IRShardNode ───
/// Shard node dalam CapabilityGraph IR.
#[derive(Debug, Clone)]
pub struct IRShardNode {
    pub id: u32,
    pub core_id: u32,
    pub budget_mb: u32,
    /// Service IDs dalam shard ini
    pub services: Vec<u32>,
    /// Gates yang dibenarkan
    pub allowed_gates: Vec<CapabilityRef>,
}

impl IRShardNode {
    pub fn new(id: u32, core_id: u32, budget_mb: u32) -> Self {
        Self {
            id,
            core_id,
            budget_mb,
            services: Vec::new(),
            allowed_gates: Vec::new(),
        }
    }
}

// ─── IRDoorEdge ───
/// Door edge (cross-shard SPSC) dalam CapabilityGraph IR.
#[derive(Debug, Clone)]
pub struct IRDoorEdge {
    pub from_service: u32,
    pub to_service: u32,
    pub message_type: String,
    pub capacity: usize,
}

// ─── IRGateEdge ───
/// Gate edge (capability contract) dalam CapabilityGraph IR.
#[derive(Debug, Clone)]
pub struct IRGateEdge {
    pub from: u32,
    pub to: u32,
    pub capability: CapabilityRef,
}

// ─── CapabilityGraph ───
/// THE IR — Single Source of Truth untuk semua output targets.
#[derive(Debug)]
pub struct CapabilityGraph {
    /// Semua servis (indexed by ID)
    pub services: HashMap<u32, IRServiceNode>,
    /// Semua shards (indexed by ID)
    pub shards: HashMap<u32, IRShardNode>,
    /// Door edges (cross-shard communication)
    pub doors: Vec<IRDoorEdge>,
    /// Gate edges (capability contracts)
    pub gates: Vec<IRGateEdge>,
    /// Output target
    pub target: CompileTarget,
    /// Versi
    pub version: String,
}

impl CapabilityGraph {
    /// Graph kosong.
    pub fn new(target: CompileTarget) -> Self {
        Self {
            services: HashMap::new(),
            shards: HashMap::new(),
            doors: Vec::new(),
            gates: Vec::new(),
            target,
            version: "1.35.0-alpha".to_string(),
        }
    }

    /// Tambah servis.
    pub fn add_service(&mut self, node: IRServiceNode) {
        self.services.insert(node.id, node);
    }

    /// Tambah shard.
    pub fn add_shard(&mut self, node: IRShardNode) {
        self.shards.insert(node.id, node);
    }

    /// Tambah door edge.
    pub fn add_door(&mut self, edge: IRDoorEdge) {
        self.doors.push(edge);
    }

    /// Tambah gate edge.
    pub fn add_gate(&mut self, edge: IRGateEdge) {
        self.gates.push(edge);
    }

    // ─── Integration: Build from existing structures ───

    /// Build dari SemanticSummary (v1.31) — servis/functions.
    pub fn from_semantic_summaries(&mut self, summaries: &[SemanticSummary]) {
        for s in summaries {
            let node = IRServiceNode {
                id: s.symbol_id,
                name: s.name.clone(),
                port: None, // semantic summaries don't have ports
                requires: s.requires_gates.iter().map(CapabilityRef::from).collect(),
                provides: s.provides_gates.iter().map(CapabilityRef::from).collect(),
                handler: s.name.clone(),
                assigned_shard: None,
                policy: "Block".to_string(),
                effects: s.effects,
                inline_cost: s.inline_cost,
                channels_used: s.channels_used.clone(),
            };
            self.add_service(node);
        }
    }

    /// Build dari CapabilityTopology (v1.32) — gate contracts.
    /// v1.38: Fully implemented — imports all gate contracts as IRGateEdge.
    pub fn from_topology(&mut self, topology: &CapabilityTopology) {
        // Create service nodes for each contract's module
        for (idx, contract) in topology.contracts().iter().enumerate() {
            let module_id = idx as u32;
            // Only add if not already present
            if self.service_by_name(&contract.module_name).is_none() {
                let node = IRServiceNode::new(module_id, contract.module_name.clone());
                self.add_service(node);
            }
        }
        // Create gate edges from provides → requires relationships
        for (idx, contract) in topology.contracts().iter().enumerate() {
            let module_id = idx as u32;
            // For each provided gate, create an edge from this module
            for gate in &contract.provides {
                let cap_ref = CapabilityRef::from(gate);
                self.add_gate(IRGateEdge {
                    from: module_id,
                    to: 0, // 0 = system/generic consumer
                    capability: cap_ref,
                });
            }
            // For each required gate, create an edge to this module
            for gate in &contract.requires {
                let cap_ref = CapabilityRef::from(gate);
                self.add_gate(IRGateEdge {
                    from: 0, // 0 = system/generic provider
                    to: module_id,
                    capability: cap_ref,
                });
            }
        }
    }

    /// Build dari ShardTopology (v1.34) — shards + services + doors.
    pub fn from_shard_topology(&mut self, topology: &ShardTopology) {
        // Import shards
        for assignment in &topology.assignments {
            let shard = IRShardNode::new(
                assignment.shard_id,
                assignment.core_id,
                assignment.budget_mb,
            );
            self.add_shard(shard);
        }

        // Import doors
        for door in &topology.doors {
            // Find service IDs from shard services
            // (Simplified — in full impl, map shard → services)
            let edge = IRDoorEdge {
                from_service: door.from_shard, // mapped to service ID
                to_service: door.to_shard,     // mapped to service ID
                message_type: door.message_type.clone(),
                capacity: door.capacity,
            };
            self.add_door(edge);
        }
    }

    // ─── Unified Verification ───

    /// Verify unified CapabilityGraph.
    /// Combines all checks from v1.32 topology + v1.34 shard topology.
    pub fn verify(&self) -> IRVerifyResult {
        let mut violations = Vec::new();

        // 1. Setiap servis mesti ada dalam graph
        if self.services.is_empty() {
            violations.push(IRViolation::EmptyGraph);
        }

        // 2. WASM target: tiada hardware gate dalam servis
        if self.target == CompileTarget::Wasm || self.target == CompileTarget::All {
            for (_id, svc) in &self.services {
                for req in &svc.requires {
                    if req.is_hardware() {
                        violations.push(IRViolation::WasmHardwareGate {
                            service: svc.name.clone(),
                            gate: req.canonical(),
                        });
                    }
                }
            }
        }

        // 3. Setiap servis yang di-assign ke shard mesti shard wujud
        for (_id, svc) in &self.services {
            if let Some(shard_id) = svc.assigned_shard {
                if !self.shards.contains_key(&shard_id) {
                    violations.push(IRViolation::InvalidShardAssignment {
                        service: svc.name.clone(),
                        shard_id,
                    });
                }
            }
        }

        // 4. Setiap door mesti menghubungkan servis yang wujud
        for door in &self.doors {
            if !self.services.contains_key(&door.from_service) {
                violations.push(IRViolation::UnknownServiceInDoor {
                    service_id: door.from_service,
                });
            }
            if !self.services.contains_key(&door.to_service) {
                violations.push(IRViolation::UnknownServiceInDoor {
                    service_id: door.to_service,
                });
            }
        }

        // 5. Tiada gate edge ke servis yang tak wujud
        for gate in &self.gates {
            if !self.services.contains_key(&gate.from) {
                violations.push(IRViolation::UnknownServiceInGate {
                    service_id: gate.from,
                });
            }
            if !self.services.contains_key(&gate.to) {
                violations.push(IRViolation::UnknownServiceInGate {
                    service_id: gate.to,
                });
            }
        }

        // 6. Budget check: setiap shard mesti ada servis
        for (id, shard) in &self.shards {
            if shard.services.is_empty() {
                violations.push(IRViolation::EmptyShard { shard_id: *id });
            }
        }

        IRVerifyResult {
            valid: violations.is_empty(),
            violations,
            service_count: self.services.len(),
            shard_count: self.shards.len(),
            door_count: self.doors.len(),
            gate_count: self.gates.len(),
            target: self.target,
        }
    }

    // ─── Output Generation ───

    /// Generate .cap file (audit manifest) — Fasa A.
    pub fn to_cap(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!(
            "# Logicodex Capability Graph v{}",
            env!("CARGO_PKG_VERSION")
        ));
        lines.push(format!("# Target: {}", self.target.as_str()));
        lines.push("# Project capability model INTO the WASM ecosystem".to_string());
        lines.push(String::new());

        // Services
        lines.push("## SERVICES".to_string());
        let mut svc_ids: Vec<_> = self.services.keys().collect();
        svc_ids.sort();
        for id in svc_ids {
            let s = &self.services[id];
            lines.push(format!("service {} {{", s.name));
            lines.push(format!("  id: {},", s.id));
            if let Some(port) = s.port {
                lines.push(format!("  port: {},", port));
            }
            if let Some(shard) = s.assigned_shard {
                lines.push(format!("  shard: {},", shard));
            }
            if !s.handler.is_empty() {
                lines.push(format!("  handler: {},", s.handler));
            }
            for req in &s.requires {
                lines.push(format!(
                    "  requires: {}{},",
                    req.canonical(),
                    req.wit_mapping
                        .as_ref()
                        .map(|w| format!(" ({})", w))
                        .unwrap_or_default()
                ));
            }
            lines.push("}".to_string());
        }
        lines.push(String::new());

        // Shards
        lines.push("## SHARDS".to_string());
        let mut shard_ids: Vec<_> = self.shards.keys().collect();
        shard_ids.sort();
        for id in shard_ids {
            let sh = &self.shards[id];
            lines.push(format!("shard {} {{", sh.id));
            lines.push(format!("  core: {},", sh.core_id));
            lines.push(format!("  budget_mb: {},", sh.budget_mb));
            lines.push(format!(
                "  services: [{}],",
                sh.services
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            lines.push("}".to_string());
        }
        lines.push(String::new());

        // Doors
        lines.push("## DOORS".to_string());
        for door in &self.doors {
            lines.push(format!(
                "door {} → {} {{ type: {}, capacity: {} }}",
                door.from_service, door.to_service, door.message_type, door.capacity
            ));
        }
        lines.push(String::new());

        // Gates
        lines.push("## GATES".to_string());
        for gate in &self.gates {
            lines.push(format!(
                "gate {} → {}: {}",
                gate.from,
                gate.to,
                gate.capability.canonical()
            ));
        }

        lines
    }

    /// Generate WIT string — Fasa B foundation (stub).
    /// Dalam Fasa B penuh: CTL Mapper akan generate WIT auto dari CapabilityGraph.
    pub fn to_wit_stub(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "// WIT Auto-Generated dari CapabilityGraph (Logicodex v{})",
            env!("CARGO_PKG_VERSION")
        ));
        lines.push("// Fasa B: CTL Mapper akan generate ini secara auto".to_string());
        lines.push(String::new());

        // Worlds — satu per shard
        for (shard_id, shard) in &self.shards {
            lines.push(format!("world shard-{} {{", shard_id));
            lines.push("  // Imports — capabilities yang diperlukan".to_string());
            for cap in &shard.allowed_gates {
                let wit = cap
                    .wit_mapping
                    .as_ref()
                    .map(|w| w.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "{}:{}",
                            cap.domain.to_lowercase(),
                            cap.operation.to_lowercase()
                        )
                    });
                lines.push(format!("  import {};", wit));
            }
            lines.push("}}".to_string());
            lines.push(String::new());
        }

        // Interface untuk setiap domain
        let mut domains: HashMap<String, Vec<&CapabilityRef>> = HashMap::new();
        for (_, svc) in &self.services {
            for req in &svc.requires {
                domains.entry(req.domain.clone()).or_default().push(req);
            }
        }

        for (domain, caps) in &domains {
            lines.push(format!("interface {} {{", domain.to_lowercase()));
            for cap in caps {
                let wit = cap
                    .wit_mapping
                    .as_ref()
                    .map(|w| w.clone())
                    .unwrap_or_else(|| cap.operation.to_lowercase());
                lines.push(format!("  // {}", cap.canonical()));
                lines.push(format!("  {}: func() → result<string, error>;", wit));
            }
            lines.push("}".to_string());
            lines.push(String::new());
        }

        lines.join("\n")
    }

    /// Dapatkan servis mengikut nama.
    pub fn service_by_name(&self, name: &str) -> Option<&IRServiceNode> {
        self.services.values().find(|s| s.name == name)
    }

    /// Dapatkan shard mengikut ID.
    pub fn shard(&self, shard_id: u32) -> Option<&IRShardNode> {
        self.shards.get(&shard_id)
    }
}

// ─── IRVerifyResult ───
#[derive(Debug, Clone)]
pub struct IRVerifyResult {
    pub valid: bool,
    pub violations: Vec<IRViolation>,
    pub service_count: usize,
    pub shard_count: usize,
    pub door_count: usize,
    pub gate_count: usize,
    pub target: CompileTarget,
}

/// Pelanggaran dalam CapabilityGraph.
#[derive(Debug, Clone)]
pub enum IRViolation {
    EmptyGraph,
    /// WASM target tak boleh ada hardware gate
    WasmHardwareGate {
        service: String,
        gate: String,
    },
    /// Servis di-assign ke shard yang tak wujud
    InvalidShardAssignment {
        service: String,
        shard_id: u32,
    },
    /// Door menghubungkan servis tak dikenali
    UnknownServiceInDoor {
        service_id: u32,
    },
    UnknownServiceInGate {
        service_id: u32,
    },
    EmptyShard {
        shard_id: u32,
    },
}
