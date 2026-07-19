// =========================================================================
// UART (Serial Port) Output for Freestanding Targets
//
// x86_64 port I/O driver for COM1 (0x3F8).
// Used for debug output in bare-metal environments where there's no OS
// to provide stdout/stderr.
//
// Provides:
//   - `uart_init()`      : Initialize UART (8N1, 115200 baud)
//   - `uart_putc(c)`     : Send single byte
//   - `uart_puts(s)`     : Send string
//   - `uart_hex(n)`      : Send hex number
//   - `print!` / `println!` : Macros (same interface as std)
//
// Also provides VGA text mode output (0xB8000) as fallback/alternative.
// =========================================================================

use core::fmt;

// ─── COM1 Port Addresses ───

const PORT_DATA: u16 = 0x3F8;       // Data register (read/write)
const PORT_IER: u16 = 0x3F8 + 1;    // Interrupt Enable
const PORT_FCR: u16 = 0x3F8 + 2;    // FIFO Control
const PORT_LCR: u16 = 0x3F8 + 3;    // Line Control
const PORT_MCR: u16 = 0x3F8 + 4;    // Modem Control
const PORT_LSR: u16 = 0x3F8 + 5;    // Line Status (bit 5 = TX empty)
const PORT_DLL: u16 = 0x3F8 + 0;    // Divisor Latch Low (when DLAB=1)
const PORT_DLH: u16 = 0x3F8 + 1;    // Divisor Latch High (when DLAB=1)

// ─── Divisor for 115200 baud ───
const BAUD_DIVISOR: u16 = 1; // 115200 = 115200 / 1

// ─── Initialization ───

/// Initialize COM1 for 8N1 115200 baud.
/// Call once from `_start` before any output.
///
/// # Safety
/// Uses port I/O — safe on x86_64 bare metal only.
pub unsafe fn uart_init() {
    // Disable interrupts
    outb(PORT_IER, 0x00);
    // Enable DLAB (Divisor Latch Access Bit)
    outb(PORT_LCR, 0x80);
    // Set baud rate divisor
    outb(PORT_DLL, (BAUD_DIVISOR & 0xFF) as u8);
    outb(PORT_DLH, ((BAUD_DIVISOR >> 8) & 0xFF) as u8);
    // 8 bits, no parity, one stop bit (8N1)
    outb(PORT_LCR, 0x03);
    // Enable FIFO, clear buffers, 14-byte threshold
    outb(PORT_FCR, 0xC7);
    // Enable IRQs, RTS/DSR set
    outb(PORT_MCR, 0x0B);
}

// ─── Core Output ───

/// Send a single byte to the UART.
/// Polls the transmitter holding register until it's ready.
///
/// # Safety
/// Uses port I/O — safe on x86_64 bare metal only.
pub unsafe fn uart_putc(byte: u8) {
    // Wait for transmitter holding register to be empty
    while (inb(PORT_LSR) & 0x20) == 0 {
        core::hint::spin_loop();
    }
    outb(PORT_DATA, byte);
}

/// Send a byte slice to the UART.
///
/// # Safety
/// Uses port I/O — safe on x86_64 bare metal only.
pub unsafe fn uart_write_bytes(bytes: &[u8]) {
    for &b in bytes {
        uart_putc(b);
    }
}

// ─── Formatted Output ───

/// Send a string to the UART.
pub fn uart_puts(s: &str) {
    for byte in s.bytes() {
        unsafe { uart_putc(byte); }
    }
}

/// Send a newline.
pub fn uart_newline() {
    unsafe {
        uart_putc(b'\r');
        uart_putc(b'\n');
    }
}

/// Send a `u64` as hexadecimal (with "0x" prefix).
pub fn uart_hex(n: u64) {
    uart_puts("0x");
    for i in (0..16).rev() {
        let digit = ((n >> (i * 4)) & 0xF) as u8;
        let c = if digit < 10 { b'0' + digit } else { b'a' + (digit - 10) };
        unsafe { uart_putc(c); }
    }
}

/// Send a `u64` as decimal.
pub fn uart_decimal(n: u64) {
    if n == 0 {
        unsafe { uart_putc(b'0'); }
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut n = n;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        unsafe { uart_putc(buf[j]); }
    }
}

// ─── fmt::Write implementation ───

/// UART writer implementing `core::fmt::Write`.
/// Use with `write!()` and `writeln!()` macros.
pub struct UartWriter;

impl fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        uart_puts(s);
        Ok(())
    }
}

// ─── print! / println! macros ───
// Same interface as std::print! / std::println!

/// Print a formatted string to the UART (no newline).
#[macro_export]
macro_rules! uart_print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut writer = $crate::uart::UartWriter;
        let _ = write!(writer, $($arg)*);
    });
}

/// Print a formatted string to the UART with newline.
#[macro_export]
macro_rules! uart_println {
    () => ($crate::uart_print!("\r\n"));
    ($($arg:tt)*) => ($crate::uart_print!("{}\r\n", format_args!($($arg)*)));
}

// ─── VGA Text Mode ───
// Alternative output: write directly to VGA text buffer at 0xB8000

const VGA_BUFFER: *mut u8 = 0xB8000 as *mut u8;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

/// VGA color attributes
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

/// VGA text mode writer.
pub struct VgaWriter {
    row: usize,
    col: usize,
    color: u8,
}

impl VgaWriter {
    /// Create a new VGA writer with default colors (white on black).
    pub const fn new() -> Self {
        VgaWriter {
            row: 0,
            col: 0,
            color: vga_entry_color(VgaColor::White, VgaColor::Black),
        }
    }

    /// Set foreground and background colors.
    pub fn set_color(&mut self, fg: VgaColor, bg: VgaColor) {
        self.color = vga_entry_color(fg, bg);
    }

    /// Write a single byte to the VGA buffer.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.col = 0,
            byte => {
                if self.col >= VGA_WIDTH {
                    self.new_line();
                }
                let idx = self.row * VGA_WIDTH + self.col;
                unsafe {
                    *VGA_BUFFER.add(idx * 2) = byte;
                    *VGA_BUFFER.add(idx * 2 + 1) = self.color;
                }
                self.col += 1;
            }
        }
    }

    /// Write a string to the VGA buffer.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Printable ASCII or newline
                0x20..=0x7E | b'\n' | b'\r' => self.write_byte(byte),
                // Non-printable: print placeholder
                _ => self.write_byte(b'?'),
            }
        }
    }

    fn new_line(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row >= VGA_HEIGHT {
            self.scroll_up();
        }
    }

    fn scroll_up(&mut self) {
        // Copy rows 1..24 up to 0..23
        for row in 1..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                let src_idx = row * VGA_WIDTH + col;
                let dst_idx = (row - 1) * VGA_WIDTH + col;
                unsafe {
                    *VGA_BUFFER.add(dst_idx * 2) = *VGA_BUFFER.add(src_idx * 2);
                    *VGA_BUFFER.add(dst_idx * 2 + 1) = *VGA_BUFFER.add(src_idx * 2 + 1);
                }
            }
        }
        // Clear last row
        for col in 0..VGA_WIDTH {
            let idx = (VGA_HEIGHT - 1) * VGA_WIDTH + col;
            unsafe {
                *VGA_BUFFER.add(idx * 2) = b' ';
                *VGA_BUFFER.add(idx * 2 + 1) = self.color;
            }
        }
        self.row = VGA_HEIGHT - 1;
    }
}

/// Create VGA color attribute byte (foreground + background).
const fn vga_entry_color(fg: VgaColor, bg: VgaColor) -> u8 {
    (bg as u8) << 4 | (fg as u8)
}

// ─── Port I/O helpers ───

/// x86_64 `inb` instruction — read byte from I/O port.
#[inline]
unsafe fn inb(port: u16) -> u8 {
    let result: u8;
    core::arch::asm!("in al, dx", out("al") result, in("dx") port, options(nomem, nostack, preserves_flags));
    result
}

/// x86_64 `outb` instruction — write byte to I/O port.
#[inline]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

// =========================================================================
// Tests
// =========================================================================

