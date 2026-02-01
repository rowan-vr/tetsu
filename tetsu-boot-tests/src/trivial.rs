use tetsu_tests::{check, check_eq};

#[test_case]
fn trivial_assertion() -> Result<(), ()>{
    check!(1 == 1);
    Ok(())
}

#[test_case]
fn math_works() -> Result<(), ()>{
    check_eq!(2 + 2, 4);
    Ok(())
}
