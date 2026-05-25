// =========================================================================
// Logicodex v1.36.0-alpha — CTL Mapper Tests (Fasa B)
//
// Tests: WitDomain, WitOperation, CtlMapper, CtlMappingStats
//        map_capability, map_graph, generate_wit, generate_host_reactor_stub
//
// Coverage:
//   - All 6 domain mappings (Storage, Net, UI, HW, Audio, Crypto)
//   - Manual override functionality
//   - HW gate detection
//   - Unknown domain handling
//   - Full pipeline: map_and_generate_wit
//   - Stats reporting
// =========================================================================

use logicodex::tier2::{
    CtlMapper, CtlMappingStats, WitDomain, WitOperation,
    get_wit_operations, map_and_generate_wit, map_and_generate_wit_with_overrides,
    CapabilityGraph, CapabilityRef, CompileTarget,
    IRServiceNode, IRShardNode, IRGateEdge,
    GateType,
};
use std::collections::HashMap;

// ─── 1. WitDomain ───

#[test]
fn wit_domain_storage() {
    let d = WitDomain::from_logicodex_domain("Storage");
    assert_eq!(d, WitDomain::WasiFilesystem);
    assert!(d.is_known());
    assert!(!d.is_host_reactor());
    assert_eq!(d.wit_package_interface(), "wasi:filesystem");
}

#[test]
fn wit_domain_net() {
    let d = WitDomain::from_logicodex_domain("Net");
    assert_eq!(d, WitDomain::WasiSockets);
    assert!(d.is_known());
    assert_eq!(d.wit_package_interface(), "wasi:sockets");
}

#[test]
fn wit_domain_ui() {
    let d = WitDomain::from_logicodex_domain("UI");
    assert_eq!(d, WitDomain::WasiCli);
    assert!(d.is_known());
    assert_eq!(d.wit_package_interface(), "wasi:cli");
}

#[test]
fn wit_domain_hw() {
    let d = WitDomain::from_logicodex_domain("HW");
    assert_eq!(d, WitDomain::HostReactor);
    assert!(d.is_known());
    assert!(d.is_host_reactor());
    assert_eq!(d.wit_package_interface(), "logicodex:host-reactor");
}

#[test]
fn wit_domain_audio() {
    let d = WitDomain::from_logicodex_domain("Audio");
    assert_eq!(d, WitDomain::WasiIoCustom);
    assert!(d.is_known());
    assert_eq!(d.wit_package_interface(), "wasi:io/custom");
}

#[test]
fn wit_domain_crypto() {
    let d = WitDomain::from_logicodex_domain("Crypto");
    assert_eq!(d, WitDomain::WasiCrypto);
    assert!(d.is_known());
    assert_eq!(d.wit_package_interface(), "wasi:crypto");
}

#[test]
fn wit_domain_unknown() {
    let d = WitDomain::from_logicodex_domain("SomethingElse");
    assert!(matches!(d, WitDomain::Unknown(s) if s == "SomethingElse"));
    assert!(!d.is_known());
    assert_eq!(d.wit_package_interface(), "logicodex:somethingelse");
}

// ─── 2. WitOperation ───

#[test]
fn wit_operation_new() {
    let op = WitOperation::new("Baca", "read");
    assert_eq!(op.logicodex_op, "Baca");
    assert_eq!(op.wit_op, "read");
    assert!(op.params.is_empty());
    assert!(op.returns.is_none());
}

#[test]
fn wit_operation_with_param() {
    let op = WitOperation::new("Baca", "read")
        .with_param("path", "string")
        .with_return("result<string, error>");
    assert_eq!(op.params.len(), 1);
    assert_eq!(op.params[0], ("path".to_string(), "string".to_string()));
    assert_eq!(op.returns, Some("result<string, error>".to_string()));
}

#[test]
fn wit_operation_signature_no_params() {
    let op = WitOperation::new("stdout", "stdout");
    let sig = op.wit_signature();
    assert!(sig.contains("stdout: func()"));
}

#[test]
fn wit_operation_signature_with_params_and_return() {
    let op = WitOperation::new("Baca", "read")
        .with_param("path", "string")
        .with_param("offset", "u64")
        .with_return("result<list<u8>, error>");
    let sig = op.wit_signature();
    assert!(sig.contains("read: func(path: string, offset: u64)"));
    assert!(sig.contains("→"));
}

// ─── 3. get_wit_operations ───

#[test]
fn wit_ops_storage() {
    let ops = get_wit_operations("Storage");
    assert_eq!(ops.len(), 3);
    assert!(ops.iter().any(|o| o.logicodex_op == "Baca"));
    assert!(ops.iter().any(|o| o.logicodex_op == "Tulis"));
    assert!(ops.iter().any(|o| o.logicodex_op == "Padam"));
}

#[test]
fn wit_ops_net() {
    let ops = get_wit_operations("Net");
    assert_eq!(ops.len(), 3);
    assert!(ops.iter().any(|o| o.logicodex_op == "Send"));
    assert!(ops.iter().any(|o| o.logicodex_op == "Recv"));
    assert!(ops.iter().any(|o| o.logicodex_op == "Raw"));
}

#[test]
fn wit_ops_hw() {
    let ops = get_wit_operations("HW");
    assert_eq!(ops.len(), 3);
    assert!(ops.iter().any(|o| o.logicodex_op == "GPIO"));
    assert!(ops.iter().any(|o| o.logicodex_op == "Timer"));
    assert!(ops.iter().any(|o| o.logicodex_op == "DMA"));
}

#[test]
fn wit_ops_unknown_domain() {
    let ops = get_wit_operations("NotADomain");
    assert!(ops.is_empty());
}

// ─── 4. CtlMapper — new ───

#[test]
fn ctl_mapper_new() {
    let mapper = CtlMapper::new();
    let stats = mapper.stats();
    assert_eq!(stats.mappings_applied, 0);
    assert_eq!(stats.hw_gates_detected, 0);
    assert!(stats.unknown_domains.is_empty());
}

// ─── 5. CtlMapper — map_capability (Storage) ───

#[test]
fn map_capability_storage_read() {
    let mut mapper = CtlMapper::new();
    let mut cap = CapabilityRef::new("Storage", "Baca", GateType::DirectCall);
    mapper.map_capability(&mut cap);
    assert!(cap.wit_mapping.is_some());
    let wit = cap.wit_mapping.unwrap();
    assert!(wit.contains("wasi:filesystem"));
    assert!(wit.contains("read"));
}

#[test]
fn map_capability_net_send() {
    let mut mapper = CtlMapper::new();
    let mut cap = CapabilityRef::new("Net", "Send", GateType::Message);
    mapper.map_capability(&mut cap);
    assert!(cap.wit_mapping.is_some());
    let wit = cap.wit_mapping.unwrap();
    assert!(wit.contains("wasi:sockets"));
}

#[test]
fn map_capability_ui_display() {
    let mut mapper = CtlMapper::new();
    let mut cap = CapabilityRef::new("UI", "Papar", GateType::DirectCall);
    mapper.map_capability(&mut cap);
    assert!(cap.wit_mapping.is_some());
    let wit = cap.wit_mapping.unwrap();
    assert!(wit.contains("wasi:cli"));
}

#[test]
fn map_capability_hw_gpio() {
    let mut mapper = CtlMapper::new();
    let mut cap = CapabilityRef::new("HW", "GPIO", GateType::Hardware);
    mapper.map_capability(&mut cap);
    assert!(cap.wit_mapping.is_some());
    let wit = cap.wit_mapping.unwrap();
    assert!(wit.contains("host-reactor"));
    assert!(mapper.has_hw_gates());
    assert_eq!(mapper.stats().hw_gates_detected, 1);
}

#[test]
fn map_capability_audio() {
    let mut mapper = CtlMapper::new();
    let mut cap = CapabilityRef::new("Audio", "Main", GateType::Message);
    mapper.map_capability(&mut cap);
    assert!(cap.wit_mapping.is_some());
    let wit = cap.wit_mapping.unwrap();
    assert!(wit.contains("wasi:io"));
}

#[test]
fn map_capability_crypto() {
    let mut mapper = CtlMapper::new();
    let mut cap = CapabilityRef::new("Crypto", "Hash", GateType::DirectCall);
    mapper.map_capability(&mut cap);
    assert!(cap.wit_mapping.is_some());
    let wit = cap.wit_mapping.unwrap();
    assert!(wit.contains("wasi:crypto"));
}

#[test]
fn map_capability_unknown_domain() {
    let mut mapper = CtlMapper::new();
    let mut cap = CapabilityRef::new("CustomDomain", "DoIt", GateType::DirectCall);
    mapper.map_capability(&mut cap);
    assert!(cap.wit_mapping.is_some());
    let wit = cap.wit_mapping.unwrap();
    assert!(wit.contains("logicodex:custom"));
    assert!(mapper.stats().unknown_domains.contains(&"CustomDomain".to_string()));
}

// ─── 6. CtlMapper — manual override ───

#[test]
fn map_capability_manual_override() {
    let mut mapper = CtlMapper::new();
    mapper.add_override("Storage.Baca", "my-custom:custom-read");
    let mut cap = CapabilityRef::new("Storage", "Baca", GateType::DirectCall);
    mapper.map_capability(&mut cap);
    assert_eq!(cap.wit_mapping, Some("my-custom:custom-read".to_string()));
}

#[test]
fn override_takes_precedence() {
    let mut mapper = CtlMapper::new();
    // Even though Net.Send would map to wasi:sockets, override to custom
    mapper.add_override("Net.Send", "custom:my-net/send");
    let mut cap = CapabilityRef::new("Net", "Send", GateType::Message);
    mapper.map_capability(&mut cap);
    assert_eq!(cap.wit_mapping, Some("custom:my-net/send".to_string()));
}

// ─── 7. CtlMapper — map_graph ───

#[test]
fn map_graph_maps_all_capabilities() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    let svc = IRServiceNode::new(1, "FileSvc")
        .require(CapabilityRef::new("Storage", "Baca", GateType::DirectCall))
        .require(CapabilityRef::new("Net", "Send", GateType::Message));
    graph.add_service(svc);

    let mut mapper = CtlMapper::new();
    mapper.map_graph(&mut graph);

    let svc = graph.service_by_name("FileSvc").unwrap();
    assert!(svc.requires[0].wit_mapping.is_some());
    assert!(svc.requires[1].wit_mapping.is_some());
    assert_eq!(mapper.stats().mappings_applied, 2);
}

#[test]
fn map_graph_maps_shard_gates() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    let mut shard = IRShardNode::new(0, 0, 256);
    shard.allowed_gates.push(CapabilityRef::new("Crypto", "Hash", GateType::DirectCall));
    graph.add_shard(shard);

    let mut mapper = CtlMapper::new();
    mapper.map_graph(&mut graph);

    let shard = graph.shard(0).unwrap();
    assert!(shard.allowed_gates[0].wit_mapping.is_some());
}

#[test]
fn map_graph_maps_gate_edges() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.add_service(IRServiceNode::new(1, "A"));
    graph.add_service(IRServiceNode::new(2, "B"));
    graph.add_gate(IRGateEdge {
        from: 1,
        to: 2,
        capability: CapabilityRef::new("Storage", "Baca", GateType::DirectCall),
    });

    let mut mapper = CtlMapper::new();
    mapper.map_graph(&mut graph);

    assert!(graph.gates[0].capability.wit_mapping.is_some());
}

// ─── 8. CtlMapper — generate_wit ───

#[test]
fn generate_wit_has_header() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "Svc")
        .require(CapabilityRef::new("Storage", "Baca", GateType::DirectCall)));

    let mut mapper = CtlMapper::new();
    mapper.map_graph(&mut graph);
    let wit = mapper.generate_wit(&graph);

    assert!(wit.contains("CTL Mapper"));
    assert!(wit.contains("package logicodex:generated"));
}

#[test]
fn generate_wit_has_world() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "Svc")
        .require(CapabilityRef::new("Storage", "Baca", GateType::DirectCall)));

    let mut mapper = CtlMapper::new();
    mapper.map_graph(&mut graph);
    let wit = mapper.generate_wit(&graph);

    assert!(wit.contains("world"));
    assert!(wit.contains("IMPORTS"));
}

#[test]
fn generate_wit_includes_wasi_filesystem() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "Svc")
        .require(CapabilityRef::new("Storage", "Baca", GateType::DirectCall)));

    let mut mapper = CtlMapper::new();
    mapper.map_graph(&mut graph);
    let wit = mapper.generate_wit(&graph);

    // The interface name for wasi:filesystem
    assert!(wit.contains("wasi-filesystem") || wit.contains("filesystem"));
}

#[test]
fn generate_wit_hw_gates_get_host_reactor() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "Driver")
        .require(CapabilityRef::new("HW", "GPIO", GateType::Hardware)));

    let mut mapper = CtlMapper::new();
    mapper.map_graph(&mut graph);
    let wit = mapper.generate_wit(&graph);

    assert!(wit.contains("host-reactor"));
}

// ─── 9. CtlMapper — generate_host_reactor_stub ───

#[test]
fn host_reactor_stub_for_hw() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "Driver")
        .require(CapabilityRef::new("HW", "GPIO", GateType::Hardware)));

    let mut mapper = CtlMapper::new();
    mapper.map_graph(&mut graph);
    let stub = mapper.generate_host_reactor_stub();

    assert!(stub.contains("Host Reactor Stub"));
    assert!(stub.contains("host_reactor_hw_gpio"));
    assert!(stub.contains("HostReactor"));
}

#[test]
fn host_reactor_stub_empty_when_no_hw() {
    let graph = CapabilityGraph::new(CompileTarget::Wasm);
    let mapper = CtlMapper::new();
    let stub = mapper.generate_host_reactor_stub();

    // Should still have header but no functions
    assert!(stub.contains("Host Reactor Stub"));
    assert!(!stub.contains("fn host_reactor_"));
}

// ─── 10. CtlMappingStats ───

#[test]
fn stats_ok_when_no_unknown() {
    let stats = CtlMappingStats {
        mappings_applied: 5,
        hw_gates_detected: 0,
        unknown_domains: vec![],
        overrides_used: 0,
    };
    assert!(stats.is_ok());
}

#[test]
fn stats_not_ok_with_unknown() {
    let stats = CtlMappingStats {
        mappings_applied: 3,
        hw_gates_detected: 0,
        unknown_domains: vec!["WeirdDomain".to_string()],
        overrides_used: 0,
    };
    assert!(!stats.is_ok());
}

#[test]
fn stats_summary_contains_counts() {
    let stats = CtlMappingStats {
        mappings_applied: 10,
        hw_gates_detected: 2,
        unknown_domains: vec![],
        overrides_used: 1,
    };
    let summary = stats.summary();
    assert!(summary.contains("10"));
    assert!(summary.contains("2"));
    assert!(summary.contains("1"));
}

// ─── 11. Full pipeline: map_and_generate_wit ───

#[test]
fn full_pipeline_basic() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "App")
        .require(CapabilityRef::new("Storage", "Baca", GateType::DirectCall))
        .require(CapabilityRef::new("UI", "Papar", GateType::DirectCall)));

    let (wit, stats) = map_and_generate_wit(&mut graph);

    assert!(!wit.is_empty());
    assert!(stats.is_ok());
    assert_eq!(stats.mappings_applied, 2);
    assert_eq!(stats.hw_gates_detected, 0);
}

#[test]
fn full_pipeline_with_hw() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "Driver")
        .require(CapabilityRef::new("HW", "GPIO", GateType::Hardware))
        .require(CapabilityRef::new("HW", "Timer", GateType::Hardware)));

    let (wit, stats) = map_and_generate_wit(&mut graph);

    assert!(stats.hw_gates_detected, 2);
    assert!(wit.contains("host-reactor"));
}

#[test]
fn full_pipeline_with_unknown_domain() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "App")
        .require(CapabilityRef::new("WeirdThing", "DoIt", GateType::DirectCall)));

    let (wit, stats) = map_and_generate_wit(&mut graph);

    assert!(!stats.is_ok());
    assert!(stats.unknown_domains.contains(&"WeirdThing".to_string()));
    assert!(wit.contains("logicodex:custom"));
}

// ─── 12. Full pipeline with overrides ───

#[test]
fn pipeline_with_overrides() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "App")
        .require(CapabilityRef::new("Storage", "Baca", GateType::DirectCall)));

    let mut overrides = HashMap::new();
    overrides.insert("Storage.Baca".to_string(), "my-custom:special-read".to_string());

    let (wit, stats) = map_and_generate_wit_with_overrides(&mut graph, overrides);

    assert!(stats.is_ok());
    assert_eq!(stats.overrides_used, 1);
    assert_eq!(stats.mappings_applied, 1);
}

// ─── 13. CtlMapper — multiple HW gates tracked ───

#[test]
fn multiple_hw_gates_tracked() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "Driver")
        .require(CapabilityRef::new("HW", "GPIO", GateType::Hardware))
        .require(CapabilityRef::new("HW", "Timer", GateType::Hardware))
        .require(CapabilityRef::new("HW", "DMA", GateType::Hardware)));

    let mut mapper = CtlMapper::new();
    mapper.map_graph(&mut graph);

    assert_eq!(mapper.hw_gates().len(), 3);
    assert_eq!(mapper.stats().hw_gates_detected, 3);
}

// ─── 14. Default implementations ───

#[test]
fn ctl_mapper_default() {
    let mapper: CtlMapper = Default::default();
    assert_eq!(mapper.stats().mappings_applied, 0);
}

// ─── 15. Edge case: empty graph ───

#[test]
fn empty_graph_generates_empty_wit() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    let (wit, stats) = map_and_generate_wit(&mut graph);

    assert!(!wit.is_empty()); // Still has header
    assert!(wit.contains("package logicodex:generated"));
    assert_eq!(stats.mappings_applied, 0);
    assert!(stats.is_ok());
}

// ─── 16. All 6 domains in one graph ───

#[test]
fn all_six_domains_mapped() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "MegaApp")
        .require(CapabilityRef::new("Storage", "Baca", GateType::DirectCall))
        .require(CapabilityRef::new("Net", "Send", GateType::Message))
        .require(CapabilityRef::new("UI", "Papar", GateType::DirectCall))
        .require(CapabilityRef::new("HW", "GPIO", GateType::Hardware))
        .require(CapabilityRef::new("Audio", "Main", GateType::Message))
        .require(CapabilityRef::new("Crypto", "Hash", GateType::DirectCall)));

    let (wit, stats) = map_and_generate_wit(&mut graph);

    assert_eq!(stats.mappings_applied, 6);
    assert_eq!(stats.hw_gates_detected, 1);
    assert!(stats.is_ok());
    assert!(wit.contains("wasi:filesystem") || wit.contains("wasi-filesystem"));
    assert!(wit.contains("wasi:sockets") || wit.contains("wasi-sockets"));
    assert!(wit.contains("host-reactor"));
}
