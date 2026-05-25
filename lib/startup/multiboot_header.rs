// =========================================================================
// Logicodex v1.44 — Multiboot Header
//
// Multiboot 1 header for booting with GRUB (and compatible bootloaders).
// Place this at the beginning of the kernel image so GRUB can find it.
//
// GRUB loads the kernel at 0x100000 (1MB) and jumps to the entry point.
// This header must be within the first 8192 bytes of the kernel.
//
// Usage:
//   Include this module and place `multiboot_header` at the start of .text
//   via the linker script:
//
//   .text : {
//       *(.multiboot)    /* Multiboot header first */
//       *(.text*)        /* Code follows */
//   }
//
// Reference: https://www.gnu.org/software/grub/manual/multiboot/multiboot.html
// =========================================================================

/// Multiboot magic number — GRUB looks for this to identify the kernel.
const MULTIBOOT_MAGIC: u32 = 0x1BADB002;

/// Flags: page align + memory info
///   bit 0: align loaded modules on page boundaries
///   bit 1: provide memory map in boot info
const MULTIBOOT_FLAGS: u32 = 0x00000003;

/// Multiboot header — must be 12 bytes, aligned to 4 bytes.
#[repr(C, align(4))]
pub struct MultibootHeader {
    /// Magic number: 0x1BADB002
    magic: u32,
    /// Feature flags
    flags: u32,
    /// Checksum: -(magic + flags)
    checksum: u32,
}

impl MultibootHeader {
    /// Create a new Multiboot header.
    pub const fn new() -> Self {
        let magic = MULTIBOOT_MAGIC;
        let flags = MULTIBOOT_FLAGS;
        MultibootHeader {
            magic,
            flags,
            checksum: -(magic as i32 + flags as i32) as u32,
        }
    }
}

/// The Multiboot header instance — placed in the `.multiboot` section.
/// This is what GRUB searches for.
#[link_section = ".multiboot"]
#[used]
pub static MULTIBOOT_HEADER: MultibootHeader = MultibootHeader::new();

/// Multiboot boot information structure (provided by GRUB).
#[repr(C)]
pub struct MultibootInfo {
    /// Total size of boot info
    pub total_size: u32,
    /// Reserved (must be 0)
    pub reserved: u32,
    // Followed by variable-length tags...
}

/// Basic memory information from GRUB.
#[repr(C)]
pub struct MultibootMemoryInfo {
    /// Lower memory (KB, 0-640KB)
    pub mem_lower: u32,
    /// Upper memory (KB, above 1MB)
    pub mem_upper: u32,
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiboot_magic_is_correct() {
        assert_eq!(MULTIBOOT_MAGIC, 0x1BADB002);
    }

    #[test]
    fn multiboot_header_size() {
        assert_eq!(core::mem::size_of::<MultibootHeader>(), 12);
    }

    #[test]
    fn multiboot_header_alignment() {
        assert!(core::mem::align_of::<MultibootHeader>() >= 4);
    }

    #[test]
    fn multiboot_checksum_valid() {
        let header = MultibootHeader::new();
        let sum = header.magic.wrapping_add(header.flags).wrapping_add(header.checksum);
        assert_eq!(sum, 0, "Multiboot checksum must be 0 (magic + flags + checksum = 0)");
    }

    #[test]
    fn multiboot_header_static() {
        let h = &MULTIBOOT_HEADER;
        assert_eq!(h.magic, MULTIBOOT_MAGIC);
        assert_eq!(h.flags, MULTIBOOT_FLAGS);

        let sum = h.magic.wrapping_add(h.flags).wrapping_add(h.checksum);
        assert_eq!(sum, 0);
    }

    #[test]
    fn multiboot_info_size() {
        assert_eq!(core::mem::size_of::<MultibootInfo>(), 8);
    }

    #[test]
    fn memory_info_size() {
        assert_eq!(core::mem::size_of::<MultibootMemoryInfo>(), 8);
    }
}
