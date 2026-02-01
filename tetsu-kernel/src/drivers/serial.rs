#![allow(dead_code)]
use core::fmt;
use core::ops::Add;

pub trait PortIo {
    #[allow(clippy::missing_safety_doc)]
    unsafe fn outb(&self, port: u16, val: u8);

    #[allow(clippy::missing_safety_doc)]
    unsafe fn inb(&self, port: u16) -> u8;
}

pub struct RealPortIo;

impl PortIo for RealPortIo {
    #[inline(always)]
    unsafe fn outb(&self, port: u16, val: u8) {
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("dx") port,
                in("al") val,
                options(nomem, nostack, preserves_flags)
            );
        }
    }

    #[inline(always)]
    unsafe fn inb(&self, port: u16) -> u8 {
        let val: u8;
        unsafe {
            core::arch::asm!(
                "in al, dx",
                in("dx") port,
                out("al") val,
                options(nomem, nostack, preserves_flags)
            );
        }
        val
    }
}

#[derive(Copy, Clone)]
pub struct IoPort(u16);

impl IoPort {
    pub const COM1: Self = Self(0x3F8);
    pub const COM2: Self = Self(0x2F8);

    #[inline(always)]
    pub const fn addr(self) -> u16 {
        self.0
    }
}

impl Add<u16> for IoPort {
    type Output = u16;

    #[inline(always)]
    fn add(self, rhs: u16) -> u16 {
        self.0 + rhs
    }
}

pub struct SerialConnection<IO: PortIo> {
    com: IoPort,
    pub io: IO,
}

impl SerialConnection<RealPortIo> {
    pub fn new(com: IoPort) -> Self {
        Self::new_with_io(com, RealPortIo)
    }
}

impl<IO: PortIo> SerialConnection<IO> {
    pub fn new_with_io(com: IoPort, io: IO) -> Self {
        let s = Self { com, io };
        s.init();
        s
    }

    fn init(&self) {
        unsafe {
            self.io.outb(self.com + 1, 0x00); // IER: disable interrupts (polling)
            self.io.outb(self.com + 3, 0x80); // LCR: enable DLAB
            self.io.outb(self.com + 0, 0x03); // DLL: divisor low (3) -> 38400
            self.io.outb(self.com + 1, 0x00); // DLM
            self.io.outb(self.com + 3, 0x03); // LCR: 8N1, DLAB=0
            self.io.outb(self.com + 2, 0xC7); // FCR
            self.io.outb(self.com + 4, 0x0B); // MCR: DTR|RTS|OUT2
        }
    }

    #[inline(always)]
    fn tx_ready(&self) -> bool {
        unsafe { (self.io.inb(self.com + 5) & 0x20) != 0 } // LSR bit 5: THR empty
    }

    pub fn write_byte(&self, b: u8) {
        while !self.tx_ready() {
            core::hint::spin_loop();
        }
        unsafe { self.io.outb(self.com + 0, b) }
    }

    pub fn write_str(&self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }
}

impl<IO: PortIo> fmt::Write for SerialConnection<IO> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        SerialConnection::write_str(self, s);
        Ok(())
    }
}
