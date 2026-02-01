#![allow(dead_code)]

use core::arch::asm;

const COM1: u16 = 0x3F8;
const QEMU_EXIT_PORT: u16 = 0xF4;

pub const RESET: &str  = "\x1b[0m";
pub const GREEN: &str  = "\x1b[32m";
pub const RED: &str    = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const BOLD: &str   = "\x1b[1m";

// ---------- Port I/O helpers ----------

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!(
        "in al, dx",
        in("dx") port,
        out("al") val,
        options(nomem, nostack, preserves_flags)
        );
    }
    val
}

#[inline(always)]
unsafe fn outl(port: u16, val: u32) {
    unsafe {
        asm!(
        "out dx, eax",
        in("dx") port,
        in("eax") val,
        options(nomem, nostack, preserves_flags)
        );
    }
}

// ---------- Serial (COM1) ----------
// https://www.ti.com/lit/ds/symlink/tl16c550d.pdf

pub fn serial_init() {
    unsafe {
        outb(COM1 + 1, 0x00); // Disable interrupts
        outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(COM1 + 0, 0x03); // Divisor = 3 -> 38400 baud (common/simple)
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);// 8 bits, no parity, one stop bit
        outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
        outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

#[inline(always)]
fn serial_is_transmit_empty() -> bool {
    unsafe { (inb(COM1 + 5) & 0x20) != 0 }
}

pub fn serial_write_byte(b: u8) {
    // Wait until the transmit buffer is empty.
    while !serial_is_transmit_empty() {}

    unsafe {
        outb(COM1, b);
    }
}

pub fn serial_write_str(s: &str) {
    for b in s.bytes() {
        // QEMU + many terminals expect CRLF for newlines
        if b == b'\n' {
            serial_write_byte(b'\r');
        }
        serial_write_byte(b);
    }
}

/// Writes an unsigned number in decimal (base-10), no allocation.
pub fn serial_write_number(mut n: u64) {
    // Special-case 0
    if n == 0 {
        serial_write_byte(b'0');
        return;
    }

    // Max u64 decimal digits = 20
    let mut buf = [0u8; 20];
    let mut i = buf.len();

    while n > 0 {
        let digit = (n % 10) as u8;
        n /= 10;
        i -= 1;
        buf[i] = b'0' + digit;
    }

    for &b in &buf[i..] {
        serial_write_byte(b);
    }
}

// ---------- QEMU exit (isa-debug-exit) ----------

pub fn qemu_exit_success() -> ! {
    unsafe { outl(QEMU_EXIT_PORT, 0x10) };
    halt_forever()
}

pub fn qemu_exit_fail() -> ! {
    unsafe { outl(QEMU_EXIT_PORT, 0x11) };
    halt_forever()
}

pub fn halt_forever() -> ! {
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) }
    }
}
