use crate::qemu::{qemu_exit_success, serial_write_number, serial_write_str, BOLD, RESET};
use crate::testable::Testable;

pub fn test_runner(tests: &[&dyn Testable]) -> ! {
    let total = tests.len();
    let mut passed = 0;

    for test in tests {
        test.run();
        passed += 1;
    }

    serial_write_str(BOLD);
    serial_write_str("RESULT: ");
    serial_write_number(passed);
    serial_write_str("/");
    serial_write_number(total as u64);
    serial_write_str(" passed\n");
    serial_write_str(RESET);

    qemu_exit_success();
}
