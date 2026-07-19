// =========================================================================
// Tier 2: Gate / Door Capability System
//
// "Gate menentukan SIAPA boleh bercakap, Door menentukan BAGAIMANA
//  mereka bercakap."
//
// Architecture:
//   Gate  = Kontrak compile-time (Type-Level). Menentukan identiti,
//           privilege, dan kebenaran. Divalidasi oleh 2-Pass Engine.
//   Door  = Pengangkutan runtime (SPSC Ring Buffer atau Function Pointer).
//           Tidak perlu validasi — Gate dah buktikan selamat.
//
// Tiga Jenis Gate (mengelakkan perangkap ioctl/magic numbers):
//   1. DirectCall  — Panggilan sync langsung. Inline-able. Math/crypto.
//   2. Message     — SPSC async. Lock-free queue. Sensor/telemetry/I/O.
//   3. Hardware    — Bare-metal register access. Eksklusif untuk driver.
//
// Zero Runtime Mediation: Semua semakan Gate berlaku pada masa kompil.
// Jika program berjaya dikompil, ia sudah proven safe.
// =========================================================================

use std::fmt;

// ─── GateType ───
/// Tiga jenis gate yang mengelakkan "mega gate" / ioctl trap.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GateType {
    /// Direct Call Gate: Panggilan sync ekstrem — inline-able.
    /// Sesuai untuk matematik, kriptografi, utility functions.
    /// Kompiler akan inline jika inline_cost ≤ Small.
    DirectCall,
    /// Message Gate: SPSC asynchronous — lock-free queue.
    /// Berpaut dengan arsitektur Actor/Channel.
    /// Sesuai untuk sensor, telemetry, network I/O.
    Message,
    /// Hardware/Unsafe Gate: Eksklusif untuk bare-metal.
    /// Memerlukan akses ke physical registers / unsafe blocks.
    /// Hanya boleh digunakan dalam HardwareZone.
    Hardware,
}

impl fmt::Display for GateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GateType::DirectCall => write!(f, "DirectCall"),
            GateType::Message => write!(f, "Message"),
            GateType::Hardware => write!(f, "Hardware"),
        }
    }
}

// ─── GateRef ───
/// Rujukan ke satu keupayaan (capability) dalam sistem Gate.
///
/// Format: "Domain.Operation" — contoh: "Storage.Baca", "Net.Send"
/// Domain = namespace keupayaan (Storage, Net, UI, HW, DB, etc.)
/// Operation = tindakan spesifik (Baca, Tulis, Raw, Papar, etc.)
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GateRef {
    pub domain: String,
    pub operation: String,
    pub gate_type: GateType,
}

impl GateRef {
    /// Cipta GateRef baru.
    pub fn new(
        domain: impl Into<String>,
        operation: impl Into<String>,
        gate_type: GateType,
    ) -> Self {
        Self {
            domain: domain.into(),
            operation: operation.into(),
            gate_type,
        }
    }

    /// Parse daripada string format "Domain.Operation[:Type]".
    /// Contoh: "Storage.Baca", "Net.Raw:Message", "HW.GPIO:Hardware"
    pub fn parse(s: &str) -> Result<Self, GateParseError> {
        let parts: Vec<&str> = s.split(':').collect();
        let gate_type = if parts.len() == 2 {
            match parts[1] {
                "DirectCall" => GateType::DirectCall,
                "Message" => GateType::Message,
                "Hardware" => GateType::Hardware,
                other => return Err(GateParseError::UnknownGateType(other.to_string())),
            }
        } else {
            GateType::DirectCall // default
        };

        let domain_op: Vec<&str> = parts[0].split('.').collect();
        if domain_op.len() != 2 {
            return Err(GateParseError::InvalidFormat(s.to_string()));
        }

        Ok(Self {
            domain: domain_op[0].to_string(),
            operation: domain_op[1].to_string(),
            gate_type,
        })
    }

    /// Format sebagai string canonikal.
    pub fn canonical(&self) -> String {
        format!("{}.{}", self.domain, self.operation)
    }

    /// Format penuh dengan GateType.
    pub fn full(&self) -> String {
        format!("{}.{}", self.domain, self.operation)
    }

    /// Check jika gate ini merujuk ke domain hardware.
    pub fn is_hardware(&self) -> bool {
        matches!(self.gate_type, GateType::Hardware)
    }

    /// Check jika gate ini boleh di-inline.
    pub fn is_inlineable(&self) -> bool {
        matches!(self.gate_type, GateType::DirectCall)
    }
}

impl fmt::Debug for GateRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Gate({}.{} / {})",
            self.domain, self.operation, self.gate_type
        )
    }
}

impl fmt::Display for GateRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.domain, self.operation)
    }
}

// ─── GateParseError ───
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateParseError {
    InvalidFormat(String),
    UnknownGateType(String),
}

impl fmt::Display for GateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GateParseError::InvalidFormat(s) => {
                write!(
                    f,
                    "Format gate tidak sah '{}' — jangkaan 'Domain.Operation'",
                    s
                )
            }
            GateParseError::UnknownGateType(s) => {
                write!(
                    f,
                    "Jenis gate tidak dikenali '{}' — guna DirectCall/Message/Hardware",
                    s
                )
            }
        }
    }
}

impl std::error::Error for GateParseError {}

// ─── GateContract ───
/// Kontrak gate yang diisytiharkan oleh satu modul/pakej.
/// Menentukan gate apa yang modul PROVIDE dan REQUIRE.
#[derive(Debug, Clone)]
pub struct GateContract {
    /// Nama modul yang mengisytiharkan kontrak ini
    pub module_name: String,
    /// Gate-gate yang disediakan oleh modul ini (lain boleh guna)
    pub provides: Vec<GateRef>,
    /// Gate-gate yang diperlukan oleh modul ini (mesti ada provider)
    pub requires: Vec<GateRef>,
}

impl GateContract {
    /// Kontrak kosong.
    pub fn new(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
            provides: Vec::new(),
            requires: Vec::new(),
        }
    }

    /// Tambah gate yang disediakan.
    pub fn provide(&mut self, gate: GateRef) {
        self.provides.push(gate);
    }

    /// Tambah gate yang diperlukan.
    pub fn require(&mut self, gate: GateRef) {
        self.requires.push(gate);
    }

    /// Check jika modul ini menyediakan gate tertentu.
    pub fn provides_gate(&self, gate: &GateRef) -> bool {
        self.provides
            .iter()
            .any(|g| g.domain == gate.domain && g.operation == gate.operation)
    }

    /// Check jika modul ini memerlukan gate tertentu.
    pub fn requires_gate(&self, gate: &GateRef) -> bool {
        self.requires
            .iter()
            .any(|g| g.domain == gate.domain && g.operation == gate.operation)
    }
}

// ─── GateDomain ───
/// Domain-domain gate standard dalam ekosistem Logicodex.
/// Ini adalah "vocabulary" keupayaan yang tersedia.
pub struct GateDomain;

impl GateDomain {
    /// Storage — operasi fail / storage
    pub fn storage_read() -> GateRef {
        GateRef::new("Storage", "Baca", GateType::DirectCall)
    }
    pub fn storage_write() -> GateRef {
        GateRef::new("Storage", "Tulis", GateType::DirectCall)
    }
    pub fn storage_delete() -> GateRef {
        GateRef::new("Storage", "Padam", GateType::DirectCall)
    }

    /// Net — operasi rangkaian
    pub fn net_send() -> GateRef {
        GateRef::new("Net", "Send", GateType::Message)
    }
    pub fn net_recv() -> GateRef {
        GateRef::new("Net", "Recv", GateType::Message)
    }
    pub fn net_raw() -> GateRef {
        GateRef::new("Net", "Raw", GateType::Hardware)
    }

    /// UI — antaramuka pengguna
    pub fn ui_display() -> GateRef {
        GateRef::new("UI", "Papar", GateType::DirectCall)
    }
    pub fn ui_input() -> GateRef {
        GateRef::new("UI", "Input", GateType::Message)
    }

    /// HW — hardware / bare-metal
    pub fn hw_gpio() -> GateRef {
        GateRef::new("HW", "GPIO", GateType::Hardware)
    }
    pub fn hw_timer() -> GateRef {
        GateRef::new("HW", "Timer", GateType::Hardware)
    }
    pub fn hw_dma() -> GateRef {
        GateRef::new("HW", "DMA", GateType::Hardware)
    }

    /// Audio — audio processing
    pub fn audio_play() -> GateRef {
        GateRef::new("Audio", "Main", GateType::Message)
    }
    pub fn audio_record() -> GateRef {
        GateRef::new("Audio", "Rakam", GateType::Message)
    }

    /// Crypto — kriptografi (selalu DirectCall, inline-able)
    pub fn crypto_hash() -> GateRef {
        GateRef::new("Crypto", "Hash", GateType::DirectCall)
    }
    pub fn crypto_encrypt() -> GateRef {
        GateRef::new("Crypto", "Encrypt", GateType::DirectCall)
    }
}

// ─── Semua domain sebagai vec (untuk validator, documentation) ───
pub fn all_standard_domains() -> Vec<(String, Vec<GateRef>)> {
    vec![
        (
            "Storage".to_string(),
            vec![
                GateDomain::storage_read(),
                GateDomain::storage_write(),
                GateDomain::storage_delete(),
            ],
        ),
        (
            "Net".to_string(),
            vec![
                GateDomain::net_send(),
                GateDomain::net_recv(),
                GateDomain::net_raw(),
            ],
        ),
        (
            "UI".to_string(),
            vec![GateDomain::ui_display(), GateDomain::ui_input()],
        ),
        (
            "HW".to_string(),
            vec![
                GateDomain::hw_gpio(),
                GateDomain::hw_timer(),
                GateDomain::hw_dma(),
            ],
        ),
        (
            "Audio".to_string(),
            vec![GateDomain::audio_play(), GateDomain::audio_record()],
        ),
        (
            "Crypto".to_string(),
            vec![GateDomain::crypto_hash(), GateDomain::crypto_encrypt()],
        ),
    ]
}
