// =========================================================================
// Tier 2: Capability Topology IR
//
// The Capability Topology is a compile-time graph that maps:
//   WHO (which symbol) → CAN DO WHAT (which gate)
//
// Built during Pass 1 (Pre-declaration) from GateContract annotations.
// Verified during Pass 2 (Deep Streaming) — any unresolved requirement
// triggers CapabilityContractViolation.
//
// This enables "Zero Runtime Mediation":
//   If it compiled, it's proven safe. No runtime checks needed.
// =========================================================================

use super::gate::{GateContract, GateRef};
use super::metadata::MetadataGraph;
use std::collections::HashMap;

/// Capability Topology — graph of all gate providers and consumers.
#[derive(Debug, Default)]
pub struct CapabilityTopology {
    /// Gate canonical name → Vec<symbol_id> yang PROVIDE gate ini
    providers: HashMap<String, Vec<u32>>,
    /// Gate canonical name → Vec<symbol_id> yang REQUIRE gate ini
    consumers: HashMap<String, Vec<u32>>,
    /// Semua kontrak gate yang didaftarkan
    contracts: Vec<GateContract>,
    /// Nama modul → symbol_id mapping untuk modul-modul
    module_symbols: HashMap<String, u32>,
}

impl CapabilityTopology {
    /// Get all gate contracts (provider → consumer relationships).
    pub fn contracts(&self) -> &[GateContract] {
        &self.contracts
    }

    /// Get providers for a given gate canonical name.
    pub fn providers_of(&self, gate: &str) -> Option<&[u32]> {
        self.providers.get(gate).map(|v| v.as_slice())
    }

    /// Get consumers for a given gate canonical name.
    pub fn consumers_of(&self, gate: &str) -> Option<&[u32]> {
        self.consumers.get(gate).map(|v| v.as_slice())
    }

    /// Iterate all registered gates with their provider symbol IDs.
    pub fn all_providers(&self) -> &HashMap<String, Vec<u32>> {
        &self.providers
    }

    /// Iterate all registered gates with their consumer symbol IDs.
    pub fn all_consumers(&self) -> &HashMap<String, Vec<u32>> {
        &self.consumers
    }

    /// Get module symbol ID by module name.
    pub fn module_symbol(&self, name: &str) -> Option<u32> {
        self.module_symbols.get(name).copied()
    }

    /// All registered module names.
    pub fn module_names(&self) -> impl Iterator<Item = &str> {
        self.module_symbols.keys().map(|s| s.as_str())
    }
}

/// Hasil verifikasi topology.
#[derive(Debug, Clone)]
pub struct TopologyVerifyResult {
    pub valid: bool,
    pub violations: Vec<TopologyViolation>,
    pub gates_provided: usize,
    pub gates_required: usize,
    pub gates_satisfied: usize,
    pub gates_unsatisfied: usize,
}

/// Satu pelanggaran topology — gate yang diperlukan tapi takde provider.
#[derive(Debug, Clone)]
pub struct TopologyViolation {
    pub symbol_id: u32,
    pub symbol_name: String,
    pub missing_gate: GateRef,
    /// Adakah ada modul lain yang provide gate dalam domain sama tapi op berbeza?
    pub similar_available: Vec<String>,
}

impl CapabilityTopology {
    /// Topology kosong.
    pub fn new() -> Self {
        Self::default()
    }

    /// Daftarkan satu GateContract ke dalam topology.
    pub fn register_contract(&mut self, symbol_id: u32, contract: GateContract) {
        // Register provides
        for gate in &contract.provides {
            let key = gate.canonical();
            self.providers.entry(key).or_default().push(symbol_id);
        }
        // Register requires
        for gate in &contract.requires {
            let key = gate.canonical();
            self.consumers.entry(key).or_default().push(symbol_id);
        }
        // Track module → symbol mapping
        self.module_symbols
            .insert(contract.module_name.clone(), symbol_id);
        self.contracts.push(contract);
    }

    /// Daftarkan gate provide tunggal (shortcut).
    pub fn add_provide(&mut self, symbol_id: u32, gate: GateRef) {
        let key = gate.canonical();
        self.providers.entry(key).or_default().push(symbol_id);
    }

    /// Daftarkan gate require tunggal (shortcut).
    pub fn add_require(&mut self, symbol_id: u32, gate: GateRef) {
        let key = gate.canonical();
        self.consumers.entry(key).or_default().push(symbol_id);
    }

    /// Verifikasi topology: setiap REQUIRE mesti ada PROVIDE.
    /// Ini berjalan semasa Pass 2 — selepas semua kontrak didaftarkan.
    pub fn verify(&self, _graph: &MetadataGraph) -> TopologyVerifyResult {
        let mut violations = Vec::new();
        let mut gates_satisfied = 0;
        let mut gates_unsatisfied = 0;

        // Untuk setiap gate yang diperlukan (consumers)
        for (gate_key, consumer_ids) in &self.consumers {
            let has_provider = self.providers.contains_key(gate_key);

            if has_provider {
                gates_satisfied += consumer_ids.len();
            } else {
                gates_unsatisfied += consumer_ids.len();

                // Cari GateRef asal untuk violation message
                for symbol_id in consumer_ids {
                    if let Some(contract) = self.find_contract_for_symbol(*symbol_id) {
                        if let Some(gate) = contract
                            .requires
                            .iter()
                            .find(|g| &g.canonical() == gate_key)
                        {
                            // Cari gate serupa yang available (same domain, different op)
                            let similar = self.find_similar_gates(gate);
                            let symbol_name = contract.module_name.clone();

                            violations.push(TopologyViolation {
                                symbol_id: *symbol_id,
                                symbol_name,
                                missing_gate: gate.clone(),
                                similar_available: similar,
                            });
                        }
                    }
                }
            }
        }

        let gates_provided: usize = self.providers.values().map(|v| v.len()).sum();
        let gates_required: usize = self.consumers.values().map(|v| v.len()).sum();

        TopologyVerifyResult {
            valid: violations.is_empty(),
            violations,
            gates_provided,
            gates_required,
            gates_satisfied,
            gates_unsatisfied,
        }
    }

    /// Serialkan topology ke format .cap (Capability Graph).
    /// Output: Vec<baris> yang boleh ditulis ke fail .cap
    pub fn serialize(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push("# Logicodex Capability Topology Graph v1.0".to_string());
        lines.push("# Gate = SIAPA boleh bercakap".to_string());
        lines.push("# Door = BAGAIMANA mereka bercakap".to_string());
        lines.push("".to_string());

        // Section: Providers
        lines.push("## PROVIDERS".to_string());
        let mut provider_keys: Vec<_> = self.providers.keys().collect();
        provider_keys.sort();
        for key in provider_keys {
            let symbol_ids = &self.providers[key];
            let names: Vec<String> = symbol_ids
                .iter()
                .filter_map(|id| self.find_contract_for_symbol(*id))
                .map(|c| c.module_name.clone())
                .collect();
            lines.push(format!("{} <- {}", key, names.join(", ")));
        }
        lines.push("".to_string());

        // Section: Consumers
        lines.push("## CONSUMERS".to_string());
        let mut consumer_keys: Vec<_> = self.consumers.keys().collect();
        consumer_keys.sort();
        for key in consumer_keys {
            let symbol_ids = &self.consumers[key];
            let names: Vec<String> = symbol_ids
                .iter()
                .filter_map(|id| self.find_contract_for_symbol(*id))
                .map(|c| c.module_name.clone())
                .collect();
            lines.push(format!("{} -> {}", key, names.join(", ")));
        }
        lines.push("".to_string());

        // Section: Contracts
        lines.push("## CONTRACTS".to_string());
        for contract in &self.contracts {
            lines.push(format!("MODULE {}", contract.module_name));
            for gate in &contract.provides {
                lines.push(format!("  PROVIDE {}.{}", gate.domain, gate.operation));
            }
            for gate in &contract.requires {
                lines.push(format!("  REQUIRE {}.{}", gate.domain, gate.operation));
            }
        }

        lines
    }

    /// Total gate dalam topology.
    pub fn gate_count(&self) -> usize {
        let mut all: std::collections::HashSet<String> = self.providers.keys().cloned().collect();
        all.extend(self.consumers.keys().cloned());
        all.len()
    }

    /// Jumlah kontrak.
    pub fn contract_count(&self) -> usize {
        self.contracts.len()
    }

    // ─── Helper Methods ───

    fn find_contract_for_symbol(&self, symbol_id: u32) -> Option<&GateContract> {
        // Cari contract yang module_symbols mapping ke symbol_id
        let module_name = self
            .module_symbols
            .iter()
            .find(|(_, &id)| id == symbol_id)
            .map(|(name, _)| name.as_str())?;
        self.contracts.iter().find(|c| c.module_name == module_name)
    }

    /// Cari gate yang available dalam domain sama tapi operation berbeza.
    fn find_similar_gates(&self, gate: &GateRef) -> Vec<String> {
        let mut similar = Vec::new();
        for key in self.providers.keys() {
            // Parse key untuk dapatkan domain
            if let Some(dot_pos) = key.find('.') {
                let domain = &key[..dot_pos];
                if domain == gate.domain && key != &gate.canonical() {
                    similar.push(key.clone());
                }
            }
        }
        similar.sort();
        similar
    }
}

impl TopologyVerifyResult {
    /// Format violation sebagai string yang readable.
    pub fn format_violations(&self) -> Vec<String> {
        self.violations
            .iter()
            .map(|v| {
                let mut msg = format!(
                    "[VIOLATION] '{}' memerlukan Gate '{}' tapi tiada modul yang menyediakannya",
                    v.symbol_name,
                    v.missing_gate.full()
                );
                if !v.similar_available.is_empty() {
                    msg.push_str(&format!(
                        "\n  Gate berkaitan yang tersedia: {}",
                        v.similar_available.join(", ")
                    ));
                }
                msg
            })
            .collect()
    }
}

// ─── Capability Diff ───
/// Banding dua topology dan detect privilege escalation.
pub fn diff_topology(
    old_topology: &CapabilityTopology,
    new_topology: &CapabilityTopology,
) -> CapabilityDiff {
    let mut added_gates = Vec::new();
    let mut removed_gates = Vec::new();
    let mut escalation_detected = Vec::new();

    // Find all unique gate keys in both topologies
    let mut all_keys: std::collections::HashSet<String> =
        old_topology.providers.keys().cloned().collect();
    all_keys.extend(new_topology.providers.keys().cloned());
    all_keys.extend(old_topology.consumers.keys().cloned());
    all_keys.extend(new_topology.consumers.keys().cloned());

    for key in &all_keys {
        let _old_provided = old_topology.providers.contains_key(key);
        let _new_provided = new_topology.providers.contains_key(key);
        let old_consumed = old_topology.consumers.contains_key(key);
        let new_consumed = new_topology.consumers.contains_key(key);

        if !old_consumed && new_consumed {
            // A module now REQUIRES a gate it didn't before
            if let Some(symbol_ids) = new_topology.consumers.get(key) {
                for sid in symbol_ids {
                    if let Some(contract) = new_topology.find_contract_for_symbol(*sid) {
                        added_gates.push(format!("{} -> {}", contract.module_name, key));
                    }
                }
            }
        }

        if old_consumed && !new_consumed {
            // A module no longer requires a gate
            if let Some(symbol_ids) = old_topology.consumers.get(key) {
                for sid in symbol_ids {
                    if let Some(contract) = old_topology.find_contract_for_symbol(*sid) {
                        removed_gates.push(format!("{} -/-> {}", contract.module_name, key));
                    }
                }
            }
        }

        // Privilege escalation: a module now requires a MORE PRIVILEGED gate
        // in the same domain (e.g., Net.Send → Net.Raw)
        if new_consumed && !old_consumed {
            // This is a new requirement — check if it's a sensitive domain
            if key.starts_with("Net.Raw")
                || key.starts_with("HW.")
                || key.starts_with("Storage.Delete")
            {
                if let Some(symbol_ids) = new_topology.consumers.get(key) {
                    for sid in symbol_ids {
                        if let Some(contract) = new_topology.find_contract_for_symbol(*sid) {
                            escalation_detected.push(format!(
                                "PRIVILEGE ESCALATION: '{}' kini memerlukan '{}' — akses sensitif dikesan!",
                                contract.module_name, key
                            ));
                        }
                    }
                }
            }
        }
    }

    CapabilityDiff {
        added_gates,
        removed_gates,
        privilege_escalation: escalation_detected,
    }
}

/// Hasil perbezaan antara dua topology.
#[derive(Debug, Clone)]
pub struct CapabilityDiff {
    pub added_gates: Vec<String>,
    pub removed_gates: Vec<String>,
    /// Privilege escalation detected — supply chain attack indicator
    pub privilege_escalation: Vec<String>,
}

impl CapabilityDiff {
    /// Adakah ada perubahan?
    pub fn has_changes(&self) -> bool {
        !self.added_gates.is_empty() || !self.removed_gates.is_empty()
    }

    /// Adakah privilege escalation dikesan?
    pub fn has_escalation(&self) -> bool {
        !self.privilege_escalation.is_empty()
    }

    /// Format summary.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Gate baharu: {}", self.added_gates.len()));
        lines.push(format!("Gate dipadam: {}", self.removed_gates.len()));
        if self.has_escalation() {
            lines.push(format!(
                "⚠️  PRIVILEGE ESCALATION dikesan: {}",
                self.privilege_escalation.len()
            ));
        }
        lines.join("\n")
    }
}
