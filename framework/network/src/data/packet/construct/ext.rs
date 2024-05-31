use crate::wire::VLCB_MAX_PAYLOAD;

/// Opcode extensions
///
/// These can be used in times when basic 32 opcodes are not enough.
/// This extension supports additional 256 opcode values, these have no formal definition
/// as of yet and support up to 6 bytes of data.

use super::{construct, PacketPayload};
use vlcb_defs::OpCode;
use heapless::Vec;

/// Construct a packet with extended opcode and a payload
///
/// # Panics
/// This method panics if the payload is over 6 octets long
pub fn from_bytes(opcode_ext: u8, payload: &[u8]) -> PacketPayload {
    let len = payload.len();
    if len > 6 {
        construct::len_mismatch_fail(len, 6);
    }

    let opc = match len {
        0 => OpCode::ExtOpCode,
        1 => OpCode::ExtOpCode1,
        2 => OpCode::ExtOpCode2,
        3 => OpCode::ExtOpCode3,
        4 => OpCode::ExtOpCode4,
        5 => OpCode::ExtOpCode5,
        6 => OpCode::ExtOpCode6,
        _ => unreachable!(),
    };

    // TODO: maybe use unchecked because we know the size
    let mut buf: Vec<u8, VLCB_MAX_PAYLOAD> = Vec::new();
    buf.push(opc as u8);
    buf.push(opcode_ext);
    buf.extend_from_slice(payload);
    construct::from_bytes(buf.as_slice())
}

/// Constructs a packet with extended opcode and no payload
pub fn no_data(opcode_ext: u8) -> PacketPayload {
    construct::one_byte(OpCode::ExtOpCode, opcode_ext)
}
