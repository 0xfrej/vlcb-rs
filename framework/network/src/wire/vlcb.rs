use super::{Error, Result};
use vlcb_defs::CbusOpCodes;
use core::fmt;

/// VLCB sub-protocol.
///
/// VLCB doesn't define any "sub-protocols", but this library
/// uses them to separate messages to separate socket types
/// for easier handling.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Protocol {
    // Every other opcode that is not tied to a specific protocol
    Module,

    /// Long message protocol
    LongMsg,
}

//TODO: we need to properly test this and check for data_len constraints

/// Size of an VLCB address in octets. (The address is 11bit wide)
pub const ADDR_SIZE: usize = 2;

/// Max size of a VLCB packet in octets
pub const VLCB_MAX_PAYLOAD: usize = 8;

/// A two-octet VLCB address (11 bit).
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct Address(pub [u8; ADDR_SIZE]);

impl Address {
    /// Construct an VLCB address from parts.
    pub const fn new(a0: u8, a1: u8) -> Address {
        Address([a0, a1])
    }

    /// Construct an VLCB address from a sequence of octets, in big-endian.
    ///
    /// # Panics
    /// The function panics if `data` is not two octets long.
    pub fn from_bytes(data: &[u8]) -> Address {
        let mut bytes = [0; ADDR_SIZE];
        bytes.copy_from_slice(data);
        Address(bytes)
    }

    /// Return an CBUS address as a sequence of octets, in big-endian.
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.0;
        write!(f, "{:02X}.{:02X}", bytes[0], bytes[1])
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Address {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{=u8:X}.{=u8:X}", self.0[0], self.0[1])
    }
}

/// A read/write wrapper around an VLCB packet buffer.
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    use crate::wire::field::*;

    pub const OPCODE: Single = 0;
    pub const OPCODE_MASK: u8 = 0x1F;
    pub const DATA_LEN: Single = 0;
    pub const DATA_LEN_MASK: u8 = 0xE0;
}

pub const HEADER_LEN: usize = 1;

impl<T: AsRef<[u8]>> Packet<T> {
    /// Imbue a raw octet buffer with VLCB packet structure.
    pub const fn new_unchecked(buffer: T) -> Packet<T> {
        Packet { buffer }
    }

    /// Shorthand for a combination of [new_unchecked] and [check_len].
    ///
    /// [new_unchecked]: #method.new_unchecked
    /// [check_len]: #method.check_len
    pub fn new_checked(buffer: T) -> Result<Packet<T>> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        Ok(packet)
    }

    /// Ensure that no accessor method will panic if called.
    /// Returns `Err(Error)` if the buffer is too short.
    /// Returns `Err(Error)` if the header length is greater
    /// than total length.
    ///
    /// The result of this check is invalidated by calling [set_header_len]
    /// and [set_total_len].
    ///
    /// [set_header_len]: #method.set_header_len
    /// [set_total_len]: #method.set_total_len
    #[allow(clippy::if_same_then_else)]
    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < self.header_len() as usize {
            Err(Error)
        } else if self.header_len() > self.total_len() {
            Err(Error)
        } else if len < self.total_len() as usize {
            Err(Error)
        } else {
            Ok(())
        }
    }

    /// Consume the packet, returning the underlying buffer.
    pub fn into_inner(self) -> T {
        self.buffer
    }

    /// Return the header length, in octets.
    #[inline]
    pub fn header_len(&self) -> u8 {
        HEADER_LEN as u8
    }

    /// Return the total length field.
    #[inline]
    pub fn total_len(&self) -> u8 {
        self.payload_len()
    }

    /// Return the VLCB OpCode
    #[inline]
    pub fn opcode(&self) -> u8 {
        self.buffer.as_ref()[field::OPCODE] & field::OPCODE_MASK
    }

    /// Return the payload len for current OpCode
    #[inline]
    pub fn payload_len(&self) -> u8 {
        (self.buffer.as_ref()[field::DATA_LEN] & field::DATA_LEN_MASK) >> 5
    }

    /// Return the next header protocol type
    pub fn next_header(&self) -> Protocol {
        match CbusOpCodes::from(self.opcode()) {
            CbusOpCodes::DTXC => Protocol::LongMsg,
            _ => Protocol::Module,
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Packet<&'a T> {
    /// Return a pointer to the payload.
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        let range = self.header_len() as usize..self.total_len() as usize;
        let data = self.buffer.as_ref();
        &data[range]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Packet<T> {
    #[inline]
    pub fn set_opcode(&mut self, value: u8) {
        let data = self.buffer.as_ref()[field::OPCODE] & !field::OPCODE_MASK;
        self.buffer.as_mut()[field::OPCODE] = data | (value & field::OPCODE_MASK);
    }

    #[inline]
    pub fn set_payload_len(&mut self, value: u8) {
        let data = self.buffer.as_ref()[field::DATA_LEN] & !field::DATA_LEN_MASK;
        self.buffer.as_mut()[field::DATA_LEN] = data | ((value << 5) & field::DATA_LEN_MASK);
    }

    /// Return a mutable pointer to the payload.
    #[inline]
    pub fn payload_mut(&mut self) -> &mut [u8] {
        let range = self.header_len() as usize..self.total_len() as usize;
        let data = self.buffer.as_mut();
        &mut data[range]
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Packet<T> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

/// A high-level representation of an VLCB packet header.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Repr {
    pub data_len: u8,
    pub opcode: CbusOpCodes,
    pub next_header: Protocol,
}

impl Repr {
    pub fn new(opcode: CbusOpCodes, data_len: u8, next_header: Protocol) -> Self {
        Self { opcode, data_len, next_header }
    }

    /// Parse an VLCB packet and return a high-level representation.
    pub fn parse<T: AsRef<[u8]> + ?Sized>(packet: &Packet<&T>) -> Result<Repr> {
        Ok(Repr {
            data_len: packet.payload_len(),
            opcode: CbusOpCodes::try_from(packet.opcode()).unwrap(),
            next_header: packet.next_header(),
        })
    }

    /// Return the length of a header that will be emitted from this high-level representation.
    pub const fn header_len(&self) -> usize {
        HEADER_LEN
    }

    /// Emit a high-level representation into an VLCB packet.
    pub fn emit<T: AsRef<[u8]> + AsMut<[u8]>>(
        &self,
        packet: &mut Packet<T>,
        emit_payload: impl FnOnce(&mut [u8]),
    ) {
        packet.set_opcode(self.opcode.into());
        packet.set_payload_len(self.data_len);
        emit_payload(packet.payload_mut());
    }

    /// Return the next header protocol type
    pub fn next_header(&self) -> Protocol {
        self.next_header
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> fmt::Display for Packet<&'a T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match Repr::parse(self) {
            Ok(repr) => write!(f, "{repr}"),
            Err(err) => {
                write!(f, "VLCB ({err})")?;
                write!(
                    f,
                    " opcode={} total_len={}",
                    self.opcode(),
                    self.total_len()
                )?;
                Ok(())
            }
        }
    }
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "VLCB opcode={:?} data_len={:?}",
            self.opcode, self.data_len
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
