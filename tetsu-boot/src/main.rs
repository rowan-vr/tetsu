#![no_main]
#![no_std]

use core::arch::asm;
use log::info;
use uefi::boot::{AllocateType, MemoryType};
use uefi::{CStr16, Error};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;
use tetsu_abi::{BootInfo, FramebufferInfo};

const KERNEL_PATH: &CStr16 = cstr16!(r"\kernel.bin");
const KERNEL_LOAD_ADDR: u64 = 0x0010_0000; // 1 MiB

fn capture_framebuffer_info() -> Result<FramebufferInfo, Error> {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let (width, height) = gop.current_mode_info().resolution();
    let pixel_format = gop.current_mode_info().pixel_format();
    let stride = gop.current_mode_info().stride();

    let mut fb = gop.frame_buffer();
    let size = fb.size();
    let base = fb.as_mut_ptr() as u64;

    Ok(FramebufferInfo {
        base,
        height: height as u32,
        width: width as u32,
        size: size as u64,
        format: pixel_format as u32,
        stride: stride as u32,
        bpp: 4
    })
}


fn alloc_stack_pages(pages: usize) -> u64 {
    let addr = boot::allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)
        .expect("[tetsu-boot] alloc stack pages failed");
    addr.as_ptr() as u64
}

unsafe fn jump_kernel(entry_addr: usize, stack_top: u64, boot_info_ptr: *const BootInfo) -> ! {
    unsafe {asm!("cli", options(nomem, nostack, preserves_flags)) };

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

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("[tetsu-boot] Started bootloader...");

    let image = boot::image_handle();

    let loaded = boot::open_protocol_exclusive::<LoadedImage>(image)
        .expect("[tetsu-boot] LoadedImage failed to load");
    let device = loaded.device().expect("[tetsu-boot] LoadedImage has no device");

    let mut sfs = boot::open_protocol_exclusive::<SimpleFileSystem>(device)
        .expect("[tetsu-boot] SimpleFileSystem failed to open");
    let mut root = sfs.open_volume().expect("[tetsu-boot] Failed to open volume");

    let file_handle = root
        .open(KERNEL_PATH, FileMode::Read, FileAttribute::empty())
        .expect("[tetsu-boot] Failed to open kernel file");

    let mut kernel: RegularFile = match file_handle.into_type().expect("[tetsu-boot] into_type failed") {
        FileType::Regular(f) => f,
        _ => return Status::LOAD_ERROR,
    };

    let mut info_buf = [0u8; 512];
    let info = kernel
        .get_info::<FileInfo>(&mut info_buf)
        .expect("[tetsu-boot] get FileInfo failed");
    let size = info.file_size() as usize;

    info!("[tetsu-boot] kernel size: {}", size);

    let pages = (size + 0xFFF) / 0x1000;
    boot::allocate_pages(
        AllocateType::Address(KERNEL_LOAD_ADDR),
        MemoryType::LOADER_DATA,
        pages,
    )
        .expect("[tetsu-boot] allocate_pages failed");

    let dst = unsafe {
        core::slice::from_raw_parts_mut(KERNEL_LOAD_ADDR as *mut u8, size)
    };

    let read = kernel.read(dst).expect("[tetsu-boot] read failed");
    if read != size {
        info!("[tetsu-boot] short read: {} / {}", read, size);
        return Status::LOAD_ERROR;
    }

    info!("[tetsu-boot] loaded kernel @ {:#x}", KERNEL_LOAD_ADDR);

    let stack_pages = 64;
    let stack_base = alloc_stack_pages(stack_pages);
    let stack_top  = (stack_base + (stack_pages as u64 * 4096)) & !0xFu64;

    info!("[tetsu-boot] stack initialised");

    let fb = capture_framebuffer_info().expect("[tetsu-boot] Failed to get framebuffer info");

    info!("[tetsu-boot] framebuffer initialised");

    let _mmap = unsafe { boot::exit_boot_services(None) };

    let bootinfo = BootInfo {
        framebuffer: fb,
    };

    unsafe {jump_kernel(KERNEL_LOAD_ADDR as usize, stack_top, &bootinfo)}
}
