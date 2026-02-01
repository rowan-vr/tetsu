use crate::qemu::{serial_write_str, BOLD, GREEN, RED, RESET};

pub trait Testable {
    fn run(&self);
}

pub trait IntoTestResult {
    fn into_result(self) -> Result<(), ()>;
}

impl IntoTestResult for () {
    fn into_result(self) -> Result<(), ()> { Ok(()) }
}

impl IntoTestResult for Result<(), ()> {
    fn into_result(self) -> Result<(), ()> { self }
}

impl<T, R> Testable for T
where
    T: Fn() -> R,
    R: IntoTestResult,
{
    fn run(&self) {
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
            }
            Err(msg) => {
                serial_write_str(RED);
                serial_write_str("FAIL");
                serial_write_str(RESET);
                serial_write_str("\n");
            }
        }
    }
}
