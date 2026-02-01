use core::arch::asm;
use tetsu_abi::BootInfo;
use uefi::boot;
use uefi::boot::{AllocateType, MemoryType};

pub fn jump_kernel(entry_addr: usize, stack_top: u64, boot_info_ptr: *const BootInfo) -> ! {
    unsafe { asm!("cli", options(nomem, nostack, preserves_flags)) };

    unsafe {
        asm!(
        "mov rdi, {boot_info}",
        "mov rsp, {stack}",
        "jmp {entry}",

        boot_info = in(reg) boot_info_ptr as u64,
        stack = in(reg) stack_top,
        entry = in(reg) entry_addr,
        options(noreturn)
        )
    }
}

pub fn alloc_stack_pages(pages: usize) -> (u64, u64) {
    let addr = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect("[tetsu-boot] alloc stack pages failed");
    let base = addr.as_ptr() as u64;

    (base, (base + (pages as u64 * 4096)) & !0xFu64)
}
