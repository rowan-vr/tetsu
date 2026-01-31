use uefi::boot;
use tetsu_abi::{BootInfo, FramebufferInfo};
use crate::{arch, KERNEL_LOAD_ADDR};

pub(crate) fn handoff(stack_top: u64, framebuffer_info: FramebufferInfo) -> ! {
    let _mmap = unsafe { boot::exit_boot_services(None) };

    let bootinfo = BootInfo {
        framebuffer: framebuffer_info,
    };

    unsafe { arch::x64_86::jump_kernel(KERNEL_LOAD_ADDR as usize, stack_top, &bootinfo) }
}