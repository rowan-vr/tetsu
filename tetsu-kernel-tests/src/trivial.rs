use tetsu_tests::{check, check_eq};

#[test_case]
fn trivial_assertion() -> Result<(), &'static str> {
    check!(1 == 1);
    Ok(())
}

#[test_case]
fn math_works() -> Result<(), &'static str> {
    check_eq!(2 + 2, 4);
    Ok(())
}
