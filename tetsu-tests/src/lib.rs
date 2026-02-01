#![no_std]

pub mod qemu;
pub mod runner;
pub mod testable;

/// Fail the current test with a message if condition is false.
#[macro_export]
macro_rules! check {
    ($cond:expr) => {
        if !($cond) {
            return Err("check failed");
        }
    };
    ($cond:expr, $msg:expr) => {
        if !($cond) {
            return Err(stringify!($msg));
        }
    };
}

/// Fail the current test with a message if two expressions are not equal.
#[macro_export]
macro_rules! check_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return Err("check_eq failed");
        }
    };
    ($left:expr, $right:expr, $msg:expr) => {
        if $left != $right {
            return Err(stringify!($msg));
        }
    };
}
