#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(unsafe_code)]

#[cfg(test)]
extern crate alloc;

pub mod macros;
pub mod service;
pub mod can;
pub mod cbus;
pub mod dcc;
pub mod fast_clock;
pub mod module;