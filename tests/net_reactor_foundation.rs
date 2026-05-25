// =========================================================================
// Logicodex v1.33.0-alpha — Network Reactor Foundation Tests
//
// Tests: Connection RAII, TaintState, ServiceRegistry, Reactor event loop,
//        Event types, Backpressure policies
// =========================================================================

use logicodex::ast::{Expr, Stmt, Type};
use logicodex::lexer::{Lexer, Lexicon};
use logicodex::parser::{CompilerPipeline, Parser};
use logicodex::net::{
    Action, BackpressureDecision, BackpressurePolicy, Connection, ConnectionError,
    ConnectionStats, Event, EventKind, Interest, PolicyConfig, Reactor, ReactorError,
    Service, ServiceRegistry, ServiceRegistryError, TaintState,
};

// ─── 1. TaintState basics ───

#[test]
fn taint_state_default_healthy() {
    let t = TaintState::default();
    assert_eq!(t, TaintState::Healthy);
    assert!(t.is_active());
    assert!(!t.requires_monitoring());
}

#[test]
fn taint_state_suspicious_requires_monitoring() {
    let t = TaintState::Suspicious;
    assert!(t.is_active());
    assert!(t.requires_monitoring());
}

#[test]
fn taint_state_closing_not_active() {
    let t = TaintState::Closing;
    assert!(!t.is_active());
}

#[test]
fn taint_state_escalate() {
    assert_eq!(TaintState::Healthy.escalate(), TaintState::Healthy);
    assert_eq!(TaintState::Suspicious.escalate(), TaintState::Closing);
    assert_eq!(TaintState::Closing.escalate(), TaintState::Closing);
}

#[test]
fn taint_state_recover() {
    assert_eq!(TaintState::Suspicious.recover(), TaintState::Healthy);
    assert_eq!(TaintState::Healthy.recover(), TaintState::Healthy);
    assert_eq!(TaintState::Closing.recover(), TaintState::Closing);
}

// ─── 2. Connection creation ───

#[test]
fn connection_new() {
    let c = Connection::new(42, 8080, "TestService", PolicyConfig::default());
    assert_eq!(c.fd(), 42);
    assert_eq!(c.port, 8080);
    assert_eq!(c.service_name, "TestService");
    assert_eq!(c.taint, TaintState::Healthy);
    assert!(!c.is_closed());
    assert_eq!(c.bytes_received, 0);
    assert_eq!(c.bytes_sent, 0);
}

#[test]
fn connection_read_increments_counter() {
    let mut c = Connection::new(1, 8080, "Test", PolicyConfig::default());
    let mut buf = [0u8; 256];
    let n = c.read(&mut buf).unwrap();
    assert!(n > 0);
    assert_eq!(c.bytes_received, n as u64);
}

#[test]
fn connection_write_increments_counter() {
    let mut c = Connection::new(1, 8080, "Test", PolicyConfig::default());
    let buf = [1u8; 100];
    let n = c.write(&buf).unwrap();
    assert!(n > 0);
    assert_eq!(c.bytes_sent, n as u64);
}

#[test]
fn connection_closed_cannot_read() {
    let mut c = Connection::new(1, 8080, "Test", PolicyConfig::default());
    c.shutdown();
    assert!(c.is_closed());
    let mut buf = [0u8; 256];
    assert!(c.read(&mut buf).is_err());
}

#[test]
fn connection_handle_hangup() {
    let mut c = Connection::new(1, 8080, "Test", PolicyConfig::default());
    let event = Event::new(1, EventKind::Hangup, 0);
    let action = c.handle_event(&event);
    assert_eq!(action, Action::Close);
    assert_eq!(c.taint, TaintState::Closing);
}

#[test]
fn connection_three_errors_to_closing() {
    let mut c = Connection::new(1, 8080, "Test", PolicyConfig::default());
    // 3 errors → Closing
    for _ in 0..3 {
        let event = Event::new(1, EventKind::Error, 0);
        let action = c.handle_event(&event);
        if action == Action::Close {
            break;
        }
    }
    assert_eq!(c.taint, TaintState::Closing);
}

// ─── 3. Event ───

#[test]
fn event_readable() {
    let e = Event::new(42, EventKind::Readable, 1024);
    assert!(e.is_readable());
    assert!(!e.is_writable());
    assert!(!e.is_terminal());
}

#[test]
fn event_hangup_is_terminal() {
    let e = Event::new(42, EventKind::Hangup, 0);
    assert!(e.is_terminal());
}

#[test]
fn event_closed_is_terminal() {
    let e = Event::new(42, EventKind::Closed, 0);
    assert!(e.is_terminal());
}

// ─── 4. Backpressure policies ───

#[test]
fn policy_block() {
    let p = BackpressurePolicy::Block;
    assert_eq!(p.as_str(), "Block");
    let decision = p.apply(TaintState::Healthy);
    assert_eq!(decision, BackpressureDecision::Wait);
}

#[test]
fn policy_drop_oldest() {
    let p = BackpressurePolicy::DropOldest;
    assert_eq!(p.as_str(), "DropOldest");
    let decision = p.apply(TaintState::Healthy);
    assert_eq!(decision, BackpressureDecision::Drop);
}

#[test]
fn policy_error_on_healthy() {
    let p = BackpressurePolicy::Error;
    let decision = p.apply(TaintState::Healthy);
    assert_eq!(decision, BackpressureDecision::Reject);
}

#[test]
fn policy_error_on_suspicious_closes() {
    let p = BackpressurePolicy::Error;
    let decision = p.apply(TaintState::Suspicious);
    assert_eq!(decision, BackpressureDecision::Close);
}

#[test]
fn policy_config_default() {
    let c = PolicyConfig::default();
    assert_eq!(c.backpressure, BackpressurePolicy::Block);
    assert_eq!(c.max_buffer_size, 64 * 1024);
    assert_eq!(c.max_connections, 1024);
}

#[test]
fn policy_config_high_throughput() {
    let c = PolicyConfig::high_throughput();
    assert_eq!(c.backpressure, BackpressurePolicy::DropOldest);
    assert_eq!(c.max_buffer_size, 256 * 1024);
    assert_eq!(c.max_connections, 4096);
}

// ─── 5. Service ───

#[test]
fn service_new() {
    let s = Service::new("WebServer", 443, "WebHandler");
    assert_eq!(s.name, "WebServer");
    assert_eq!(s.port, 443);
    assert_eq!(s.handler, "WebHandler");
    assert!(s.is_sensitive()); // port < 1024
    assert!(s.enabled);
}

#[test]
fn service_user_port_not_sensitive() {
    let s = Service::new("ApiServer", 8080, "ApiHandler");
    assert!(!s.is_sensitive());
}

#[test]
fn service_with_policy() {
    let s = Service::new("GameServer", 7777, "GameHandler")
        .with_policy(BackpressurePolicy::DropOldest);
    assert_eq!(s.policy, BackpressurePolicy::DropOldest);
}

#[test]
fn service_with_config() {
    let s = Service::new("StreamServer", 9000, "StreamHandler")
        .with_config(PolicyConfig::high_throughput());
    assert_eq!(s.policy_config.max_connections, 4096);
}

#[test]
fn service_manifest_not_empty() {
    let s = Service::new("Test", 8080, "Handler");
    let manifest = s.to_manifest();
    assert!(!manifest.is_empty());
    let content = manifest.join("\n");
    assert!(content.contains("service Test"));
    assert!(content.contains("port: 8080"));
    assert!(content.contains("handler: Handler"));
}

// ─── 6. ServiceRegistry ───

#[test]
fn registry_register_and_lookup() {
    let mut reg = ServiceRegistry::new();
    let s = Service::new("Web", 80, "WebH");
    reg.register(s).unwrap();
    assert!(reg.has_service("Web"));
    assert!(reg.has_port(80));
    let looked_up = reg.get("Web").unwrap();
    assert_eq!(looked_up.port, 80);
}

#[test]
fn registry_duplicate_name_rejected() {
    let mut reg = ServiceRegistry::new();
    reg.register(Service::new("Dup", 80, "H1")).unwrap();
    let result = reg.register(Service::new("Dup", 81, "H2"));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ServiceRegistryError::DuplicateName(_)));
}

#[test]
fn registry_duplicate_port_rejected() {
    let mut reg = ServiceRegistry::new();
    reg.register(Service::new("A", 80, "H1")).unwrap();
    let result = reg.register(Service::new("B", 80, "H2"));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ServiceRegistryError::DuplicatePort { .. }));
}

#[test]
fn registry_by_port() {
    let mut reg = ServiceRegistry::new();
    reg.register(Service::new("Api", 8080, "ApiH")).unwrap();
    let found = reg.by_port(8080).unwrap();
    assert_eq!(found.name, "Api");
}

#[test]
fn registry_stats() {
    let mut reg = ServiceRegistry::new();
    reg.register(Service::new("Web", 443, "H1")).unwrap(); // sensitive
    reg.register(Service::new("Api", 8080, "H2")).unwrap(); // not sensitive
    let stats = reg.stats();
    assert_eq!(stats.total_services, 2);
    assert_eq!(stats.sensitive_ports, 1);
    assert_eq!(stats.ports_used, 2);
}

// ─── 7. Reactor register/unregister ───

#[test]
fn reactor_new_is_empty() {
    let r = Reactor::new();
    assert_eq!(r.connection_count(), 0);
}

#[test]
fn reactor_register_connection() {
    let mut r = Reactor::new();
    let c = Connection::new(42, 8080, "Test", PolicyConfig::default());
    let fd = r.register(c).unwrap();
    assert_eq!(fd, 42);
    assert_eq!(r.connection_count(), 1);
}

#[test]
fn reactor_unregister_connection() {
    let mut r = Reactor::new();
    let c = Connection::new(42, 8080, "Test", PolicyConfig::default());
    r.register(c).unwrap();
    r.unregister(42);
    assert_eq!(r.connection_count(), 0);
}

#[test]
fn reactor_duplicate_fd_rejected() {
    let mut r = Reactor::new();
    let c1 = Connection::new(42, 8080, "Test1", PolicyConfig::default());
    let c2 = Connection::new(42, 8080, "Test2", PolicyConfig::default());
    r.register(c1).unwrap();
    let result = r.register(c2);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ReactorError::DuplicateFd(42)));
}

#[test]
fn reactor_max_connections() {
    let mut r = Reactor::with_capacity(2);
    let mut config = PolicyConfig::default();
    config.max_connections = 2;
    
    r.register(Connection::new(1, 8080, "S1", config.clone())).unwrap();
    r.register(Connection::new(2, 8080, "S2", config.clone())).unwrap();
    let result = r.register(Connection::new(3, 8080, "S3", config));
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ReactorError::MaxConnectionsReached));
}

// ─── 8. Reactor process events ───

#[test]
fn reactor_process_readable() {
    let mut r = Reactor::new();
    let c = Connection::new(10, 8080, "Test", PolicyConfig::default());
    r.register(c).unwrap();
    let event = Event::new(10, EventKind::Readable, 256);
    r.process_event(&event);
    assert_eq!(r.connection_count(), 1); // Keep
}

#[test]
fn reactor_process_hangup_closes() {
    let mut r = Reactor::new();
    let c = Connection::new(10, 8080, "Test", PolicyConfig::default());
    r.register(c).unwrap();
    let event = Event::new(10, EventKind::Hangup, 0);
    r.process_event(&event);
    assert_eq!(r.connection_count(), 0); // Close
}

#[test]
fn reactor_run_once_processes_events() {
    let mut r = Reactor::new();
    r.register(Connection::new(1, 8080, "S1", PolicyConfig::default())).unwrap();
    r.register(Connection::new(2, 8080, "S2", PolicyConfig::default())).unwrap();
    
    let events = vec![
        Event::new(1, EventKind::Readable, 100),
        Event::new(2, EventKind::Hangup, 0),
    ];
    let processed = r.run_once(&events);
    assert_eq!(processed, 2);
    assert_eq!(r.connection_count(), 1); // fd 2 closed
}

// ─── 9. Interest ───

#[test]
fn interest_read() {
    let i = Interest::read();
    assert!(i.read);
    assert!(!i.write);
}

#[test]
fn interest_read_write() {
    let i = Interest::read_write();
    assert!(i.read);
    assert!(i.write);
}

// ─── 10. Parser: service manifest ───

#[test]
fn parse_service_manifest() {
    let source = r#"
service WebServer {
    port: 443,
    handler: WebHandler,
    policy: Block
}
"#;
    let lexicon = Lexicon::from_str("{}").unwrap();
    let tokens = Lexer::new(source, &lexicon).tokenize().unwrap();
    let parser = Parser::new(tokens).with_pipeline(CompilerPipeline::V130);
    let program = parser.parse().unwrap();
    
    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Stmt::Service { name, port, requires, handler, policy } => {
            assert_eq!(name, "WebServer");
            assert_eq!(*port, 443);
            assert!(requires.is_none());
            assert_eq!(handler, "WebHandler");
            assert_eq!(policy, "Block");
        }
        other => panic!("Expected Service, got {:?}", other),
    }
}
