// =========================================================================
// Logicodex v1.44 — Bump Allocator for Freestanding Targets
//
// A simple bump allocator: allocates by incrementing a pointer.
// No deallocation (memory grows until exhausted, then panics).
//
// Suitable for:
//   - Freestanding programs that don't free memory (common pattern)
//   - Programs where deallocation is handled by program restart
//   - Phase 1 of a more sophisticated allocator
//
// Heap: starts at __heap_start (after BSS), grows upward.
// Size: 0x80000000 - __heap_start (up to 2GB)
//
// Thread-safety: uses AtomicUsize — safe for multi-core (SMP).
// =========================================================================

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

/// Start of the heap — set by the linker script after BSS.
/// Default for testing; actual value comes from linker symbol.
const HEAP_START: usize = 0x300000; // 3MB (after code at 1MB + stack at 2MB)

/// Maximum heap size: 1MB (conservative for bare metal).
/// Can be increased when more RAM is available.
const HEAP_SIZE: usize = 0x100000; // 1MB

/// Bump allocator state.
/// `next` tracks the next free byte in the heap.
/// `end` is the exclusive upper bound of the heap.
pub struct BumpAllocator {
    /// Next free address (atomically updated)
    next: AtomicUsize,
    /// End of heap (exclusive)
    end: usize,
}

impl BumpAllocator {
    /// Create a new bump allocator with the default heap.
    ///
    /// # Safety
    /// The heap memory region must be valid and unused.
    /// Must only be called once (before any allocation).
    pub const unsafe fn new() -> Self {
        BumpAllocator {
            next: AtomicUsize::new(HEAP_START),
            end: HEAP_START + HEAP_SIZE,
        }
    }

    /// Create a bump allocator with a specific heap region.
    ///
    /// # Safety
    /// `[start, start + size)` must be valid, unused RAM.
    pub const unsafe fn with_region(start: usize, size: usize) -> Self {
        BumpAllocator {
            next: AtomicUsize::new(start),
            end: start + size,
        }
    }

    /// Initialize from linker script symbols.
    /// Called from `_start` after BSS init, before `main()`.
    ///
    /// # Safety
    /// Must be called exactly once before any allocation.
    pub unsafe fn init_from_linker(&mut self) {
        extern "C" {
            static __heap_start: u8;
        }
        let heap_start = &__heap_start as *const _ as usize;
        self.next.store(heap_start, Ordering::Relaxed);
        self.end = heap_start + HEAP_SIZE;
    }

    /// Get the total number of bytes allocated so far.
    pub fn used(&self) -> usize {
        self.next.load(Ordering::Relaxed) - HEAP_START
    }

    /// Get the total available heap size.
    pub fn total(&self) -> usize {
        self.end - HEAP_START
    }

    /// Get the remaining free bytes.
    pub fn remaining(&self) -> usize {
        self.end - self.next.load(Ordering::Relaxed)
    }
}

// ─── GlobalAlloc Implementation ───

unsafe impl GlobalAlloc for BumpAllocator {
    /// Allocate `layout.size()` bytes aligned to `layout.align()`.
    /// Returns null if out of memory (OOM).
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        // Load current position
        let mut current = self.next.load(Ordering::Relaxed);

        loop {
            // Align up to the required alignment
            let aligned = align_up(current, align);
            let new_next = aligned + size;

            // Check for overflow / out of memory
            if new_next > self.end {
                return core::ptr::null_mut(); // OOM
            }

            // Try to atomically update next (CAS loop)
            match self.next.compare_exchange_weak(
                current,
                new_next,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return aligned as *mut u8,
                Err(actual) => current = actual,
            }
        }
    }

    /// Deallocation is a no-op in bump allocator.
    /// Memory is only reclaimed on program restart.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Intentionally a no-op.
        // Freestanding programs typically don't free memory.
    }

    /// Reallocation: allocate new block, copy data, leave old block.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
        let new_ptr = self.alloc(new_layout);
        if !new_ptr.is_null() {
            let old_size = layout.size();
            let copy_size = if old_size < new_size { old_size } else { new_size };
            core::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
        }
        new_ptr
    }
}

// ─── Alignment helper ───

/// Round `addr` up to the nearest multiple of `align`.
/// `align` must be a power of 2.
const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

// ─── Global allocator instance ───
// Only active for freestanding targets. Hosted targets use the system allocator.

#[cfg(target_os = "none")]
#[global_allocator]
static ALLOCATOR: BumpAllocator = unsafe { BumpAllocator::new() };

// =========================================================================
// Tests (hosted only — use std allocator for test framework)
// =========================================================================

