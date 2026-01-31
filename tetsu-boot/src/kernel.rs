use log::info;
use uefi::{boot, CStr16, Status};
use uefi::boot::{AllocateType, MemoryType};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType, RegularFile};
use uefi::proto::media::fs::SimpleFileSystem;

pub(crate) fn load_kernel(kernel_path: &CStr16, kernel_addr: u64) -> Result<(), Status>{
    let image = boot::image_handle();

    let loaded = boot::open_protocol_exclusive::<LoadedImage>(image)
        .expect("[tetsu-boot] failed to open image");
    let device = loaded.device().expect("[tetsu-boot] LoadedImage has no device");

    let mut sfs = boot::open_protocol_exclusive::<SimpleFileSystem>(device)
        .expect("[tetsu-boot] failed to open image");
    let mut root = sfs.open_volume()
        .expect("[tetsu-boot] failed to open volume");

    let file_handle = root
        .open(kernel_path, FileMode::Read, FileAttribute::empty())
        .expect("[tetsu-boot] failed to open kernel");

    let mut kernel: RegularFile = match file_handle.into_type().expect("[tetsu-boot] into_type failed") {
        FileType::Regular(f) => f,
        _ => return Err(Status::LOAD_ERROR),
    };

    let mut info_buf = [0u8; 512];
    let info = kernel
        .get_info::<FileInfo>(&mut info_buf)
        .expect("[tetsu-boot] get FileInfo failed");
    let size = info.file_size() as usize;

    info!("[tetsu-boot] kernel size: {}", size);

    let pages = (size + 0xFFF) / 0x1000;
    boot::allocate_pages(
        AllocateType::Address(kernel_addr),
        MemoryType::LOADER_DATA,
        pages,
    )
        .expect("[tetsu-boot] allocate_pages failed");

    let dst = unsafe {
        core::slice::from_raw_parts_mut(kernel_addr as *mut u8, size)
    };

    let read = kernel.read(dst).expect("[tetsu-boot] read failed");
    if read != size {
        info!("[tetsu-boot] short read: {} / {}", read, size);
        return Err(Status::LOAD_ERROR)
    }
    
    Ok(())
}