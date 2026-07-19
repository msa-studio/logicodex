// =========================================================================
// Network Reactor: Service Manifest + Registry
//
// Servis dideklarasikan secara deklaratif dalam kod sumber Logicodex:
//
//   service WebServer {
//       port: 443,
//       requires: Net.Admin,
//       handler: WebHandler,
//       policy: Block,
//   }
//
// Kompiler akan:
//   1. Daftarkan servis ke dalam ServiceRegistry
//   2. Verify requires gate ada dalam CapabilityTopology
//   3. Generate Service record dalam .cap file
//   4. Kompiler-reject servis dengan gate tak sah / port duplicate
//
// Tiada bind(), listen(), accept() manual. Semua diuruskan oleh
// Reactor berdasarkan manifest.
// =========================================================================

use super::policy::{BackpressurePolicy, PolicyConfig};
use crate::tier2::gate::GateRef;
use std::collections::HashMap;

/// Manifest satu servis rangkaian.
/// Dideklarasikan oleh programmer dalam kod .ldx
#[derive(Debug, Clone)]
pub struct Service {
    /// Nama servis (unik dalam satu program)
    pub name: String,
    /// Port TCP/UDP
    pub port: u16,
    /// Gate yang diperlukan oleh servis ini (diverifikasi oleh CapabilityTopology)
    pub requires_gate: Option<GateRef>,
    /// Nama handler function/actor
    pub handler: String,
    /// Polisi backpressure
    pub policy: BackpressurePolicy,
    /// Konfigurasi polisi lengkap
    pub policy_config: PolicyConfig,
    /// Servis aktif atau tidak
    pub enabled: bool,
}

impl Service {
    /// Cipta servis baru.
    pub fn new(
        name: impl Into<String>,
        port: u16,
        handler: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            port,
            requires_gate: None,
            handler: handler.into(),
            policy: BackpressurePolicy::default(),
            policy_config: PolicyConfig::default(),
            enabled: true,
        }
    }

    /// Tetapkan gate yang diperlukan.
    pub fn with_gate(mut self, gate: GateRef) -> Self {
        self.requires_gate = Some(gate);
        self
    }

    /// Tetapkan polisi backpressure.
    pub fn with_policy(mut self, policy: BackpressurePolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Tetapkan konfigurasi polisi.
    pub fn with_config(mut self, config: PolicyConfig) -> Self {
        self.policy_config = config;
        self
    }

    /// Format sebagai service manifest (untuk .cap file).
    pub fn to_manifest(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("service {} {{", self.name));
        lines.push(format!("    port: {},", self.port));
        if let Some(gate) = &self.requires_gate {
            lines.push(format!("    requires: {},", gate.full()));
        }
        lines.push(format!("    handler: {},", self.handler));
        lines.push(format!("    policy: {},", self.policy.as_str()));
        lines.push(format!("    max_buffer: {},", self.policy_config.max_buffer_size));
        lines.push(format!("    max_connections: {},", self.policy_config.max_connections));
        lines.push("}".to_string());
        lines
    }

    /// Check jika servis ini menggunakan port raw / sensitif.
    pub fn is_sensitive(&self) -> bool {
        self.port < 1024 // well-known ports
    }
}

// ─── ServiceRegistry ───
/// Pendaftaran semua servis dalam satu program.
/// Dibina semasa Pass 1 dan diverifikasi semasa Pass 2.
#[derive(Debug, Default)]
pub struct ServiceRegistry {
    /// Semua servis: name → Service
    services: HashMap<String, Service>,
    /// Port → service name (elak duplicate port)
    port_map: HashMap<u16, String>,
    /// Servis mengikut keadaan
    enabled_services: Vec<String>,
}

impl ServiceRegistry {
    /// Registry kosong.
    pub fn new() -> Self {
        Self::default()
    }

    /// Daftarkan satu servis.
    /// Mengembalikan error jika nama atau port sudah digunakan.
    pub fn register(&mut self, service: Service) -> Result<(), ServiceRegistryError> {
        // Check duplicate nama
        if self.services.contains_key(&service.name) {
            return Err(ServiceRegistryError::DuplicateName(service.name.clone()));
        }

        // Check duplicate port
        if let Some(existing) = self.port_map.get(&service.port) {
            return Err(ServiceRegistryError::DuplicatePort {
                port: service.port,
                existing: existing.clone(),
                new_service: service.name.clone(),
            });
        }

        let name = service.name.clone();
        let enabled = service.enabled;

        self.port_map.insert(service.port, name.clone());
        self.services.insert(name.clone(), service);

        if enabled {
            self.enabled_services.push(name);
        }

        Ok(())
    }

    /// Cari servis mengikut nama.
    pub fn get(&self, name: &str) -> Option<&Service> {
        self.services.get(name)
    }

    /// Cari servis mengikut port.
    pub fn by_port(&self, port: u16) -> Option<&Service> {
        self.port_map.get(&port).and_then(|name| self.services.get(name))
    }

    /// Check jika nama servis wujud.
    pub fn has_service(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    /// Check jika port sudah digunakan.
    pub fn has_port(&self, port: u16) -> bool {
        self.port_map.contains_key(&port)
    }

    /// Bilangan servis.
    pub fn len(&self) -> usize {
        self.services.len()
    }

    /// Semua nama servis.
    pub fn service_names(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }

    /// Servis yang aktif.
    pub fn enabled_names(&self) -> &[String] {
        &self.enabled_services
    }

    /// Semua port yang didaftarkan.
    pub fn ports(&self) -> Vec<u16> {
        self.port_map.keys().cloned().collect()
    }

    /// Serialize semua servis ke format manifest (untuk .cap).
    pub fn serialize_manifests(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push("## SERVICES".to_string());
        for service in self.services.values() {
            lines.extend(service.to_manifest());
            lines.push(String::new());
        }
        lines
    }

    /// Verify: setiap requires_gate mesti ada dalam CapabilityTopology.
    /// Ini dipanggil semasa Pass 2.
    pub fn verify_gates(&self, available_gates: &[String]) -> Vec<GateVerificationError> {
        let mut errors = Vec::new();
        for service in self.services.values() {
            if let Some(gate) = &service.requires_gate {
                let gate_key = gate.canonical();
                if !available_gates.contains(&gate_key) {
                    errors.push(GateVerificationError {
                        service_name: service.name.clone(),
                        missing_gate: gate_key,
                    });
                }
            }
        }
        errors
    }

    /// Statistik registry.
    pub fn stats(&self) -> ServiceRegistryStats {
        let total = self.services.len();
        let sensitive = self.services.values().filter(|s| s.is_sensitive()).count();
        let with_gate = self.services.values().filter(|s| s.requires_gate.is_some()).count();

        ServiceRegistryStats {
            total_services: total,
            enabled_services: self.enabled_services.len(),
            sensitive_ports: sensitive,
            gated_services: with_gate,
            ports_used: self.port_map.len(),
        }
    }
}

/// Statistik ServiceRegistry.
#[derive(Debug, Clone)]
pub struct ServiceRegistryStats {
    pub total_services: usize,
    pub enabled_services: usize,
    pub sensitive_ports: usize,
    pub gated_services: usize,
    pub ports_used: usize,
}

/// Error pendaftaran servis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceRegistryError {
    DuplicateName(String),
    DuplicatePort { port: u16, existing: String, new_service: String },
}

impl std::fmt::Display for ServiceRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceRegistryError::DuplicateName(name) => {
                write!(f, "Servis '{}' sudah didaftarkan", name)
            }
            ServiceRegistryError::DuplicatePort { port, existing, new_service } => {
                write!(f, "Port {} sudah digunakan oleh '{}' — cuba didaftarkan oleh '{}'", port, existing, new_service)
            }
        }
    }
}

impl std::error::Error for ServiceRegistryError {}

/// Error verifikasi gate.
#[derive(Debug, Clone)]
pub struct GateVerificationError {
    pub service_name: String,
    pub missing_gate: String,
}
