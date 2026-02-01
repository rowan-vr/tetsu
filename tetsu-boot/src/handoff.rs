use uefi::boot;
use tetsu_abi::{BootInfo, FramebufferInfo};
use crate::arch;

pub fn handoff(kernel_addr: usize, stack_top: u64, framebuffer_info: FramebufferInfo) -> ! {
    let _mmap = unsafe { boot::exit_boot_services(None) };

    let bootinfo = BootInfo {
        framebuffer: framebuffer_info,
    };

    arch::x64_86::jump_kernel(kernel_addr, stack_top, &bootinfo)
}