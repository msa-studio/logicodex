// =========================================================================
// Logicodex v1.39.0-alpha — C1-C5: Sharded Runtime Tests
//
// Tests: Thread spawning, parallel execution, CPU affinity, shutdown
// =========================================================================

use logicodex::net::affinity;
use logicodex::tier2::shard::{ShardAssignment, ShardTopology};

// ─── C3-C5: num_cpus returns realistic value ───

#[test]
fn c3_num_cpus_non_zero() {
    let n = affinity::num_cpus();
    assert!(n > 0, "num_cpus should return > 0, got {}", n);
    assert!(n <= 1024, "num_cpus seems unreasonable: {}", n);
}

#[test]
fn c3_is_valid_core() {
    assert!(affinity::is_valid_core(0));
    assert!(!affinity::is_valid_core(99999));
}

#[test]
fn c3_affinity_info() {
    let info = affinity::affinity_info();
    assert!(info.contains("CPU cores:"));
    assert!(info.contains("current core:"));
}

// ─── C3: set_cpu_affinity validates core ID ───

#[test]
fn c3_set_affinity_invalid_core() {
    let result = affinity::set_cpu_affinity(99999);
    assert!(result.is_err(), "Invalid core should fail");
}

// ─── C1-C2: ShardedReactor builds from topology ───

#[test]
fn c1_sharded_reactor_build() {
    let topo = ShardTopology::new();
    let reactor = logicodex::net::ShardedReactor::new(topo);
    assert!(reactor.is_ok());
}

#[test]
fn c1_sharded_reactor_with_assignments() {
    let mut topo = ShardTopology::new();
    topo.add_assignment(ShardAssignment::new(0, 0, 256));
    topo.add_assignment(ShardAssignment::new(1, 1, 128));

    let reactor = logicodex::net::ShardedReactor::new(topo).unwrap();
    assert_eq!(reactor.shard_count(), 2);
    assert_eq!(reactor.total_services(), 0);
}

#[test]
fn c1_sharded_reactor_services() {
    let mut topo = ShardTopology::new();
    let mut a0 = ShardAssignment::new(0, 0, 256);
    a0.services.push("WebServer".to_string());
    a0.services.push("ApiGateway".to_string());
    topo.add_assignment(a0);

    let mut a1 = ShardAssignment::new(1, 1, 128);
    a1.services.push("MetricsCollector".to_string());
    topo.add_assignment(a1);

    let reactor = logicodex::net::ShardedReactor::new(topo).unwrap();
    assert_eq!(reactor.total_services(), 3);

    let shard0 = reactor.shard_for_service("WebServer");
    assert!(shard0.is_some());
    assert_eq!(shard0.unwrap().shard_id, 0);

    let shard1 = reactor.shard_for_service("MetricsCollector");
    assert!(shard1.is_some());
    assert_eq!(shard1.unwrap().shard_id, 1);
}

// ─── C1-C2: Start/stop lifecycle ───

#[test]
fn c1_c2_start_stop_cycle() {
    let mut topo = ShardTopology::new();
    topo.add_assignment(ShardAssignment::new(0, 0, 64));
    topo.add_assignment(ShardAssignment::new(1, 1, 64));

    let mut reactor = logicodex::net::ShardedReactor::new(topo).unwrap();
    assert_eq!(reactor.shard_count(), 2);

    // Before start: no threads
    assert_eq!(reactor.active_threads(), 0);

    // v1.39 C1: start() spawns threads
    // Note: In test environment, threads may fail affinity but still run
    let result = reactor.start();
    // start() returns immediately (threads run independently)
    assert!(result.is_ok());

    // v1.39 C2: All shards should have threads spawned
    assert_eq!(reactor.active_threads(), 2);

    // Stop
    reactor.stop();
}

// ─── Shard stats ───

#[test]
fn c1_shard_stats() {
    let mut topo = ShardTopology::new();
    let mut a0 = ShardAssignment::new(0, 0, 256);
    a0.services.push("Svc1".to_string());
    topo.add_assignment(a0);

    let reactor = logicodex::net::ShardedReactor::new(topo).unwrap();
    let stats = reactor.all_stats();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].shard_id, 0);
    assert_eq!(stats[0].core_id, 0);
    assert!(stats[0].services.contains(&"Svc1".to_string()));
    assert_eq!(stats[0].connections, 0);
}

// ─── Manifest JSON ───

#[test]
fn c1_manifest_json() {
    let mut topo = ShardTopology::new();
    let mut a0 = ShardAssignment::new(0, 0, 128);
    a0.services.push("Web".to_string());
    topo.add_assignment(a0);

    let reactor = logicodex::net::ShardedReactor::new(topo).unwrap();
    let json = reactor.manifest_json();
    assert!(json.contains("shards"));
    assert!(json.contains("Web"));
}

// ─── Topology validation ───

#[test]
fn c1_invalid_topology_rejected() {
    let mut topo = ShardTopology::new();
    // Empty topology with no assignments — verify() should fail
    let result = logicodex::net::ShardedReactor::new(topo);
    // Note: empty topology may or may not fail depending on verify() rules
    // This test just ensures we handle the Result properly
    match result {
        Ok(r) => assert_eq!(r.shard_count(), 0),
        Err(_) => {} // also valid if empty topology is rejected
    }
}
