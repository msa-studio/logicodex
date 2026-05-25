// =========================================================================
// Logicodex v1.34.0-alpha — Sharded Reactor Tests
//
// Tests: ShardInstance, ShardLocalPool, ShardedReactor, Affinity
// =========================================================================

use logicodex::net::{
    AcquireResult, BudgetError, PoolStats, ShardInstance, ShardLocalPool,
    ShardStats, ShardedReactor, ShardedReactorError,
};
use logicodex::tier2::{
    DoorRef, ShardAssignment, ShardTopology,
};

// ─── 1. ShardLocalPool: basic allocation ───

#[test]
fn pool_new_with_mb() {
    let p = ShardLocalPool::with_mb(64);
    assert_eq!(p.total(), 64 * 1024 * 1024);
    assert_eq!(p.used(), 0);
    assert_eq!(p.available(), 64 * 1024 * 1024);
}

#[test]
fn pool_acquire_success() {
    let mut p = ShardLocalPool::with_mb(1); // 1MB
    let result = p.acquire(1024).unwrap();
    assert_eq!(result.size, 1024);
    assert_eq!(p.used(), 1024);
    assert_eq!(p.active_allocs(), 1);
}

#[test]
fn pool_acquire_exceeds_budget() {
    let mut p = ShardLocalPool::with_mb(1); // 1MB
    let result = p.acquire(2 * 1024 * 1024); // 2MB > 1MB
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BudgetError::BudgetExceeded { .. }));
}

#[test]
fn pool_acquire_zero_size() {
    let mut p = ShardLocalPool::with_mb(1);
    let result = p.acquire(0);
    assert!(matches!(result.unwrap_err(), BudgetError::ZeroSize));
}

#[test]
fn pool_release() {
    let mut p = ShardLocalPool::with_mb(1);
    p.acquire(1024).unwrap();
    assert_eq!(p.used(), 1024);
    p.release(1024);
    assert_eq!(p.used(), 0);
    assert_eq!(p.active_allocs(), 0);
}

#[test]
fn pool_utilization() {
    let mut p = ShardLocalPool::with_mb(100); // 100MB
    assert_eq!(p.utilization(), 0.0);
    p.acquire(50 * 1024 * 1024).unwrap(); // 50MB
    assert!((p.utilization() - 0.5).abs() < 0.01);
}

#[test]
fn pool_stats() {
    let mut p = ShardLocalPool::with_mb(10);
    p.acquire(5 * 1024 * 1024).unwrap();
    let s = p.stats();
    assert_eq!(s.total_bytes, 10 * 1024 * 1024);
    assert_eq!(s.used_bytes, 5 * 1024 * 1024);
    assert!(s.available_bytes > 0);
    assert_eq!(s.active_allocs, 1);
    assert_eq!(s.successful_allocs, 1);
}

// ─── 2. ShardInstance ───

#[test]
fn shard_instance_from_assignment() {
    let mut a = ShardAssignment::new(0, 2, 128);
    a.add_service("S1");
    let si = ShardInstance::from_assignment(&a);
    assert_eq!(si.shard_id, 0);
    assert_eq!(si.core_id, 2);
    assert_eq!(si.services.len(), 1);
    assert_eq!(si.reactor.connection_count(), 0);
}

#[test]
fn shard_instance_stats() {
    let a = ShardAssignment::new(3, 1, 64);
    let si = ShardInstance::from_assignment(&a);
    let stats = si.stats();
    assert_eq!(stats.shard_id, 3);
    assert_eq!(stats.core_id, 1);
    assert_eq!(stats.connections, 0);
    assert!(!stats.running);
}

// ─── 3. ShardedReactor: valid topology ───

#[test]
fn sharded_reactor_new_valid() {
    let mut topo = ShardTopology::new();
    let mut a = ShardAssignment::new(0, 0, 256);
    a.add_service("WebServer");
    topo.add_assignment(a);
    
    let reactor = ShardedReactor::new(topo);
    assert!(reactor.is_ok());
    let r = reactor.unwrap();
    assert_eq!(r.shard_count(), 1);
    assert_eq!(r.total_services(), 1);
}

#[test]
fn sharded_reactor_new_invalid() {
    let topo = ShardTopology::new(); // empty = invalid (empty shards)
    let result = ShardedReactor::new(topo);
    assert!(result.is_err());
}

#[test]
fn sharded_reactor_run() {
    let mut topo = ShardTopology::new();
    let mut a = ShardAssignment::new(0, 0, 256);
    a.add_service("WebServer");
    topo.add_assignment(a);
    
    let mut reactor = ShardedReactor::new(topo).unwrap();
    reactor.run().unwrap();
    let stats = reactor.all_stats();
    assert_eq!(stats.len(), 1);
    assert!(stats[0].running);
}

#[test]
fn sharded_reactor_shard_for_service() {
    let mut topo = ShardTopology::new();
    let mut a = ShardAssignment::new(0, 0, 256);
    a.add_service("WebServer");
    a.add_service("ApiGateway");
    topo.add_assignment(a);
    
    let reactor = ShardedReactor::new(topo).unwrap();
    let shard = reactor.shard_for_service("WebServer");
    assert!(shard.is_some());
    assert_eq!(shard.unwrap().shard_id, 0);
    assert!(reactor.shard_for_service("Unknown").is_none());
}

#[test]
fn sharded_reactor_manifest_json() {
    let mut topo = ShardTopology::new();
    let mut a = ShardAssignment::new(0, 0, 256);
    a.add_service("WebServer");
    topo.add_assignment(a);
    topo.add_door(DoorRef::new(0, 1, "Msg", 1024));
    
    let reactor = ShardedReactor::new(topo).unwrap();
    let json = reactor.manifest_json();
    assert!(json.contains("shards"));
    assert!(json.contains("doors"));
    assert!(json.contains("WebServer"));
}

#[test]
fn sharded_reactor_shutdown() {
    let mut topo = ShardTopology::new();
    let mut a = ShardAssignment::new(0, 0, 256);
    a.add_service("S1");
    topo.add_assignment(a);
    
    let mut reactor = ShardedReactor::new(topo).unwrap();
    reactor.run().unwrap();
    reactor.shutdown();
    let stats = reactor.all_stats();
    assert!(!stats[0].running);
}

// ─── 4. ShardedReactor: multi-shard ───

#[test]
fn sharded_reactor_three_shards() {
    let mut topo = ShardTopology::new();
    
    let mut a0 = ShardAssignment::new(0, 0, 256);
    a0.add_service("WebServer");
    a0.add_service("ApiGateway");
    
    let mut a1 = ShardAssignment::new(1, 1, 128);
    a1.add_service("MetricsCollector");
    
    let mut a2 = ShardAssignment::new(2, 2, 128);
    a2.add_service("AdminPanel");
    
    topo.add_assignment(a0);
    topo.add_assignment(a1);
    topo.add_assignment(a2);
    
    let reactor = ShardedReactor::new(topo).unwrap();
    assert_eq!(reactor.shard_count(), 3);
    assert_eq!(reactor.total_services(), 4);
    assert_eq!(reactor.total_connections(), 0);
}

// ─── 5. Affinity ───

#[test]
fn num_cpus_returns_positive() {
    let n = logicodex::net::affinity::num_cpus();
    assert!(n > 0);
}

#[test]
fn current_core_id_returns_zero() {
    let id = logicodex::net::affinity::current_core_id();
    assert_eq!(id, 0); // stub
}

#[test]
fn invalid_core_rejected() {
    let max = logicodex::net::affinity::num_cpus() as u32;
    assert!(!logicodex::net::affinity::is_valid_core(max + 100));
}
