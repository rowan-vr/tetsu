#![no_main]
#![no_std]

mod gop;
mod arch;
mod kernel;
mod handoff;

use log::info;
use uefi::{CStr16};
use uefi::prelude::*;

const KERNEL_PATH: &CStr16 = cstr16!(r"\kernel.bin");
const KERNEL_LOAD_ADDR: u64 = 0x0010_0000; // 1 MiB

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("[tetsu-boot] Started bootloader...");

    match kernel::load_kernel(KERNEL_PATH, KERNEL_LOAD_ADDR) {
        Ok(()) => info!("[tetsu-boot] loaded kernel successfully!"),
        Err(status) => return status,
    };

    info!("[tetsu-boot] loaded kernel @ {:#x}", KERNEL_LOAD_ADDR);

    let (_,stack_top)  = arch::x64_86::alloc_stack_pages(64);

    info!("[tetsu-boot] stack initialised");

    let fb = gop::capture_framebuffer_info().expect("[tetsu-boot] Failed to get framebuffer info");

    info!("[tetsu-boot] framebuffer initialised");

    info!("[tetsu-boot] Jumping to kernel...");

    handoff::handoff(stack_top, fb);
}
