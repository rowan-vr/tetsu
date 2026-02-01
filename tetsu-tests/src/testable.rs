use crate::qemu::{BOLD, GREEN, RED, RESET, serial_write_str};

pub trait Testable {
    fn run(&self) -> bool;
}

pub trait IntoTestResult {
    fn into_result(self) -> Result<(), &'static str>;
}

impl IntoTestResult for () {
    fn into_result(self) -> Result<(), &'static str> {
        Ok(())
    }
}

impl IntoTestResult for Result<(), &'static str> {
    fn into_result(self) -> Result<(), &'static str> {
        self
    }
}

impl<T, R> Testable for T
where
    T: Fn() -> R,
    R: IntoTestResult,
{
    fn run(&self) -> bool {
        serial_write_str(BOLD);
        serial_write_str("[RUN] ");
        serial_write_str(core::any::type_name::<T>());
        serial_write_str(" ... ");
        match self().into_result() {
            Ok(()) => {
                serial_write_str(GREEN);
                serial_write_str("OK");
                serial_write_str(RESET);
                serial_write_str("\n");
                true
            }
            Err(_msg) => {
                serial_write_str(RED);
                serial_write_str("FAIL");
                serial_write_str(RESET);
                serial_write_str("\n");
                false
            }
        }
    }
}
