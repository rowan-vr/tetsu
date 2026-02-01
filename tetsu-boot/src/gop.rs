use uefi::{boot, Error};
use uefi::proto::console::gop::GraphicsOutput;
use tetsu_abi::FramebufferInfo;

pub fn capture_framebuffer_info() -> Result<FramebufferInfo, Error> {
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
