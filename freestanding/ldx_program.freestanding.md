# Logicodex Freestanding Target Plan

Target artifact: `/home/msa_admin/SprojeX/logicodex/freestanding/ldx_program.o`
LLVM target triple: `x86_64-unknown-none`
Architecture: `X86_64`
LLVM features: `+sse2`
Code model: `Kernel`

The `--target freestanding` path emits a raw object for bootloader, kernel, hypervisor, or firmware integration. The backend selects:
- Target triple: `x86_64-unknown-none`
- Relocation: static
- Code model: `Kernel`
- Entry symbol: `_start`
- Features: `+sse2`
- Startup: `_start` (stack init, BSS zero, data copy)
- Allocator: bump allocator
- Output: UART 0x3F8 + VGA 0xB8000

Physical memory access plan: `PhysicalMemoryAccessPlan { raw_pointer_type: "*int", vga_text_buffer: 753664, serial_uart_port: 1016, requires_unsafe_backend_gate: true }`

Raw pointer representation (`*int`) is reserved for memory-mapped I/O (VGA `0xB8000`, UART `0x3F8`) under explicit backend safety gates.
