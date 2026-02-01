#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(tetsu_tests::runner::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(unused_imports)]
#![allow(unreachable_code)]

mod trivial;

use core::panic::PanicInfo;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use tetsu_abi::BootInfo;
use tetsu_tests::*;

static BOOT_INFO_PTR: AtomicPtr<BootInfo> = AtomicPtr::new(ptr::null_mut());

#[cfg(not(test))]
fn test_main() -> ! {
    panic!("Test not implemented");
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "C" fn _start(boot_info_ptr: *const BootInfo) -> ! {
    BOOT_INFO_PTR.store(boot_info_ptr as *mut BootInfo, Ordering::Release);

    test_main();

    panic!("Test main ended without closure");
}

pub fn boot_info() -> Result<&'static BootInfo, &'static str> {
    let p = BOOT_INFO_PTR.load(Ordering::Acquire);
    check!(!p.is_null(), "BOOT_INFO_PTR not initialized");
    unsafe { Ok(&*p) }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    qemu::serial_write_str("[FATAL]\n");
    let _ = info;

    qemu::qemu_exit_fail()
}
