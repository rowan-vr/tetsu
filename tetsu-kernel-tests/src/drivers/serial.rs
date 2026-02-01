use core::cell::UnsafeCell;
use core::fmt::Write;
use tetsu_kernel::drivers::serial::{IoPort, PortIo, SerialConnection};
use tetsu_tests::qemu::{serial_write_byte, serial_write_str};
use tetsu_tests::{check, check_eq};

const MAX_WRITES: usize = 512;
const MAX_READS: usize = 512;

#[derive(Copy, Clone)]
struct WriteOp {
    port: u16,
    val: u8,
}

#[derive(Copy, Clone)]
struct ReadOp {
    port: u16,
    val: u8,
}

struct MockPortIo {
    writes: [WriteOp; MAX_WRITES],
    writes_len: usize,

    reads: [ReadOp; MAX_READS],
    reads_len: usize,
    reads_idx: usize,
}

impl MockPortIo {
    const fn new() -> Self {
        const W: WriteOp = WriteOp { port: 0, val: 0 };
        const R: ReadOp = ReadOp { port: 0, val: 0 };
        Self {
            writes: [W; MAX_WRITES],
            writes_len: 0,
            reads: [R; MAX_READS],
            reads_len: 0,
            reads_idx: 0,
        }
    }

    fn push_read(&mut self, port: u16, val: u8) {
        // In tests, you usually want to fail hard if you exceed capacity
        assert!(self.reads_len < MAX_READS);
        self.reads[self.reads_len] = ReadOp { port, val };
        self.reads_len += 1;
    }

    fn record_write(&mut self, port: u16, val: u8) {
        assert!(self.writes_len < MAX_WRITES);
        self.writes[self.writes_len] = WriteOp { port, val };
        self.writes_len += 1;
    }

    fn next_read(&mut self, port: u16) -> u8 {
        if self.reads_idx >= self.reads_len {
            // Default to "not ready" if not scripted.
            return 0x00;
        }
        let r = self.reads[self.reads_idx];
        self.reads_idx += 1;

        // Be strict: ensure the driver read from the port we expected
        assert!(r.port == port);
        r.val
    }
}

// Interior mutability wrapper so we can share &self in driver calls
struct MockIoCell(UnsafeCell<MockPortIo>);
unsafe impl Sync for MockIoCell {}

impl MockIoCell {
    fn new() -> Self {
        Self(UnsafeCell::new(MockPortIo::new()))
    }

    fn push_read(&self, port: u16, val: u8) {
        let m = unsafe { &mut *self.0.get() };
        m.push_read(port, val);
    }

    fn writes_len(&self) -> usize {
        let m = unsafe { &*self.0.get() };
        m.writes_len
    }

    fn write_at(&self, idx: usize) -> (u16, u8) {
        let m = unsafe { &*self.0.get() };
        assert!(idx < m.writes_len);
        (m.writes[idx].port, m.writes[idx].val)
    }

    fn clear_writes(&self) {
        let m = unsafe { &mut *self.0.get() };
        m.writes_len = 0;
    }

    fn count_data_writes(&self, data_port: u16, byte: u8) -> usize {
        let m = unsafe { &*self.0.get() };
        let mut c = 0;
        let mut i = 0;
        while i < m.writes_len {
            let w = m.writes[i];
            if w.port == data_port && w.val == byte {
                c += 1;
            }
            i += 1;
        }
        c
    }

    fn collect_data_bytes(&self, data_port: u16, out: &mut [u8]) -> usize {
        let m = unsafe { &*self.0.get() };
        let mut n = 0usize;
        let mut i = 0usize;
        while i < m.writes_len && n < out.len() {
            let w = m.writes[i];
            if w.port == data_port {
                out[n] = w.val;
                n += 1;
            }
            i += 1;
        }
        n
    }
}

impl PortIo for MockIoCell {
    unsafe fn outb(&self, port: u16, val: u8) {
        let m = unsafe { &mut *self.0.get() };
        m.record_write(port, val);
    }

    unsafe fn inb(&self, port: u16) -> u8 {
        let m = unsafe { &mut *self.0.get() };
        m.next_read(port)
    }
}

#[test_case]
fn ioport_addr_and_add() -> Result<(), &'static str> {
    check_eq!(IoPort::COM1.addr(), 0x3F8);
    check_eq!(IoPort::COM2.addr(), 0x2F8);
    check_eq!(IoPort::COM1 + 5, 0x3F8 + 5);
    Ok(())
}

#[test_case]
fn init_programs_expected_register_sequence() -> Result<(), &'static str> {
    let io = MockIoCell::new();
    let _s = SerialConnection::new_with_io(IoPort::COM1, io);

    let base = IoPort::COM1.addr();
    let expected: [(u16, u8); 7] = [
        (base + 1, 0x00),
        (base + 3, 0x80),
        (base + 0, 0x03),
        (base + 1, 0x00),
        (base + 3, 0x03),
        (base + 2, 0xC7),
        (base + 4, 0x0B),
    ];

    check_eq!(_s.io.writes_len(), expected.len());

    let mut i = 0usize;
    while i < expected.len() {
        let (p, v) = _s.io.write_at(i);
        check_eq!(p, expected[i].0);
        check_eq!(v, expected[i].1);
        i += 1;
    }

    Ok(())
}

#[test_case]
fn write_str_writes_exact_bytes_no_crlf_translation() -> Result<(), &'static str> {
    let io = MockIoCell::new();
    let base = IoPort::COM1.addr();
    let lsr_port = base + 5;
    let data_port = base + 0;

    let s = SerialConnection::new_with_io(IoPort::COM1, io);

    // Drop all init() writes (including DLL at base+0).
    s.io.clear_writes();

    // Script "ready" for each byte we’ll send: "test\n" = 5 bytes.
    s.io.push_read(lsr_port, 0x20);
    s.io.push_read(lsr_port, 0x20);
    s.io.push_read(lsr_port, 0x20);
    s.io.push_read(lsr_port, 0x20);
    s.io.push_read(lsr_port, 0x20);

    s.write_str("test\n");

    let mut got = [0u8; 8];
    let n = s.io.collect_data_bytes(data_port, &mut got);

    check_eq!(n, 5);
    check_eq!(got[0], b't');
    check_eq!(got[1], b'e');
    check_eq!(got[2], b's');
    check_eq!(got[3], b't');
    check_eq!(got[4], b'\n');

    Ok(())
}

#[test_case]
fn write_byte_waits_until_ready_then_writes() -> Result<(), &'static str> {
    let io = MockIoCell::new();
    let base = IoPort::COM1.addr();

    // Not ready twice, then ready
    io.push_read(base + 5, 0x00);
    io.push_read(base + 5, 0x00);
    io.push_read(base + 5, 0x20);

    let s = SerialConnection::new_with_io(IoPort::COM1, io);
    s.write_byte(b'X');

    // Exactly one 'X' should be written to DATA port
    check_eq!(s.io.count_data_writes(base + 0, b'X'), 1);

    Ok(())
}
