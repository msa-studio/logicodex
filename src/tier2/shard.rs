// =========================================================================
// Logicodex v1.34.0-alpha — Tier 2: Service Topology Sharding
//
// "The Sharded Deterministic Reactor Manifesto"
//
// Extends CapabilityTopology (v1.32) dengan Service Graph + Shard Mapping.
// Ini adalah fondasi — semua reactor sharding bergantung pada topology ini.
//
// Prinsip:
//   1. Shard Isolation: Setiap CPU Core = satu ReactorInstance + LocalPool
//   2. Affinity-Pinned: Kompiler lakukan static mapping service → core
//   3. Deterministic Budgeting: Setiap shard ada memory quota
//   4. Cross-Shard = Door Only: SPSC Message Passing — forbidden by default
//
// Static Manifest (contoh):
//   shard WebShard { core: 0, services: [WebServer, ApiGateway], budget_mb: 256 }
//   shard MetricsShard { core: 1, services: [MetricsCollector], budget_mb: 64 }
//   door WebShard → MetricsShard { message_type: MetricPayload, capacity: 1024 }
// =========================================================================

use super::gate::GateRef;

use std::collections::HashMap;

// ─── ShardAssignment ───
/// Satu shard = satu CPU core + satu set servis + satu memory budget.
#[derive(Debug, Clone)]
pub struct ShardAssignment {
    /// Shard ID (0, 1, 2, ...)
    pub shard_id: u32,
    /// CPU core yang dipin (static mapping oleh kompiler)
    pub core_id: u32,
    /// Nama-nama servis dalam shard ini
    pub services: Vec<String>,
    /// Memory budget dalam MB
    pub budget_mb: u32,
    /// Gate-gate yang dibolehkan dalam shard ini
    pub gates: Vec<GateRef>,
}

impl ShardAssignment {
    /// Cipta assignment baru.
    pub fn new(shard_id: u32, core_id: u32, budget_mb: u32) -> Self {
        Self {
            shard_id,
            core_id,
            services: Vec::new(),
            budget_mb,
            gates: Vec::new(),
        }
    }

    /// Tambah servis ke shard.
    pub fn add_service(&mut self, name: impl Into<String>) {
        self.services.push(name.into());
    }

    /// Tambah gate ke shard.
    pub fn add_gate(&mut self, gate: GateRef) {
        self.gates.push(gate);
    }

    /// Bilangan servis dalam shard.
    pub fn service_count(&self) -> usize {
        self.services.len()
    }

    /// Total estimated memory untuk servis-servis ini (rough estimate).
    pub fn estimated_memory_mb(&self) -> u32 {
        // Rough: setiap servis ~8MB base + connection buffer
        let base = self.services.len() as u32 * 8;
        base.min(self.budget_mb) // capped at budget
    }
}

// ─── ServiceNode ───
/// Satu node dalam Service Graph.
#[derive(Debug, Clone)]
pub struct ServiceNode {
    /// Nama servis
    pub name: String,
    /// Port TCP/UDP
    pub port: u16,
    /// Gate yang diperlukan
    pub requires: Vec<GateRef>,
    /// Handler function name
    pub handler: String,
    /// Policy backpressure
    pub policy: String,
    /// Shard ID tempat servis di-assign (ditentukan oleh kompiler)
    pub assigned_shard: Option<u32>,
}

impl ServiceNode {
    pub fn new(name: impl Into<String>, port: u16, handler: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            port,
            requires: Vec::new(),
            handler: handler.into(),
            policy: "Block".to_string(),
            assigned_shard: None,
        }
    }

    pub fn with_policy(mut self, policy: impl Into<String>) -> Self {
        self.policy = policy.into();
        self
    }

    pub fn with_gate(mut self, gate: GateRef) -> Self {
        self.requires.push(gate);
        self
    }

    pub fn assign_to(mut self, shard_id: u32) -> Self {
        self.assigned_shard = Some(shard_id);
        self
    }
}

// ─── CommEdge ───
/// Communication edge antara dua servis.
/// Cross-shard HANYA melalui Door (Message gate) — divalidasi oleh kompiler.
#[derive(Debug, Clone)]
pub struct CommEdge {
    /// Servis sumber
    pub from: String,
    /// Servis destinasi
    pub to: String,
    /// Jenis komunikasi: Door (Message) = valid, Direct = invalid cross-shard
    pub comm_type: CommType,
    /// Message type (untuk Door)
    pub message_type: String,
    /// Buffer capacity (untuk Door)
    pub capacity: usize,
    /// Adakah ini cross-shard?
    pub is_cross_shard: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommType {
    /// SPSC Door — valid untuk cross-shard
    Door,
    /// Direct call — HANYA valid dalam shard sama
    Direct,
}

impl CommEdge {
    /// Cipta Door edge (cross-shard safe).
    pub fn door(
        from: impl Into<String>,
        to: impl Into<String>,
        message_type: impl Into<String>,
        capacity: usize,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            comm_type: CommType::Door,
            message_type: message_type.into(),
            capacity,
            is_cross_shard: false, // ditentukan oleh topology builder
        }
    }

    /// Cipta Direct edge (intra-shard only).
    pub fn direct(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            comm_type: CommType::Direct,
            message_type: String::new(),
            capacity: 0,
            is_cross_shard: false,
        }
    }
}

// ─── ServiceGraph ───
/// Graph servis — nodes = servis, edges = komunikasi.
#[derive(Debug, Default)]
pub struct ServiceGraph {
    nodes: HashMap<String, ServiceNode>,
    edges: Vec<CommEdge>,
}

impl ServiceGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Tambah servis node.
    pub fn add_node(&mut self, node: ServiceNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    /// Tambah komunikasi edge.
    pub fn add_edge(&mut self, edge: CommEdge) {
        self.edges.push(edge);
    }

    /// Dapatkan node mengikut nama.
    pub fn get_node(&self, name: &str) -> Option<&ServiceNode> {
        self.nodes.get(name)
    }

    /// Semua edges yang cross-shard (Door only).
    pub fn cross_shard_edges(&self) -> Vec<&CommEdge> {
        self.edges
            .iter()
            .filter(|e| e.is_cross_shard && e.comm_type == CommType::Door)
            .collect()
    }

    /// Bilangan servis.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Bilangan edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

// ─── ShardTopology ───
/// Topology lengkap: assignments + service graph + verifikasi.
#[derive(Debug)]
pub struct ShardTopology {
    /// Semua shard assignments
    pub assignments: Vec<ShardAssignment>,
    /// Service graph (nodes + edges)
    pub service_graph: ServiceGraph,
    /// Cross-shard Door connections
    pub doors: Vec<DoorRef>,
}

/// Rujukan ke satu Door (cross-shard SPSC channel).
#[derive(Debug, Clone)]
pub struct DoorRef {
    pub from_shard: u32,
    pub to_shard: u32,
    pub message_type: String,
    pub capacity: usize,
}

impl DoorRef {
    pub fn new(from: u32, to: u32, message_type: impl Into<String>, capacity: usize) -> Self {
        Self {
            from_shard: from,
            to_shard: to,
            message_type: message_type.into(),
            capacity,
        }
    }
}

/// Hasil verifikasi topology.
#[derive(Debug, Clone)]
pub struct ShardVerifyResult {
    pub valid: bool,
    pub violations: Vec<ShardViolation>,
    pub total_services: usize,
    pub total_shards: usize,
    pub cross_shard_doors: usize,
    pub total_budget_mb: u32,
    pub estimated_usage_mb: u32,
}

/// Satu pelanggaran dalam topology.
#[derive(Debug, Clone)]
pub enum ShardViolation {
    /// Servis tidak di-assign ke mana-mana shard
    UnassignedService { service: String },
    /// Servis di-assign ke lebih dari satu shard
    DuplicateAssignment {
        service: String,
        shard_a: u32,
        shard_b: u32,
    },
    /// Cross-shard communication bukan melalui Door
    ForbiddenDirectCrossShard { from: String, to: String },
    /// Budget melebihi had
    BudgetOverflow {
        shard_id: u32,
        budget: u32,
        usage: u32,
    },
    /// Shard tanpa servis
    EmptyShard { shard_id: u32 },
    /// Core ID conflict (dua shard pin ke core sama)
    CoreConflict {
        shard_a: u32,
        shard_b: u32,
        core_id: u32,
    },
}

impl ShardTopology {
    /// Topology kosong.
    pub fn new() -> Self {
        Self {
            assignments: Vec::new(),
            service_graph: ServiceGraph::new(),
            doors: Vec::new(),
        }
    }

    /// Tambah shard assignment.
    pub fn add_assignment(&mut self, assignment: ShardAssignment) {
        self.assignments.push(assignment);
    }

    /// Tambah Door (cross-shard communication).
    pub fn add_door(&mut self, door: DoorRef) {
        self.doors.push(door);
    }

    /// Verifikasi topology — compile-time safety check.
    /// Semua check ini dilakukan sebelum codegen.
    pub fn verify(&self) -> ShardVerifyResult {
        let mut violations = Vec::new();

        // 1. Check setiap servis di-assign ke tepat satu shard
        let mut service_to_shard: HashMap<String, u32> = HashMap::new();
        for assignment in &self.assignments {
            for svc in &assignment.services {
                if let Some(existing) = service_to_shard.get(svc) {
                    violations.push(ShardViolation::DuplicateAssignment {
                        service: svc.clone(),
                        shard_a: *existing,
                        shard_b: assignment.shard_id,
                    });
                } else {
                    service_to_shard.insert(svc.clone(), assignment.shard_id);
                }
            }
        }

        // Check servis dalam graph yang tak di-assign
        for (name, node) in &self.service_graph.nodes {
            if !service_to_shard.contains_key(name) && node.assigned_shard.is_none() {
                violations.push(ShardViolation::UnassignedService {
                    service: name.clone(),
                });
            }
        }

        // 2. Check cross-shard edges — HANYA Door dibenarkan
        for edge in &self.service_graph.edges {
            if edge.is_cross_shard && edge.comm_type != CommType::Door {
                violations.push(ShardViolation::ForbiddenDirectCrossShard {
                    from: edge.from.clone(),
                    to: edge.to.clone(),
                });
            }
        }

        // 3. Check budget tidak melebihi
        let mut total_budget = 0;
        let mut total_usage = 0;
        for assignment in &self.assignments {
            total_budget += assignment.budget_mb;
            let usage = assignment.estimated_memory_mb();
            total_usage += usage;
            if usage > assignment.budget_mb {
                violations.push(ShardViolation::BudgetOverflow {
                    shard_id: assignment.shard_id,
                    budget: assignment.budget_mb,
                    usage,
                });
            }
        }

        // 4. Check empty shards
        for assignment in &self.assignments {
            if assignment.services.is_empty() {
                violations.push(ShardViolation::EmptyShard {
                    shard_id: assignment.shard_id,
                });
            }
        }

        // 5. Check core conflicts
        let mut core_map: HashMap<u32, u32> = HashMap::new(); // core_id → shard_id
        for assignment in &self.assignments {
            if let Some(existing_shard) = core_map.get(&assignment.core_id) {
                if *existing_shard != assignment.shard_id {
                    violations.push(ShardViolation::CoreConflict {
                        shard_a: *existing_shard,
                        shard_b: assignment.shard_id,
                        core_id: assignment.core_id,
                    });
                }
            } else {
                core_map.insert(assignment.core_id, assignment.shard_id);
            }
        }

        ShardVerifyResult {
            valid: violations.is_empty(),
            violations,
            total_services: self.service_graph.node_count(),
            total_shards: self.assignments.len(),
            cross_shard_doors: self.doors.len(),
            total_budget_mb: total_budget,
            estimated_usage_mb: total_usage,
        }
    }

    /// Serialize ke JSON (untuk visualisasi / audit).
    pub fn to_manifest_json(&self) -> String {
        let mut lines = Vec::new();
        lines.push("{".to_string());

        // Shards
        lines.push("  \"shards\": [".to_string());
        for (i, a) in self.assignments.iter().enumerate() {
            let services = a
                .services
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ");
            let gates = a
                .gates
                .iter()
                .map(|g| format!("\"{}\"", g.canonical()))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!(
                "    {{ \"shard_id\": {}, \"core_id\": {}, \"budget_mb\": {}, \"services\": [{}], \"gates\": [{}] }}{}",
                a.shard_id, a.core_id, a.budget_mb, services, gates,
                if i < self.assignments.len() - 1 { "," } else { "" }
            ));
        }
        lines.push("  ],".to_string());

        // Doors
        lines.push("  \"doors\": [".to_string());
        for (i, d) in self.doors.iter().enumerate() {
            lines.push(format!(
                "    {{ \"from_shard\": {}, \"to_shard\": {}, \"message_type\": \"{}\", \"capacity\": {} }}{}",
                d.from_shard, d.to_shard, d.message_type, d.capacity,
                if i < self.doors.len() - 1 { "," } else { "" }
            ));
        }
        lines.push("  ],".to_string());

        // Stats
        let result = self.verify();
        lines.push(format!("  \"stats\": {{"));
        lines.push(format!(
            "    \"total_services\": {},",
            result.total_services
        ));
        lines.push(format!("    \"total_shards\": {},", result.total_shards));
        lines.push(format!(
            "    \"cross_shard_doors\": {},",
            result.cross_shard_doors
        ));
        lines.push(format!(
            "    \"total_budget_mb\": {},",
            result.total_budget_mb
        ));
        lines.push(format!(
            "    \"estimated_usage_mb\": {},",
            result.estimated_usage_mb
        ));
        lines.push(format!("    \"valid\": {}", result.valid));
        lines.push("  }".to_string());
        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Dapatkan assignment mengikut shard_id.
    pub fn get_shard(&self, shard_id: u32) -> Option<&ShardAssignment> {
        self.assignments.iter().find(|a| a.shard_id == shard_id)
    }

    /// Total budget semua shards.
    pub fn total_budget_mb(&self) -> u32 {
        self.assignments.iter().map(|a| a.budget_mb).sum()
    }
}
