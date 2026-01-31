#![no_std]
#![no_main]

use core::panic::PanicInfo;
use tetsu_abi::{BootInfo, FramebufferInfo};

// static HELLO: &[u8] = b"Hello World!";

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "C" fn _start(boot_info_ptr: *const BootInfo) -> ! {
    let boot_info = unsafe { &*boot_info_ptr };

    clear_screen(&boot_info.framebuffer, 0x00FF0000);

    loop {}
}

fn clear_screen(fb: &FramebufferInfo, color: u32) {
    let base = fb.base as *mut u32;
    let stride = fb.stride as usize;
    let width = fb.width as usize;
    let height = fb.height as usize;

    for y in 0..height {
        for x in 0..width {
            let idx = y * stride + x;
            unsafe {
                base.add(idx).write_volatile(color);
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
