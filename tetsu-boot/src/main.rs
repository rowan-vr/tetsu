#![no_main]
#![no_std]

use log::info;
use uefi::boot::{AllocateType, MemoryType};
use uefi::CStr16;
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;

const KERNEL_PATH: &CStr16 = cstr16!(r"\kernel.bin");
const KERNEL_LOAD_ADDR: u64 = 0x0010_0000; // 1 MiB

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

    let mut kernel: RegularFile = match file_handle.into_type().expect("[BL] into_type failed") {
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

    info!("[tetsu-boot] loaded kernel @ {:#x}, jumping", KERNEL_LOAD_ADDR);

    let entry: extern "C" fn() -> ! = unsafe { core::mem::transmute(KERNEL_LOAD_ADDR as usize) };
    entry();
}
