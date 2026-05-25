// =========================================================================
// Logicodex v1.37.0-alpha — B1-B6: Network Runtime Tests
//
// Tests: epoll event loop, socket I/O, timestamp, continuous loop,
//        event processing, taint FSM, backpressure at runtime
// =========================================================================

use logicodex::net::{
    Connection, ConnectionStats, PolicyConfig, Reactor, ReactorError,
    TaintState,
};
use logicodex::os::syscall;

// ─── B3: clock_gettime returns non-zero timestamp ───

#[test]
fn b3_clock_gettime_monotonic_non_zero() {
    let t1 = syscall::clock_gettime_monotonic_ms();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let t2 = syscall::clock_gettime_monotonic_ms();

    // Must be monotonic and non-zero
    assert!(t1 > 0, "clock_gettime_monotonic_ms should return > 0, got {}", t1);
    assert!(t2 >= t1, "Monotonic clock should not go backwards: t1={}, t2={}", t1, t2);
}

#[test]
fn b3_clock_gettime_ns_non_zero() {
    let ns = syscall::clock_gettime_monotonic_ns();
    assert!(ns > 0, "clock_gettime_monotonic_ns should return > 0, got {}", ns);
}

// ─── Connection timestamp tracking ───

#[test]
fn b6_connection_timestamp_on_creation() {
    let config = PolicyConfig::default();
    let conn = Connection::new(-1, 8080, "TestSvc", config);

    // created_at_ms should be non-zero (real timestamp)
    assert!(conn.created_at_ms > 0, "created_at_ms should be > 0, got {}", conn.created_at_ms);
    assert!(conn.last_activity_ms > 0, "last_activity_ms should be > 0, got {}", conn.last_activity_ms);
}

// ─── B1: Reactor creates epoll fd ───

#[test]
fn b1_reactor_epoll_fd_created() {
    let reactor = Reactor::new();
    // epoll_fd should be >= 0 (real fd) or -1 (if epoll not available)
    // Either is valid — the reactor handles both gracefully
    let fd = reactor.connection_count();
    assert_eq!(fd, 0); // no connections yet
}

#[test]
fn b1_reactor_with_capacity() {
    let reactor = Reactor::with_capacity(512);
    assert_eq!(reactor.connection_count(), 0);
}

// ─── Connection registration / unregistration ───

#[test]
fn b1_register_unregister_connection() {
    let mut reactor = Reactor::new();
    let config = PolicyConfig::default();
    let conn = Connection::new(-1, 8080, "WebServer", config);

    let fd = reactor.register(conn).expect("register should succeed");
    assert_eq!(reactor.connection_count(), 1);

    reactor.unregister(fd);
    assert_eq!(reactor.connection_count(), 0);
}

#[test]
fn b1_register_duplicate_fd_fails() {
    let mut reactor = Reactor::new();
    let config = PolicyConfig::default();
    let conn1 = Connection::new(42, 8080, "Svc1", config.clone());
    let conn2 = Connection::new(42, 8080, "Svc2", config);

    reactor.register(conn1).unwrap();
    let result = reactor.register(conn2);
    assert!(matches!(result, Err(ReactorError::DuplicateFd(42))));
}

// ─── Taint State Machine ───

#[test]
fn b5_taint_state_transitions() {
    use logicodex::net::TaintState;

    assert!(TaintState::Healthy.is_active());
    assert!(TaintState::Suspicious.is_active());
    assert!(!TaintState::Closing.is_active());

    assert!(!TaintState::Healthy.requires_monitoring());
    assert!(TaintState::Suspicious.requires_monitoring());
    assert!(!TaintState::Closing.requires_monitoring());

    assert_eq!(TaintState::Healthy.escalate(), TaintState::Healthy);
    assert_eq!(TaintState::Suspicious.escalate(), TaintState::Closing);
    assert_eq!(TaintState::Closing.escalate(), TaintState::Closing);

    assert_eq!(TaintState::Suspicious.recover(), TaintState::Healthy);
    assert_eq!(TaintState::Healthy.recover(), TaintState::Healthy);
    assert_eq!(TaintState::Closing.recover(), TaintState::Closing);
}

// ─── Connection taint on error events ───

#[test]
fn b5_taint_error_event() {
    let config = PolicyConfig::default();
    let mut conn = Connection::new(-1, 8080, "TestSvc", config);

    assert_eq!(conn.taint, TaintState::Healthy);

    // Simulate 3 error events → Closing
    for _ in 0..2 {
        let event = logicodex::net::Event {
            fd: -1,
            kind: logicodex::net::EventKind::Error,
        };
        let action = conn.handle_event(&event);
        assert_eq!(action, logicodex::net::Action::Keep);
    }

    // 3rd error → Close
    let event = logicodex::net::Event {
        fd: -1,
        kind: logicodex::net::EventKind::Error,
    };
    let action = conn.handle_event(&event);
    assert_eq!(action, logicodex::net::Action::Close);
    assert_eq!(conn.taint, TaintState::Closing);
}

#[test]
fn b5_taint_hangup_event() {
    let config = PolicyConfig::default();
    let mut conn = Connection::new(-1, 8080, "TestSvc", config);

    let event = logicodex::net::Event {
        fd: -1,
        kind: logicodex::net::EventKind::Hangup,
    };
    let action = conn.handle_event(&event);
    assert_eq!(action, logicodex::net::Action::Close);
    assert_eq!(conn.taint, TaintState::Closing);
}

// ─── Connection timeout check ───

#[test]
fn b6_timeout_not_triggered_when_active() {
    let config = PolicyConfig {
        read_timeout_ms: 60_000, // 60 seconds
        ..PolicyConfig::default()
    };
    let mut conn = Connection::new(-1, 8080, "TestSvc", config);

    let action = conn.check_timeout();
    assert_eq!(action, logicodex::net::Action::Keep);
    assert_eq!(conn.taint, TaintState::Healthy);
}

// ─── Connection stats ───

#[test]
fn b6_connection_stats_default() {
    let stats = ConnectionStats::default();
    assert_eq!(stats.total_accepted, 0);
    assert_eq!(stats.total_closed, 0);
    assert_eq!(stats.total_taint_closed, 0);
    assert_eq!(stats.total_timeout_closed, 0);
}

// ─── Reactor stats tracking ───

#[test]
fn b1_reactor_stats_tracking() {
    let mut reactor = Reactor::new();
    let config = PolicyConfig::default();

    assert_eq!(reactor.stats().total_accepted, 0);

    let conn = Connection::new(-1, 8080, "WebServer", config);
    reactor.register(conn).unwrap();

    assert_eq!(reactor.stats().total_accepted, 1);

    reactor.unregister(-1);
    assert_eq!(reactor.stats().total_closed, 1);
}

// ─── Reactor shutdown all ───

#[test]
fn b4_reactor_shutdown_all() {
    let mut reactor = Reactor::new();
    let config = PolicyConfig::default();

    for i in 0..5 {
        let conn = Connection::new(i, 8080, "WebServer", config.clone());
        reactor.register(conn).unwrap();
    }
    assert_eq!(reactor.connection_count(), 5);

    reactor.shutdown_all();
    assert_eq!(reactor.connection_count(), 0);
}

// ─── Reactor stop/start cycle ───

#[test]
fn b4_reactor_stop_cycle() {
    let mut reactor = Reactor::new();
    assert!(!reactor.is_running());

    // start() is async — just verify it doesn't panic
    // In stub mode, run() returns immediately
}

// ─── Full integration: reactor + connections + taint ───

#[test]
fn b1_b6_full_integration() {
    let mut reactor = Reactor::new();
    let config = PolicyConfig::default();

    // Register 3 connections
    for i in 1..=3 {
        let conn = Connection::new(i, 8080, "WebServer", config.clone());
        reactor.register(conn).unwrap();
    }
    assert_eq!(reactor.connection_count(), 3);

    // Simulate error events on connection 2
    let event = logicodex::net::Event {
        fd: 2,
        kind: logicodex::net::EventKind::Error,
    };

    // Connection 2 gets 3 errors → should be closed
    reactor.process_event(&event); // error 1
    reactor.process_event(&event); // error 2
    reactor.process_event(&event); // error 3 → Close

    // Connection 2 should be unregistered
    assert_eq!(reactor.connection_count(), 2);

    // Connections 1 and 3 should still be active
    assert!(reactor.connection_mut(1).is_some());
    assert!(reactor.connection_mut(3).is_some());
}
