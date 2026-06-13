// =========================================================================
// Logicodex v1.32.0-alpha — Static Capability Fabric Tests
//
// Tests: GateRef, GateType, GateContract, CapabilityTopology,
//        TopologyVerifyResult, CapabilityDiff
// =========================================================================

use logicodex::tier2::{
    diff_topology, CapabilityTopology, GateContract, GateDomain, GateRef, GateType,
    TopologyVerifyResult, TopologyViolation,
};

// ─── 1. GateRef creation ───

#[test]
fn gate_ref_new() {
    let g = GateRef::new("Storage", "Baca", GateType::DirectCall);
    assert_eq!(g.domain, "Storage");
    assert_eq!(g.operation, "Baca");
    assert!(g.is_inlineable());
    assert!(!g.is_hardware());
}

#[test]
fn gate_ref_hardware() {
    let g = GateRef::new("HW", "GPIO", GateType::Hardware);
    assert!(g.is_hardware());
    assert!(!g.is_inlineable());
}

#[test]
fn gate_ref_message() {
    let g = GateRef::new("Net", "Send", GateType::Message);
    assert!(!g.is_inlineable());
    assert!(!g.is_hardware());
}

// ─── 2. GateRef parse ───

#[test]
fn gate_ref_parse_simple() {
    let g = GateRef::parse("Storage.Baca").unwrap();
    assert_eq!(g.domain, "Storage");
    assert_eq!(g.operation, "Baca");
    assert_eq!(g.gate_type, GateType::DirectCall); // default
}

#[test]
fn gate_ref_parse_with_type() {
    let g = GateRef::parse("Net.Send:Message").unwrap();
    assert_eq!(g.domain, "Net");
    assert_eq!(g.operation, "Send");
    assert_eq!(g.gate_type, GateType::Message);
}

#[test]
fn gate_ref_parse_hardware() {
    let g = GateRef::parse("HW.GPIO:Hardware").unwrap();
    assert_eq!(g.gate_type, GateType::Hardware);
}

#[test]
fn gate_ref_parse_invalid_format() {
    let result = GateRef::parse("NoDotHere");
    assert!(result.is_err());
}

#[test]
fn gate_ref_canonical() {
    let g = GateRef::new("Storage", "Baca", GateType::DirectCall);
    assert_eq!(g.canonical(), "Storage.Baca");
}

// ─── 3. GateDomain helpers ───

#[test]
fn gate_domain_storage() {
    let g = GateDomain::storage_read();
    assert_eq!(g.domain, "Storage");
    assert_eq!(g.operation, "Baca");
}

#[test]
fn gate_domain_net() {
    let g = GateDomain::net_send();
    assert_eq!(g.domain, "Net");
    assert_eq!(g.operation, "Send");
    assert_eq!(g.gate_type, GateType::Message);
}

#[test]
fn gate_domain_hardware() {
    let g = GateDomain::hw_gpio();
    assert_eq!(g.gate_type, GateType::Hardware);
}

// ─── 4. GateContract ───

#[test]
fn gate_contract_new() {
    let c = GateContract::new("MyModule");
    assert_eq!(c.module_name, "MyModule");
    assert!(c.provides.is_empty());
    assert!(c.requires.is_empty());
}

#[test]
fn gate_contract_provide() {
    let mut c = GateContract::new("Driver");
    c.provide(GateDomain::storage_read());
    assert!(c.provides_gate(&GateDomain::storage_read()));
    assert!(!c.provides_gate(&GateDomain::storage_write()));
}

#[test]
fn gate_contract_require() {
    let mut c = GateContract::new("App");
    c.require(GateDomain::storage_read());
    assert!(c.requires_gate(&GateDomain::storage_read()));
}

// ─── 5. CapabilityTopology: register and verify valid ───

#[test]
fn topology_valid_when_require_has_provider() {
    let mut topo = CapabilityTopology::new();
    let mut provider = GateContract::new("DriverStorage");
    provider.provide(GateDomain::storage_read());
    topo.register_contract(1, provider);

    let mut consumer = GateContract::new("App");
    consumer.require(GateDomain::storage_read());
    topo.register_contract(2, consumer);

    let result = topo.verify(&logicodex::tier2::MetadataGraph::new());
    assert!(result.valid);
    assert_eq!(result.violations.len(), 0);
    assert_eq!(result.gates_satisfied, 1);
    assert_eq!(result.gates_unsatisfied, 0);
}

// ─── 6. CapabilityTopology: detect missing provider ───

#[test]
fn topology_invalid_when_require_no_provider() {
    let mut topo = CapabilityTopology::new();
    let mut consumer = GateContract::new("App");
    consumer.require(GateDomain::storage_read());
    topo.register_contract(2, consumer);

    let result = topo.verify(&logicodex::tier2::MetadataGraph::new());
    assert!(!result.valid);
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.gates_satisfied, 0);
    assert_eq!(result.gates_unsatisfied, 1);
}

// ─── 7. CapabilityTopology: multiple consumers one provider ───

#[test]
fn topology_one_provider_many_consumers() {
    let mut topo = CapabilityTopology::new();
    let mut provider = GateContract::new("DriverNet");
    provider.provide(GateDomain::net_send());
    topo.register_contract(1, provider);

    for i in 2..5 {
        let mut consumer = GateContract::new(format!("App{}", i));
        consumer.require(GateDomain::net_send());
        topo.register_contract(i, consumer);
    }

    let result = topo.verify(&logicodex::tier2::MetadataGraph::new());
    assert!(result.valid);
    assert_eq!(result.gates_satisfied, 3); // 3 consumers × 1 gate
}

// ─── 8. CapabilityTopology: serialize to .cap ───

#[test]
fn topology_serialize_not_empty() {
    let mut topo = CapabilityTopology::new();
    let mut contract = GateContract::new("Test");
    contract.provide(GateDomain::ui_display());
    topo.register_contract(1, contract);

    let lines = topo.serialize();
    assert!(!lines.is_empty());
    assert!(lines[0].contains("Capability Topology"));
}

#[test]
fn topology_serialize_has_sections() {
    let mut topo = CapabilityTopology::new();
    let mut c1 = GateContract::new("P");
    c1.provide(GateDomain::storage_read());
    let mut c2 = GateContract::new("C");
    c2.require(GateDomain::storage_read());
    topo.register_contract(1, c1);
    topo.register_contract(2, c2);

    let lines = topo.serialize();
    let content = lines.join("\n");
    assert!(content.contains("PROVIDERS"));
    assert!(content.contains("CONSUMERS"));
    assert!(content.contains("CONTRACTS"));
}

// ─── 9. CapabilityDiff: detect added gates ───

#[test]
fn diff_detects_added_gate() {
    let mut old = CapabilityTopology::new();
    let mut c1 = GateContract::new("App");
    c1.require(GateDomain::storage_read());
    old.register_contract(1, c1);

    let mut new = CapabilityTopology::new();
    let mut c2 = GateContract::new("App");
    c2.require(GateDomain::storage_read());
    c2.require(GateDomain::net_send()); // NEW requirement
    new.register_contract(1, c2);

    let diff = diff_topology(&old, &new);
    assert!(diff.has_changes());
    assert!(!diff.added_gates.is_empty());
}

// ─── 10. CapabilityDiff: privilege escalation ───

#[test]
fn diff_detects_privilege_escalation_net_raw() {
    let mut old = CapabilityTopology::new();
    let mut c1 = GateContract::new("Utils");
    c1.require(GateDomain::storage_read());
    old.register_contract(1, c1);

    let mut new = CapabilityTopology::new();
    let mut c2 = GateContract::new("Utils");
    c2.require(GateDomain::storage_read());
    c2.require(GateDomain::net_raw()); // PRIVILEGE ESCALATION
    new.register_contract(1, c2);

    let diff = diff_topology(&old, &new);
    assert!(diff.has_escalation());
    assert!(!diff.privilege_escalation.is_empty());
}

#[test]
fn diff_detects_privilege_escalation_hw() {
    let mut old = CapabilityTopology::new();
    let mut c1 = GateContract::new("App");
    c1.require(GateDomain::ui_display());
    old.register_contract(1, c1);

    let mut new = CapabilityTopology::new();
    let mut c2 = GateContract::new("App");
    c2.require(GateDomain::ui_display());
    c2.require(GateDomain::hw_gpio()); // PRIVILEGE ESCALATION
    new.register_contract(1, c2);

    let diff = diff_topology(&old, &new);
    assert!(diff.has_escalation());
}

// ─── 11. CapabilityDiff: no escalation for safe additions ───

#[test]
fn diff_no_escalation_for_safe_addition() {
    let mut old = CapabilityTopology::new();
    let mut c1 = GateContract::new("App");
    c1.require(GateDomain::storage_read());
    old.register_contract(1, c1);

    let mut new = CapabilityTopology::new();
    let mut c2 = GateContract::new("App");
    c2.require(GateDomain::storage_read());
    c2.require(GateDomain::ui_display()); // Safe addition
    new.register_contract(1, c2);

    let diff = diff_topology(&old, &new);
    assert!(diff.has_changes());
    assert!(!diff.has_escalation());
}

// ─── 12. TopologyVerifyResult format violations ───

#[test]
fn verify_result_formats_violations() {
    let mut topo = CapabilityTopology::new();
    let mut consumer = GateContract::new("BadApp");
    consumer.require(GateRef::new("Net", "Raw", GateType::Hardware));
    topo.register_contract(1, consumer);

    let result = topo.verify(&logicodex::tier2::MetadataGraph::new());
    let formatted = result.format_violations();
    assert_eq!(formatted.len(), 1);
    assert!(formatted[0].contains("BadApp"));
    assert!(formatted[0].contains("Net.Raw"));
}
