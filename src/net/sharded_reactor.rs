// =========================================================================
// Logicodex v1.34.0-alpha — Network Reactor: Sharded Multi-Core Reactor
//
// "The Sharded Deterministic Reactor"
//
// Architecture:
//   ShardedReactor { shards: Vec<ShardInstance> }
//   ShardInstance { reactor: Reactor, pool: ShardLocalPool, core_id }
//
// Prinsip:
//   1. Per-CPU-core reactor instance
//   2. Static affinity: service → shard → core (compile-time)
//   3. Zero-Sharing: Tiada shared state antara shards
//   4. Cross-Shard = Door Only: SPSC Message Passing
//
// Fasa 1 (v1.33): Single-threaded Reactor
// Fasa 2 (v1.34): Sharded multi-core (INI)
//
// Prestasi: Near-linear scaling dengan CPU cores — tiada lock contention.
// =========================================================================

use super::affinity::{self, AffinityError};
use super::connection::ConnectionStats;
use super::reactor::Reactor;
use super::shard_local_pool::{PoolStats, ShardLocalPool};
use super::service::ServiceRegistry;
use crate::tier2::shard::{ShardAssignment, ShardTopology, ShardVerifyResult};
use std::collections::HashMap;

/// Satu instance shard — satu Reactor pada satu CPU core.
pub struct ShardInstance {
    /// Shard ID
    pub shard_id: u32,
    /// CPU core yang dipin
    pub core_id: u32,
    /// Reactor untuk shard ini
    pub reactor: Reactor,
    /// Memory pool lokal (tidak dikongsi)
    pub pool: ShardLocalPool,
    /// Nama-nama servis dalam shard ini
    pub services: Vec<String>,
    /// Sedang berjalan?
    pub running: bool,
}

impl ShardInstance {
    /// Cipta shard instance daripada assignment.
    pub fn from_assignment(assignment: &ShardAssignment) -> Self {
        let pool = ShardLocalPool::with_mb(assignment.budget_mb);
        let reactor = Reactor::new();

        Self {
            shard_id: assignment.shard_id,
            core_id: assignment.core_id,
            reactor,
            pool,
            services: assignment.services.clone(),
            running: false,
        }
    }

    /// Pin thread ke core dan jalankan reactor.
    /// Untuk v1.34.0-alpha: stub (tidak spawn thread sebenar dalam tests)
    pub fn run(&mut self) -> Result<(), AffinityError> {
        affinity::set_cpu_affinity(self.core_id)?;
        self.running = true;
        eprintln!(
            "logicodex v1.34.0-alpha: Shard {} running on core {} ({} services)",
            self.shard_id, self.core_id, self.services.len()
        );
        // Dalam produksi: reactor.run() — infinite event loop
        // Untuk now: mark as running
        Ok(())
    }

    /// Hentikan shard.
    pub fn stop(&mut self) {
        self.running = false;
        self.reactor.stop();
        eprintln!(
            "logicodex v1.34.0-alpha: Shard {} stopped (core {})",
            self.shard_id, self.core_id
        );
    }

    /// Statistik shard.
    pub fn stats(&self) -> ShardStats {
        ShardStats {
            shard_id: self.shard_id,
            core_id: self.core_id,
            services: self.services.clone(),
            connections: self.reactor.connection_count(),
            pool: self.pool.stats(),
            running: self.running,
        }
    }
}

/// Statistik satu shard.
#[derive(Debug, Clone)]
pub struct ShardStats {
    pub shard_id: u32,
    pub core_id: u32,
    pub services: Vec<String>,
    pub connections: usize,
    pub pool: PoolStats,
    pub running: bool,
}

// ─── ShardedReactor ───
/// Reactor teragih — satu instance per CPU core.
pub struct ShardedReactor {
    /// Semua shard instances
    shards: Vec<ShardInstance>,
    /// Topology (diverifikasi semasa bina)
    topology: ShardTopology,
    /// Sedang berjalan?
    running: bool,
}

impl ShardedReactor {
    /// Bina ShardedReactor daripada topology yang sudah diverifikasi.
    pub fn new(topology: ShardTopology) -> Result<Self, ShardedReactorError> {
        // Verify topology dahulu
        let result = topology.verify();
        if !result.valid {
            let violations = result.violations.len();
            return Err(ShardedReactorError::TopologyInvalid { violations });
        }

        // Bina shard instances
        let mut shards = Vec::new();
        for assignment in &topology.assignments {
            let shard = ShardInstance::from_assignment(assignment);
            shards.push(shard);
        }

        Ok(Self {
            shards,
            topology,
            running: false,
        })
    }

    /// Jalankan semua shards (serentak dalam produksi, sequential dalam alpha).
    pub fn run(&mut self) -> Result<(), ShardedReactorError> {
        self.running = true;
        eprintln!("logicodex v1.34.0-alpha: ShardedReactor starting ({} shards)", self.shards.len());

        // Dalam produksi: spawn thread per shard, pin ke core
        // Untuk v1.34.0-alpha: jalankan secara sequential (stub)
        for shard in &mut self.shards {
            if let Err(e) = shard.run() {
                eprintln!("  Shard {} affinity warning: {}", shard.shard_id, e);
                // Lanjutkan walaupun affinity gagal (stub mode)
                shard.running = true;
            }
        }

        eprintln!("logicodex v1.34.0-alpha: ShardedReactor running ({} shards active)", self.shards.len());
        Ok(())
    }

    /// Hentikan semua shards (graceful shutdown).
    pub fn stop(&mut self) {
        self.running = false;
        for shard in &mut self.shards {
            shard.stop();
        }
        eprintln!("logicodex v1.34.0-alpha: ShardedReactor stopped");
    }

    /// Dapatkan shard mengikut ID.
    pub fn shard(&self, shard_id: u32) -> Option<&ShardInstance> {
        self.shards.iter().find(|s| s.shard_id == shard_id)
    }

    /// Dapatkan shard mengikut ID (mutable).
    pub fn shard_mut(&mut self, shard_id: u32) -> Option<&mut ShardInstance> {
        self.shards.iter_mut().find(|s| s.shard_id == shard_id)
    }

    /// Dapatkan shard yang mengendalikan servis tertentu.
    pub fn shard_for_service(&self, service_name: &str) -> Option<&ShardInstance> {
        self.shards.iter().find(|s| s.services.contains(&service_name.to_string()))
    }

    /// Bilangan shards.
    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }

    /// Bilangan servis keseluruhan.
    pub fn total_services(&self) -> usize {
        self.shards.iter().map(|s| s.services.len()).sum()
    }

    /// Total connections keseluruhan.
    pub fn total_connections(&self) -> usize {
        self.shards.iter().map(|s| s.reactor.connection_count()).sum()
    }

    /// Statistik semua shards.
    pub fn all_stats(&self) -> Vec<ShardStats> {
        self.shards.iter().map(|s| s.stats()).collect()
    }

    /// Serialize manifest JSON daripada topology.
    pub fn manifest_json(&self) -> String {
        self.topology.to_manifest_json()
    }

    /// Graceful shutdown semua shards + tutup semua koneksi.
    pub fn shutdown(&mut self) {
        self.stop();
        for shard in &mut self.shards {
            shard.reactor.shutdown_all();
        }
        eprintln!("logicodex v1.34.0-alpha: All shards shut down gracefully");
    }
}

impl Drop for ShardedReactor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Error ShardedReactor.
#[derive(Debug, Clone)]
pub enum ShardedReactorError {
    TopologyInvalid { violations: usize },
    ShardNotFound(u32),
    AffinityFailed { shard_id: u32, reason: String },
}

impl std::fmt::Display for ShardedReactorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShardedReactorError::TopologyInvalid { violations } => {
                write!(f, "Shard topology tidak sah ({} pelanggaran)", violations)
            }
            ShardedReactorError::ShardNotFound(id) => {
                write!(f, "Shard {} tidak wujud", id)
            }
            ShardedReactorError::AffinityFailed { shard_id, reason } => {
                write!(f, "Affinity shard {} gagal: {}", shard_id, reason)
            }
        }
    }
}

impl std::error::Error for ShardedReactorError {}
