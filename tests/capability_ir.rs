// =========================================================================
// Logicodex v1.35.0-alpha — CapabilityGraph IR Tests
//
// Tests: CompileTarget, CapabilityRef, IRServiceNode, IRShardNode,
//        IRDoorEdge, IRGateEdge, CapabilityGraph, IRVerifyResult, IRViolation
//
// Coverage:
//   - CompileTarget parsing + serialization
//   - CapabilityRef creation + WIT mapping + hardware detection
//   - IRServiceNode builder pattern
//   - IRShardNode creation
//   - CapabilityGraph construction + integration
//   - verify() — all 6 checks
//   - to_cap() output format
//   - to_wit_stub() output format
// =========================================================================

use logicodex::tier2::{
    CapabilityGraph, CapabilityRef, CompileTarget,
    IRServiceNode, IRShardNode, IRDoorEdge, IRGateEdge,
    IRVerifyResult, IRViolation,
    GateRef, GateType,
    ShardTopology, ShardAssignment, DoorRef,
    SemanticSummary, Capability, InlineCost,
};

// ─── 1. CompileTarget ───

#[test]
fn compile_target_native() {
    let t = CompileTarget::Native;
    assert_eq!(t.as_str(), "native");
    assert_eq!(CompileTarget::from_str("native"), Some(t));
    assert_eq!(CompileTarget::from_str("elf"), Some(t));
    assert_eq!(CompileTarget::from_str("ELF"), Some(t));
}

#[test]
fn compile_target_wasm() {
    let t = CompileTarget::Wasm;
    assert_eq!(t.as_str(), "wasm");
    assert_eq!(CompileTarget::from_str("wasm"), Some(t));
    assert_eq!(CompileTarget::from_str("WASM"), Some(t));
    assert_eq!(CompileTarget::from_str("webassembly"), Some(t));
}

#[test]
fn compile_target_all() {
    let t = CompileTarget::All;
    assert_eq!(t.as_str(), "all");
    assert_eq!(CompileTarget::from_str("all"), Some(t));
    assert_eq!(CompileTarget::from_str("dual"), Some(t));
}

#[test]
fn compile_target_invalid() {
    assert_eq!(CompileTarget::from_str("invalid"), None);
    assert_eq!(CompileTarget::from_str(""), None);
}

// ─── 2. CapabilityRef ───

#[test]
fn capability_ref_new() {
    let c = CapabilityRef::new("Storage", "Baca", GateType::DirectCall);
    assert_eq!(c.domain, "Storage");
    assert_eq!(c.operation, "Baca");
    assert_eq!(c.gate_type, GateType::DirectCall);
    assert!(c.wit_mapping.is_none());
}

#[test]
fn capability_ref_canonical() {
    let c = CapabilityRef::new("Net", "Send", GateType::Message);
    assert_eq!(c.canonical(), "Net.Send");
}

#[test]
fn capability_ref_wit_mapping() {
    let c = CapabilityRef::new("Storage", "Baca", GateType::DirectCall)
        .with_wit("wasi:filesystem/read");
    assert_eq!(c.wit_mapping, Some("wasi:filesystem/read".to_string()));
}

#[test]
fn capability_ref_is_hardware() {
    let hw = CapabilityRef::new("HW", "GPIO", GateType::Hardware);
    assert!(hw.is_hardware());

    let msg = CapabilityRef::new("Net", "Send", GateType::Message);
    assert!(!msg.is_hardware());

    let dc = CapabilityRef::new("Crypto", "Hash", GateType::DirectCall);
    assert!(!dc.is_hardware());
}

#[test]
fn capability_ref_from_gate_ref() {
    let g = GateRef::new("UI", "Papar", GateType::DirectCall);
    let c = CapabilityRef::from(&g);
    assert_eq!(c.domain, "UI");
    assert_eq!(c.operation, "Papar");
    assert_eq!(c.gate_type, GateType::DirectCall);
    assert!(c.wit_mapping.is_none());
}

// ─── 3. IRServiceNode ───

#[test]
fn ir_service_node_new() {
    let s = IRServiceNode::new(1, "WebServer");
    assert_eq!(s.id, 1);
    assert_eq!(s.name, "WebServer");
    assert!(s.port.is_none());
    assert!(s.requires.is_empty());
    assert!(s.provides.is_empty());
    assert_eq!(s.handler, "");
    assert!(s.assigned_shard.is_none());
    assert_eq!(s.policy, "Block");
}

#[test]
fn ir_service_node_builder() {
    let s = IRServiceNode::new(1, "ApiGateway")
        .with_port(8080)
        .with_handler("handle_request")
        .with_shard(0)
        .require(CapabilityRef::new("Net", "Send", GateType::Message))
        .provide(CapabilityRef::new("UI", "Papar", GateType::DirectCall));

    assert_eq!(s.port, Some(8080));
    assert_eq!(s.handler, "handle_request");
    assert_eq!(s.assigned_shard, Some(0));
    assert_eq!(s.requires.len(), 1);
    assert_eq!(s.provides.len(), 1);
}

// ─── 4. IRShardNode ───

#[test]
fn ir_shard_node_new() {
    let sh = IRShardNode::new(0, 0, 256);
    assert_eq!(sh.id, 0);
    assert_eq!(sh.core_id, 0);
    assert_eq!(sh.budget_mb, 256);
    assert!(sh.services.is_empty());
    assert!(sh.allowed_gates.is_empty());
}

// ─── 5. CapabilityGraph — basic construction ───

#[test]
fn capability_graph_new() {
    let graph = CapabilityGraph::new(CompileTarget::Native);
    assert!(graph.services.is_empty());
    assert!(graph.shards.is_empty());
    assert!(graph.doors.is_empty());
    assert!(graph.gates.is_empty());
    assert_eq!(graph.target, CompileTarget::Native);
    assert_eq!(graph.version, "1.35.0-alpha");
}

#[test]
fn capability_graph_add_service() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    let s = IRServiceNode::new(1, "TestService");
    graph.add_service(s);
    assert_eq!(graph.services.len(), 1);
    assert!(graph.services.contains_key(&1));
}

#[test]
fn capability_graph_add_shard() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    let sh = IRShardNode::new(0, 0, 128);
    graph.add_shard(sh);
    assert_eq!(graph.shards.len(), 1);
    assert!(graph.shards.contains_key(&0));
}

#[test]
fn capability_graph_add_door() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    let d = IRDoorEdge {
        from_service: 1,
        to_service: 2,
        message_type: "Metrics".to_string(),
        capacity: 1024,
    };
    graph.add_door(d);
    assert_eq!(graph.doors.len(), 1);
}

#[test]
fn capability_graph_add_gate() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    let g = IRGateEdge {
        from: 1,
        to: 2,
        capability: CapabilityRef::new("Storage", "Baca", GateType::DirectCall),
    };
    graph.add_gate(g);
    assert_eq!(graph.gates.len(), 1);
}

// ─── 6. verify() — EmptyGraph ───

#[test]
fn verify_empty_graph() {
    let graph = CapabilityGraph::new(CompileTarget::Native);
    let result = graph.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v, IRViolation::EmptyGraph)));
}

// ─── 7. verify() — valid graph passes ───

#[test]
fn verify_valid_native_graph() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);

    let s1 = IRServiceNode::new(1, "WebServer");
    let s2 = IRServiceNode::new(2, "ApiHandler");
    graph.add_service(s1);
    graph.add_service(s2);

    let sh = IRShardNode::new(0, 0, 256);
    graph.add_shard(sh);

    let result = graph.verify();
    assert!(result.valid);
    assert_eq!(result.service_count, 2);
    assert_eq!(result.shard_count, 1);
}

// ─── 8. verify() — WasmHardwareGate ───

#[test]
fn verify_wasm_rejects_hardware_gate() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);

    let s = IRServiceNode::new(1, "Driver")
        .require(CapabilityRef::new("HW", "GPIO", GateType::Hardware));
    graph.add_service(s);

    let result = graph.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v,
        IRViolation::WasmHardwareGate { service, gate }
        if service == "Driver" && gate == "HW.GPIO"
    )));
}

#[test]
fn verify_all_rejects_hardware_gate() {
    let mut graph = CapabilityGraph::new(CompileTarget::All);

    let s = IRServiceNode::new(1, "Driver")
        .require(CapabilityRef::new("HW", "DMA", GateType::Hardware));
    graph.add_service(s);

    let result = graph.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v,
        IRViolation::WasmHardwareGate { service, gate }
        if service == "Driver" && gate == "HW.DMA"
    )));
}

#[test]
fn verify_native_allows_hardware_gate() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);

    let s = IRServiceNode::new(1, "Driver")
        .require(CapabilityRef::new("HW", "GPIO", GateType::Hardware));
    graph.add_service(s);

    let result = graph.verify();
    assert!(result.valid);
}

// ─── 9. verify() — InvalidShardAssignment ───

#[test]
fn verify_invalid_shard_assignment() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);

    let s = IRServiceNode::new(1, "WebServer")
        .with_shard(99); // shard 99 doesn't exist
    graph.add_service(s);

    let result = graph.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v,
        IRViolation::InvalidShardAssignment { service, shard_id }
        if service == "WebServer" && *shard_id == 99
    )));
}

// ─── 10. verify() — UnknownServiceInDoor ───

#[test]
fn verify_unknown_service_in_door() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);

    let s = IRServiceNode::new(1, "KnownService");
    graph.add_service(s);

    let d = IRDoorEdge {
        from_service: 1,  // exists
        to_service: 999,  // doesn't exist
        message_type: "Msg".to_string(),
        capacity: 256,
    };
    graph.add_door(d);

    let result = graph.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v,
        IRViolation::UnknownServiceInDoor { service_id: 999 }
    )));
}

// ─── 11. verify() — UnknownServiceInGate ───

#[test]
fn verify_unknown_service_in_gate() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);

    let s = IRServiceNode::new(1, "KnownService");
    graph.add_service(s);

    let g = IRGateEdge {
        from: 1,  // exists
        to: 888,  // doesn't exist
        capability: CapabilityRef::new("Storage", "Baca", GateType::DirectCall),
    };
    graph.add_gate(g);

    let result = graph.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v,
        IRViolation::UnknownServiceInGate { service_id: 888 }
    )));
}

// ─── 12. verify() — EmptyShard ───

#[test]
fn verify_empty_shard() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);

    let s = IRServiceNode::new(1, "LoneService");
    graph.add_service(s);

    let sh = IRShardNode::new(0, 0, 256);
    // services vec is empty
    graph.add_shard(sh);

    let result = graph.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v,
        IRViolation::EmptyShard { shard_id: 0 }
    )));
}

// ─── 13. to_cap() output format ───

#[test]
fn to_cap_has_header() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.add_service(IRServiceNode::new(1, "TestSvc"));

    let cap = graph.to_cap();
    let content = cap.join("\n");
    assert!(content.contains("Capability Graph"));
    assert!(content.contains("Target: native"));
}

#[test]
fn to_cap_has_services_section() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.add_service(IRServiceNode::new(1, "Svc1"));

    let cap = graph.to_cap();
    let content = cap.join("\n");
    assert!(content.contains("SERVICES"));
    assert!(content.contains("service Svc1"));
}

#[test]
fn to_cap_has_shards_section() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.add_service(IRServiceNode::new(1, "Svc1"));
    graph.add_shard(IRShardNode::new(0, 0, 128));

    let cap = graph.to_cap();
    let content = cap.join("\n");
    assert!(content.contains("SHARDS"));
    assert!(content.contains("shard 0"));
}

#[test]
fn to_cap_has_doors_section() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.add_service(IRServiceNode::new(1, "A"));
    graph.add_service(IRServiceNode::new(2, "B"));
    graph.add_door(IRDoorEdge {
        from_service: 1,
        to_service: 2,
        message_type: "Payload",
        capacity: 512,
    });

    let cap = graph.to_cap();
    let content = cap.join("\n");
    assert!(content.contains("DOORS"));
    assert!(content.contains("door 1 → 2"));
}

#[test]
fn to_cap_has_gates_section() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.add_service(IRServiceNode::new(1, "A"));
    graph.add_service(IRServiceNode::new(2, "B"));
    graph.add_gate(IRGateEdge {
        from: 1,
        to: 2,
        capability: CapabilityRef::new("Storage", "Baca", GateType::DirectCall),
    });

    let cap = graph.to_cap();
    let content = cap.join("\n");
    assert!(content.contains("GATES"));
    assert!(content.contains("Storage.Baca"));
}

#[test]
fn to_cap_shows_wit_mapping() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    let svc = IRServiceNode::new(1, "FileSvc")
        .require(CapabilityRef::new("Storage", "Baca", GateType::DirectCall)
            .with_wit("wasi:filesystem/read"));
    graph.add_service(svc);

    let cap = graph.to_cap();
    let content = cap.join("\n");
    assert!(content.contains("wasi:filesystem/read"));
}

// ─── 14. to_wit_stub() output format ───

#[test]
fn to_wit_stub_has_header() {
    let graph = CapabilityGraph::new(CompileTarget::Wasm);
    let wit = graph.to_wit_stub();
    assert!(wit.contains("WIT Auto-Generated"));
    assert!(wit.contains("CTL Mapper"));
}

#[test]
fn to_wit_stub_generates_worlds_for_shards() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    let mut shard = IRShardNode::new(0, 0, 256);
    shard.allowed_gates.push(CapabilityRef::new("Storage", "Baca", GateType::DirectCall));
    graph.add_shard(shard);

    let wit = graph.to_wit_stub();
    assert!(wit.contains("world shard-0"));
}

#[test]
fn to_wit_stub_generates_interfaces() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);
    graph.add_service(IRServiceNode::new(1, "Svc")
        .require(CapabilityRef::new("Storage", "Baca", GateType::DirectCall)));

    let wit = graph.to_wit_stub();
    assert!(wit.contains("interface storage"));
}

// ─── 15. service_by_name ───

#[test]
fn service_by_name_found() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.add_service(IRServiceNode::new(42, "FindMe"));

    let found = graph.service_by_name("FindMe");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 42);
}

#[test]
fn service_by_name_not_found() {
    let graph = CapabilityGraph::new(CompileTarget::Native);
    assert!(graph.service_by_name("Missing").is_none());
}

// ─── 16. from_shard_topology integration ───

#[test]
fn from_shard_topology_imports_shards() {
    let mut topo = ShardTopology::new();
    topo.add_assignment(ShardAssignment::new(0, 0, 256));
    topo.add_assignment(ShardAssignment::new(1, 1, 128));

    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.from_shard_topology(&topo);

    assert_eq!(graph.shards.len(), 2);
    assert!(graph.shards.contains_key(&0));
    assert!(graph.shards.contains_key(&1));

    let shard0 = graph.shard(0).unwrap();
    assert_eq!(shard0.core_id, 0);
    assert_eq!(shard0.budget_mb, 256);
}

#[test]
fn from_shard_topology_imports_doors() {
    let mut topo = ShardTopology::new();
    topo.add_door(DoorRef::new(0, 1, "MetricPayload", 1024));

    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.from_shard_topology(&topo);

    assert_eq!(graph.doors.len(), 1);
    assert_eq!(graph.doors[0].from_service, 0);
    assert_eq!(graph.doors[0].to_service, 1);
    assert_eq!(graph.doors[0].message_type, "MetricPayload");
    assert_eq!(graph.doors[0].capacity, 1024);
}

// ─── 17. from_semantic_summaries integration ───

#[test]
fn from_semantic_summaries_imports_services() {
    let mut summary = SemanticSummary::new_function(1, "calc".to_string(), vec![], None);
    summary.effects = Capability::PURE;
    summary.inline_cost = InlineCost::Small;
    summary.requires_gates.push(GateRef::new("Storage", "Baca", GateType::DirectCall));

    let mut graph = CapabilityGraph::new(CompileTarget::Native);
    graph.from_semantic_summaries(&[summary]);

    assert_eq!(graph.services.len(), 1);
    let svc = graph.service_by_name("calc").unwrap();
    assert_eq!(svc.id, 1);
    assert_eq!(svc.effects, Capability::PURE);
    assert_eq!(svc.inline_cost, InlineCost::Small);
    assert_eq!(svc.requires.len(), 1);
    assert_eq!(svc.requires[0].canonical(), "Storage.Baca");
}

// ─── 18. verify() — multiple violations ───

#[test]
fn verify_reports_all_violations() {
    let mut graph = CapabilityGraph::new(CompileTarget::Wasm);

    // Hardware gate (violates WASM rule)
    let s = IRServiceNode::new(1, "BadDriver")
        .with_shard(99)  // invalid shard
        .require(CapabilityRef::new("HW", "GPIO", GateType::Hardware));
    graph.add_service(s);

    let result = graph.verify();
    assert!(!result.valid);
    // Should have: WasmHardwareGate + InvalidShardAssignment + EmptyGraph... wait,
    // EmptyGraph only fires when services.is_empty(), but we have a service.
    // So: WasmHardwareGate + InvalidShardAssignment = 2 violations
    assert_eq!(result.violations.len(), 2);
}

// ─── 19. verify() — empty shard with services ───

#[test]
fn verify_shard_with_services_is_valid() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);

    let s = IRServiceNode::new(1, "Svc").with_shard(0);
    graph.add_service(s);

    let mut sh = IRShardNode::new(0, 0, 256);
    sh.services.push(1); // service assigned
    graph.add_shard(sh);

    let result = graph.verify();
    assert!(result.valid);
}

// ─── 20. verify result stats ───

#[test]
fn verify_result_counts() {
    let mut graph = CapabilityGraph::new(CompileTarget::Native);

    graph.add_service(IRServiceNode::new(1, "A"));
    graph.add_service(IRServiceNode::new(2, "B"));
    graph.add_service(IRServiceNode::new(3, "C"));

    let mut sh = IRShardNode::new(0, 0, 256);
    sh.services.push(1);
    sh.services.push(2);
    graph.add_shard(sh);

    graph.add_door(IRDoorEdge {
        from_service: 1,
        to_service: 2,
        message_type: "Msg",
        capacity: 100,
    });

    graph.add_gate(IRGateEdge {
        from: 1,
        to: 2,
        capability: CapabilityRef::new("Net", "Send", GateType::Message),
    });

    let result = graph.verify();
    assert!(result.valid);
    assert_eq!(result.service_count, 3);
    assert_eq!(result.shard_count, 1);
    assert_eq!(result.door_count, 1);
    assert_eq!(result.gate_count, 1);
}

// ─── 21. IRViolation debug format ───

#[test]
fn ir_violation_debug() {
    let v = IRViolation::EmptyGraph;
    let s = format!("{:?}", v);
    assert!(s.contains("EmptyGraph"));
}

#[test]
fn ir_verify_result_debug() {
    let r = IRVerifyResult {
        valid: true,
        violations: vec![],
        service_count: 5,
        shard_count: 2,
        door_count: 3,
        gate_count: 4,
        target: CompileTarget::Native,
    };
    let s = format!("{:?}", r);
    assert!(s.contains("valid: true"));
}
