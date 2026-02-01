#![no_std]

pub mod testable;
pub mod qemu;
pub mod runner;

/// Fail the current test with a message if condition is false.
#[macro_export]
macro_rules! check {
    ($cond:expr) => {
        if !($cond) {
            return Err(());
        } 
    };
}

/// Fail the current test with a message if two expressions are not equal.
#[macro_export]
macro_rules! check_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return Err(());
        } 
    };
}
