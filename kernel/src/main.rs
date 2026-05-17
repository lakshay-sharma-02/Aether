#![no_std]
#![no_main]

use core::panic::PanicInfo;

// ---------------------------------------------------------------------------
// VGA text-mode buffer
// ---------------------------------------------------------------------------

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_WIDTH: usize = 80;

/// Color code: foreground | (background << 4)
/// 0x55 = white (0x5) on purple (0x5) — technically magenta/purple in CGA palette
const COLOR_CODE: u8 = 0x55;

/// Write a single ASCII character directly into the VGA text buffer.
///
/// Each cell is two bytes: [character byte, color byte].
/// `col` and `row` are zero-indexed column and row positions.
unsafe fn vga_put(col: usize, row: usize, ch: u8, color: u8) {
    let offset = (row * VGA_WIDTH + col) * 2;
    VGA_BUFFER.add(offset).write_volatile(ch);
    VGA_BUFFER.add(offset + 1).write_volatile(color);
}

/// Print a string slice to the VGA buffer starting at (col, row).
unsafe fn vga_print(col: usize, row: usize, s: &[u8], color: u8) {
    for (i, &byte) in s.iter().enumerate() {
        vga_put(col + i, row, byte, color);
    }
}

// ---------------------------------------------------------------------------
// Kernel entry point
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernel_main();
}

fn kernel_main() -> ! {
    // Print "Aether" at VGA position (0, 0) with color 0x55 (white on purple).
    unsafe {
        vga_print(0, 0, b"Aether", COLOR_CODE);
    }

    // Halt loop — the kernel has nothing else to do yet.
    loop {
        x86_halt();
    }
}

/// Issue a HLT instruction to pause the CPU until the next interrupt.
#[inline(always)]
fn x86_halt() {
    unsafe {
        core::arch::asm!("hlt", options(nomem, nostack));
    }
}

// ---------------------------------------------------------------------------
// Panic handler (required for no_std)
// ---------------------------------------------------------------------------

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // On panic, print "PANIC" in red-on-black (0x04) at row 1.
    unsafe {
        vga_print(0, 1, b"PANIC", 0x04);
    }
    loop {
        x86_halt();
    }
}
