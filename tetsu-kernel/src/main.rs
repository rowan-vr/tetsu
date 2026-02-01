#![no_std]
#![no_main]

mod drivers;

use core::panic::PanicInfo;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};
use tetsu_abi::{BootInfo, FramebufferInfo};

// static HELLO: &[u8] = b"Hello World!";

static BOOT_INFO_PTR: AtomicPtr<BootInfo> = AtomicPtr::new(ptr::null_mut());

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "C" fn _start(boot_info_ptr: *const BootInfo) -> ! {
    BOOT_INFO_PTR.store(boot_info_ptr as *mut BootInfo, Ordering::Release);

    clear_screen(&boot_info().unwrap().framebuffer, 0x00FF0000);

    #[allow(clippy::empty_loop)]
    loop {}
}

pub fn boot_info() -> Result<&'static BootInfo, &'static str> {
    let p = BOOT_INFO_PTR.load(Ordering::Acquire);
    unsafe { Ok(&*p) }
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
