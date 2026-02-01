use tetsu_boot::gop::capture_framebuffer_info;
use tetsu_tests::check;

#[test_case]
fn test_capture_framebuffer_info_sane() -> Result<(), ()> {
    let fb = capture_framebuffer_info().map_err(|_| ())?;

    check!(fb.base != 0);
    check!(fb.width != 0);
    check!(fb.height != 0);
    check!(fb.stride >= fb.width);

    let min = (fb.stride as u64) * (fb.height as u64) * 4;
    check!(fb.size >= min);

    Ok(())
}
