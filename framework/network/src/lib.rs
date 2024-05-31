#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(unsafe_code)]

#[cfg(any(test, feature = "alloc"))]
extern crate alloc;

#[macro_use]
mod macros;

pub mod config {
    // TODO: make this configurable
    #![allow(unused)]
    pub const CAN_RESERVE_DELAY_MS: u64 = 100;
    pub const CAN_DEFAULT_PRIORITY: u8 = 0xB;
    pub const LONG_MESSAGE_DEFAULT_DELAY: u16 = 20;
    pub const LONG_MESSAGE_RECEIVE_TIMEOUT: u16 = 5000;
}

pub mod phy;
pub mod wire;

pub mod iface;

pub mod socket;

pub mod storage;

pub mod data;