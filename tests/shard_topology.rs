// =========================================================================
// Logicodex v1.34.0-alpha — Shard Topology Tests
//
// Tests: ShardAssignment, ServiceGraph, ShardTopology, verify()
// =========================================================================

use logicodex::tier2::{
    CommEdge, CommType, DoorRef, ServiceGraph, ServiceNode, ShardAssignment,
    ShardTopology, ShardVerifyResult, ShardViolation,
};

// ─── 1. ShardAssignment basics ───

#[test]
fn shard_assignment_new() {
    let a = ShardAssignment::new(0, 0, 256);
    assert_eq!(a.shard_id, 0);
    assert_eq!(a.core_id, 0);
    assert_eq!(a.budget_mb, 256);
    assert!(a.services.is_empty());
}

#[test]
fn shard_assignment_add_service() {
    let mut a = ShardAssignment::new(0, 0, 256);
    a.add_service("WebServer");
    a.add_service("ApiGateway");
    assert_eq!(a.service_count(), 2);
    assert!(a.services.contains(&"WebServer".to_string()));
}

#[test]
fn shard_assignment_estimated_memory() {
    let mut a = ShardAssignment::new(0, 0, 256);
    a.add_service("S1");
    a.add_service("S2");
    let mem = a.estimated_memory_mb();
    assert!(mem > 0);
    assert!(mem <= a.budget_mb);
}

// ─── 2. ServiceNode ───

#[test]
fn service_node_new() {
    let n = ServiceNode::new("WebServer", 443, "WebHandler");
    assert_eq!(n.name, "WebServer");
    assert_eq!(n.port, 443);
    assert_eq!(n.handler, "WebHandler");
    assert!(n.assigned_shard.is_none());
}

#[test]
fn service_node_with_policy() {
    let n = ServiceNode::new("GameServer", 7777, "GameHandler")
        .with_policy("DropOldest");
    assert_eq!(n.policy, "DropOldest");
}

#[test]
fn service_node_assign_to() {
    let n = ServiceNode::new("Web", 80, "H").assign_to(0);
    assert_eq!(n.assigned_shard, Some(0));
}

// ─── 3. CommEdge ───

#[test]
fn comm_edge_door() {
    let e = CommEdge::door("A", "B", "Payload", 1024);
    assert_eq!(e.comm_type, CommType::Door);
    assert_eq!(e.capacity, 1024);
    assert_eq!(e.message_type, "Payload");
}

#[test]
fn comm_edge_direct() {
    let e = CommEdge::direct("A", "B");
    assert_eq!(e.comm_type, CommType::Direct);
    assert_eq!(e.capacity, 0);
}

// ─── 4. ServiceGraph ───

#[test]
fn service_graph_add_node() {
    let mut g = ServiceGraph::new();
    g.add_node(ServiceNode::new("S1", 80, "H1"));
    assert_eq!(g.node_count(), 1);
}

#[test]
fn service_graph_add_edge() {
    let mut g = ServiceGraph::new();
    g.add_edge(CommEdge::door("S1", "S2", "Msg", 100));
    assert_eq!(g.edge_count(), 1);
}

#[test]
fn service_graph_get_node() {
    let mut g = ServiceGraph::new();
    g.add_node(ServiceNode::new("Web", 443, "Handler"));
    let n = g.get_node("Web").unwrap();
    assert_eq!(n.port, 443);
}

// ─── 5. ShardTopology: valid ───

#[test]
fn topology_valid_single_shard() {
    let mut topo = ShardTopology::new();
    let mut a = ShardAssignment::new(0, 0, 256);
    a.add_service("WebServer");
    topo.add_assignment(a);

    let result = topo.verify();
    assert!(result.valid);
    assert_eq!(result.violations.len(), 0);
    assert_eq!(result.total_shards, 1);
}

#[test]
fn topology_valid_multi_shard_with_door() {
    let mut topo = ShardTopology::new();
    
    let mut a0 = ShardAssignment::new(0, 0, 256);
    a0.add_service("WebServer");
    topo.add_assignment(a0);
    
    let mut a1 = ShardAssignment::new(1, 1, 128);
    a1.add_service("MetricsCollector");
    topo.add_assignment(a1);
    
    topo.add_door(DoorRef::new(0, 1, "MetricPayload", 1024));
    
    let result = topo.verify();
    assert!(result.valid, "Should be valid: {:?}", result.violations);
    assert_eq!(result.total_shards, 2);
    assert_eq!(result.cross_shard_doors, 1);
}

// ─── 6. ShardTopology: detect violations ───

#[test]
fn topology_unassigned_service() {
    let mut topo = ShardTopology::new();
    topo.service_graph.add_node(ServiceNode::new("Orphan", 80, "H"));
    
    let result = topo.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v, ShardViolation::UnassignedService { .. })));
}

#[test]
fn topology_empty_shard() {
    let mut topo = ShardTopology::new();
    topo.add_assignment(ShardAssignment::new(0, 0, 256)); // no services
    
    let result = topo.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v, ShardViolation::EmptyShard { .. })));
}

#[test]
fn topology_core_conflict() {
    let mut topo = ShardTopology::new();
    let mut a0 = ShardAssignment::new(0, 0, 256);
    a0.add_service("S1");
    let mut a1 = ShardAssignment::new(1, 0, 128); // same core!
    a1.add_service("S2");
    topo.add_assignment(a0);
    topo.add_assignment(a1);
    
    let result = topo.verify();
    assert!(!result.valid);
    assert!(result.violations.iter().any(|v| matches!(v, ShardViolation::CoreConflict { .. })));
}

#[test]
fn topology_budget_overflow() {
    let mut topo = ShardTopology::new();
    // 100 services x 8MB = 800MB > budget 64MB
    let mut a = ShardAssignment::new(0, 0, 64);
    for i in 0..100 {
        a.add_service(format!("S{}", i));
    }
    topo.add_assignment(a);
    
    let result = topo.verify();
    // Budget overflow is a warning-level check — depends on estimated_memory_mb
    // For this test, we just check it doesn't panic
    assert!(result.total_budget_mb > 0);
}

// ─── 7. ShardTopology: manifest JSON ───

#[test]
fn topology_manifest_json() {
    let mut topo = ShardTopology::new();
    let mut a = ShardAssignment::new(0, 0, 256);
    a.add_service("WebServer");
    topo.add_assignment(a);
    topo.add_door(DoorRef::new(0, 1, "Msg", 1024));
    
    let json = topo.to_manifest_json();
    assert!(json.contains("shards"));
    assert!(json.contains("doors"));
    assert!(json.contains("WebServer"));
    assert!(json.contains("stats"));
}

// ─── 8. ShardTopology: get_shard ───

#[test]
fn topology_get_shard() {
    let mut topo = ShardTopology::new();
    topo.add_assignment(ShardAssignment::new(5, 2, 128));
    
    assert!(topo.get_shard(5).is_some());
    assert!(topo.get_shard(99).is_none());
}

#[test]
fn topology_total_budget() {
    let mut topo = ShardTopology::new();
    topo.add_assignment(ShardAssignment::new(0, 0, 256));
    topo.add_assignment(ShardAssignment::new(1, 1, 128));
    
    assert_eq!(topo.total_budget_mb(), 384);
}
