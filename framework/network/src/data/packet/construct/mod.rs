#![allow(warnings)] // TODO: remove this and fix warnings
/**
 * This module holds useful helpers such as packet constructors
 * that define structure using higher level types
 * and map them to low level buffers.
 */

use heapless::Vec;
// TODO: tests
// TODO: when implementations are finished, change names to more suitable and consistent formats

// TODO: we should implement priority for CAN bus somewhere -> either by specifying it here or by match elsewhere -> preferably here

// TODO: since this sucker doesn't have much on it we should use some data type either from `wire` or `interface` module
// so that we don't have to map data one more time

pub struct PacketPayload {
    pub payload: Vec<u8, 8>,
}

mod construct {
    use vlcb_defs::CbusOpCodes;
    use heapless::Vec;

    use super::PacketPayload;

    #[inline(never)]
    #[cold]
    #[track_caller]
    pub(super) fn len_mismatch_fail(payload_len: usize, lte: usize) -> ! {
        panic!(
            "payload slice length ({}) is greater than ({})",
            payload_len, lte,
        );
    }

    #[inline]
    pub(super) fn new(data: &[u8]) -> PacketPayload {
        debug_assert!(data.len() < 9, "payload slice cannot be larger than 8 octets, given ({})", data.len());

        PacketPayload {
            payload: Vec::from_slice(data).unwrap()
        }
    }

    #[inline]
    pub(super) fn no_data(opcode: CbusOpCodes) -> PacketPayload {
        new(&[opcode.into()])
    }

    #[inline]
    pub(super) fn one_byte(opcode: CbusOpCodes, a0: u8) -> PacketPayload {
        new(&[opcode.into(), a0])
    }

    #[inline]
    pub(super) fn two_bytes(opcode: CbusOpCodes, a0: u8, a1: u8) -> PacketPayload {
        new(&[opcode.into(), a0, a1])
    }

    #[inline]
    pub(super) fn three_bytes(opcode: CbusOpCodes, a0: u8, a1: u8, a2: u8) -> PacketPayload {
        new(&[opcode.into(), a0, a1, a2])
    }

    #[inline]
    pub(super) fn four_bytes(opcode: CbusOpCodes, a0: u8, a1: u8, a2: u8, a3: u8) -> PacketPayload {
        new(&[opcode.into(), a0, a1, a2, a3])
    }

    #[inline]
    pub(super) fn five_bytes(opcode: CbusOpCodes, a0: u8, a1: u8, a2: u8, a3: u8, a4: u8) -> PacketPayload {
        new(&[opcode.into(), a0, a1, a2, a3, a4])
    }

    #[inline]
    pub(super) fn six_bytes(opcode: CbusOpCodes, a0: u8, a1: u8, a2: u8, a3: u8, a4: u8, a5: u8) -> PacketPayload {
        new(&[opcode.into(), a0, a1, a2, a3, a4, a5])
    }

    #[inline]
    pub(super) fn seven_bytes(opcode: CbusOpCodes, a0: u8, a1: u8, a2: u8, a3: u8, a4: u8, a5: u8, a6: u8) -> PacketPayload {
        new(&[opcode.into(), a0, a1, a2, a3, a4, a5, a6])
    }
}

pub mod bus_ctrl;
pub mod loco_ctrl;
pub mod module_cfg;
pub mod layout_ctrl;
pub mod ext;