// =========================================================================
// Logicodex v1.33.0-alpha — Network Reactor Module
//
// "The Deterministic Network Reactor"
// No Global Lock, No Hidden State, No Magic.
// Semua diuruskan oleh topologi yang ditentukan oleh kompiler.
//
// Architecture:
//   Door (SPSC Ring Buffer)  → Data Transport (zero-copy, lock-free)
//   Gate (Capability)        → Compile-time contract (verified)
//   Service (Port Actor)     → Event loop + Handler
//   Reactor (epoll)          → Event dispatcher
//   Connection (RAII)        → Auto-cleanup, no socket leaks
//
// Fasa 1 (v1.33.0): Single-threaded Reactor + RAII Connection
// Fasa 2 (v1.33.0): Buffer budgeting + Taint state machine
// Fasa 3 (v1.34.0): Sharded multi-core
// =========================================================================

pub mod connection;
pub mod event;
pub mod policy;
pub mod reactor;
pub mod service;

// Re-exports
pub use connection::{Connection, ConnectionError, ConnectionStats, TaintState};
pub use event::{Action, Event, EventKind};
pub use policy::{BackpressureDecision, BackpressurePolicy, PolicyConfig};
pub use reactor::{Interest, Reactor, ReactorError};
pub use service::{GateVerificationError, Service, ServiceRegistry, ServiceRegistryError, ServiceRegistryStats};
