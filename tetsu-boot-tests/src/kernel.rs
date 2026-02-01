use uefi::{cstr16, CStr16};
use tetsu_boot::kernel;
use tetsu_tests::check;

#[test_case]
fn test_load_kernel_happy_path() -> Result<(), ()> {
    let path = cstr16!(r"\kernel.bin");

    let addr: u64 = 0x0010_0000;

    kernel::load_kernel(&path, addr).map_err(|_| ())?;

    // Verify memory at addr is not all-zero (very weak check, but simple)
    let buf = unsafe { core::slice::from_raw_parts(addr as *const u8, 16) };
    let mut any_nonzero = false;
    for &b in buf {
        if b != 0 { any_nonzero = true; break; }
    }
    if !any_nonzero { return Err(()); }

    Ok(())
}

#[test_case]
fn test_load_kernel_missing_file_fails() -> Result<(), ()> {
    let path = cstr16!(r"\missing.bin");

    let addr: u64 = 0x0010_0000;
    
    check!(kernel::load_kernel(&path, addr).is_err());

    Ok(())
}
