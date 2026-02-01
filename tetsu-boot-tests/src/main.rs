#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(tetsu_tests::runner::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(unused_imports)]
#![allow(unreachable_code)]

mod gop;
mod kernel;
mod trivial;

#[cfg(target_os = "uefi")]
use core::panic::PanicInfo;
#[cfg(target_os = "uefi")]
use tetsu_tests::*;
use uefi::{Status, entry};

#[cfg(not(test))]
fn test_main() -> ! {
    panic!("Test not implemented");
}

#[entry]
fn entry() -> Status {
    test_main();

    panic!("Test main ended without closure");
}

#[cfg(target_os = "uefi")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    qemu::serial_write_str("[FATAL]\n");
    let _ = info;

    qemu::qemu_exit_fail()
}
