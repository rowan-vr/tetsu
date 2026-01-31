#![no_std]

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FramebufferInfo {
    pub base: u64,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: u32,
    pub bpp: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BootInfo {
    pub framebuffer: FramebufferInfo,
}