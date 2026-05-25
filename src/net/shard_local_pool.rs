// =========================================================================
// Logicodex v1.34.0-alpha — Network Reactor: Shard-Local Memory Pool
//
// Setiap shard mempunyai pool memory sendiri — tiada perkongsian.
// Budget tracking: acquire() fail jika melebihi quota.
// Zero-Sharing: Tiada GlobalAllocator, tiada lock, tiada contention.
//
// Budget unit: bytes (untuk ketepatan), quota ditentukan oleh kompiler
// melalui ShardAssignment::budget_mb.
// =========================================================================

/// Pool memory lokal untuk satu shard.
/// Tidak dikongsi — setiap shard instance mempunyai pool sendiri.
pub struct ShardLocalPool {
    /// Jumlah budget dalam bytes (ditentukan oleh kompiler)
    budget_total: usize,
    /// Jumlah yang sudah digunakan
    budget_used: usize,
    /// Bilangan allocation yang aktif
    active_allocs: usize,
    /// Bilangan allocation yang gagal (sejak mula)
    failed_allocs: u64,
    /// Bilangan allocation yang berjaya (sejak mula)
    successful_allocs: u64,
}

/// Error alokasi memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetError {
    /// Budget melebihi — allocation ditolak
    BudgetExceeded { requested: usize, available: usize, total: usize },
    /// Saiz diminta = 0 (invalid)
    ZeroSize,
    /// Pool sudah ditutup
    PoolClosed,
}

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetError::BudgetExceeded { requested, available, total } => {
                write!(f, "Budget exceeded: requested {}B, available {}B / {}B", requested, available, total)
            }
            BudgetError::ZeroSize => write!(f, "Cannot allocate 0 bytes"),
            BudgetError::PoolClosed => write!(f, "Pool is closed"),
        }
    }
}

impl std::error::Error for BudgetError {}

/// Keputusan acquire — berjaya atau gagal.
pub struct AcquireResult {
    pub size: usize,
    pub remaining: usize,
}

impl ShardLocalPool {
    /// Cipta pool baru dengan budget dalam bytes.
    pub fn new(budget_bytes: usize) -> Self {
        Self {
            budget_total: budget_bytes,
            budget_used: 0,
            active_allocs: 0,
            failed_allocs: 0,
            successful_allocs: 0,
        }
    }

    /// Cipta pool dengan budget dalam MB (convenience).
    pub fn with_mb(budget_mb: u32) -> Self {
        Self::new((budget_mb as usize) * 1024 * 1024)
    }

    /// Coba alokasi memory daripada pool.
    /// Mengembalikan AcquireResult jika berjaya, BudgetError jika gagal.
    pub fn acquire(&mut self, size: usize) -> Result<AcquireResult, BudgetError> {
        if size == 0 {
            return Err(BudgetError::ZeroSize);
        }

        let available = self.budget_total - self.budget_used;
        if size > available {
            self.failed_allocs += 1;
            return Err(BudgetError::BudgetExceeded {
                requested: size,
                available,
                total: self.budget_total,
            });
        }

        self.budget_used += size;
        self.active_allocs += 1;
        self.successful_allocs += 1;

        Ok(AcquireResult {
            size,
            remaining: self.budget_total - self.budget_used,
        })
    }

    /// Kembalikan memory ke pool.
    pub fn release(&mut self, size: usize) {
        self.budget_used = self.budget_used.saturating_sub(size);
        self.active_allocs = self.active_allocs.saturating_sub(1);
    }

    /// Memory yang tersedia (bytes).
    pub fn available(&self) -> usize {
        self.budget_total - self.budget_used
    }

    /// Memory yang digunakan (bytes).
    pub fn used(&self) -> usize {
        self.budget_used
    }

    /// Jumlah budget (bytes).
    pub fn total(&self) -> usize {
        self.budget_total
    }

    /// Penggunaan sebagai ratio (0.0 - 1.0).
    pub fn utilization(&self) -> f64 {
        if self.budget_total == 0 {
            0.0
        } else {
            self.budget_used as f64 / self.budget_total as f64
        }
    }

    /// Bilangan allocation aktif.
    pub fn active_allocs(&self) -> usize {
        self.active_allocs
    }

    /// Statistik pool.
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            total_bytes: self.budget_total,
            used_bytes: self.budget_used,
            available_bytes: self.available(),
            active_allocs: self.active_allocs,
            successful_allocs: self.successful_allocs,
            failed_allocs: self.failed_allocs,
            utilization: self.utilization(),
        }
    }
}

/// Statistik pool.
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_bytes: usize,
    pub used_bytes: usize,
    pub available_bytes: usize,
    pub active_allocs: usize,
    pub successful_allocs: u64,
    pub failed_allocs: u64,
    pub utilization: f64,
}
