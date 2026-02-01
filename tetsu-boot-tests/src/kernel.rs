use tetsu_boot::kernel;
use tetsu_tests::check;
use uefi::{CStr16, cstr16};

#[test_case]
fn test_load_kernel_happy_path() -> Result<(), ()> {
    let path = cstr16!(r"\kernel.bin");

    let addr: u64 = 0x0010_0000;

    kernel::load_kernel(&path, addr).map_err(|_| ())?;

    // Verify memory at addr is not all-zero (very weak check, but simple)
    let buf = unsafe { core::slice::from_raw_parts(addr as *const u8, 16) };
    let mut any_nonzero = false;
    for &b in buf {
        if b != 0 {
            any_nonzero = true;
            break;
        }
    }
    if !any_nonzero {
        return Err(());
    }

    Ok(())
}

#[test_case]
fn test_load_kernel_missing_file_fails() -> Result<(), ()> {
    let path = cstr16!(r"\missing.bin");

    let addr: u64 = 0x0010_0000;

    check!(kernel::load_kernel(&path, addr).is_err());

    Ok(())
}

#[test_case]
fn test_alloc_stack_pages_invariants() -> Result<(), ()> {
    let pages = 8;
    let (base, top) = tetsu_boot::arch::x64_86::alloc_stack_pages(pages);

    check!(base != 0);
    check!((base & 0xFFF) == 0); // page aligned
    check!((top & 0xF) == 0); // 16-byte aligned
    check!(top > base);

    let span = top - base;
    check!(span >= (pages as u64) * 4096);

    // Touch first and last byte of allocated region (careful with "top" alignment)
    unsafe {
        let first = base as *mut u8;
        *first = 0xAA;

        // last usable byte within allocated pages
        let last = (base + (pages as u64 * 4096) - 1) as *mut u8;
        *last = 0x55;
    }

    Ok(())
}
