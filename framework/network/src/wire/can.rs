use byteorder::{ByteOrder, NetworkEndian};
use vlcb_core::can::{VlcbCanId, CANID_MASK};
use core::{borrow::BorrowMut, fmt::Debug};
use core::fmt;
use num_enum::{FromPrimitive, IntoPrimitive};

use super::{Error, Result};

/// VLCB CAN frame minor priority.
///
/// Static priority based on message and node type.
/// bits 7 - 8 of the CAN header.
///
/// VLCB unlike CBUS does not support priority ratcheting in case
/// of transport failure
#[derive(Debug, Eq, PartialEq, Copy, Clone, IntoPrimitive, FromPrimitive, Default)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Priority {
    High = 0x00,
    AboveNormal = 0x01,
    Normal = 0x02,
    #[default]
    Low = 0x03,
}

impl Priority {
    pub const MASK: u8 = 0x03;
    pub const MIN: Self = Self::Low;
    pub const MAX: Self = Self::High;
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

/// A read/write wrapper around an CAN frame buffer.
///
/// this buffer is not 1:1 representation of the frame, but
/// consists of 2 octets of standard CAN ID (11 bits) and
/// 8 octets of payload.
///
/// It's important to note that this priority part of the frame header is made from
/// 4 priority bits and 7 address bits. Higher two of priority bits (major priority) never go above
/// binary value of "10" due to the CAN protocol prohibiting a sequence of 7 or more
/// high bits at the start of the header.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Frame<T: AsRef<[u8]>> {
    buffer: T,
}

pub(crate) const HEADER_RTR_MASK: u16 = 0x8000;

mod field {
    use crate::wire::field::*;

    pub(crate) const ID_PRIORITY_MASK: u16 = 0x0780;

    // VLCB uses standard CAN frame with 11-bit identifiers only.
    pub const ID: Field = 0..2;
    pub const ID_CANID: Single = 1;
    pub const PAYLOAD: Rest = 2..;
}

/// The CAN HEADER length
pub const HEADER_LEN: usize = field::PAYLOAD.start;

impl<T: AsRef<[u8]>> Frame<T> {
    /// Construct raw CAN frame without checking anything.
    pub const fn new_unchecked(buffer: T) -> Frame<T> {
        Frame { buffer }
    }

    /// Shorthand for a combination of [new_unchecked], [check_len].
    ///
    /// [new_unchecked]: #method.new_unchecked
    /// [check_len]: #method.check_len
    pub fn new_checked(buffer: T) -> Result<Frame<T>> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        Ok(packet)
    }

    /// Ensure that no accessor method will panic if called.
    /// Returns `Err(Error)` if the buffer is too short.
    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < HEADER_LEN || len - HEADER_LEN > 8 {
            Err(Error)
        } else {
            Ok(())
        }
    }

    /// Consumes the frame, returning the underlying buffer.
    pub fn into_inner(self) -> T {
        self.buffer
    }

    /// Return the length of a frame header.
    pub const fn header_len() -> usize {
        HEADER_LEN
    }

    /// Return the length of a buffer required to hold a packet with the payload
    /// of a given length.
    pub const fn buffer_len(payload_len: usize) -> usize {
        HEADER_LEN + payload_len
    }

    /// Return the source address field.
    #[inline]
    pub fn src_addr(&self) -> VlcbCanId {
        VlcbCanId::from_bytes(&[self.buffer.as_ref()[field::ID_CANID]])
    }

    /// Return the frame priority.
    pub fn priority(&self) -> Priority {
        let prio = (NetworkEndian::read_u16(&self.buffer.as_ref()[field::ID]) & field::ID_PRIORITY_MASK << 7) as u8;

        Priority::from_primitive(prio & Priority::MASK)
    }

    // Indicate whether the frame is a CAN RTR frame
    pub fn is_rtr(&self) -> bool {
        NetworkEndian::read_u16(&self.buffer.as_ref()[field::ID]) & HEADER_RTR_MASK != 0
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Frame<&'a T> {
    /// Return a pointer to the payload.
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        let data = self.buffer.as_ref();
        &data[field::PAYLOAD]
    }
}

impl<T: AsRef<[u8]> + BorrowMut<[u8]>> Frame<T> {
    /// Set the source address field.
    #[inline]
    pub fn set_src_addr(&mut self, value: VlcbCanId) {
        let data = self.buffer.borrow_mut();
        data[field::ID_CANID] =
            vlcb_core::mask_and_insert_value!(data[field::ID_CANID], value, CANID_MASK, u8);
    }

    /// Set the priority field.
    #[inline]
    pub fn set_priority(&mut self, priority: Priority) {
        let data = self.buffer.borrow_mut();
        let val: u8 = priority as u8;
        let new_data = vlcb_core::mask_and_insert_value!(
            NetworkEndian::read_u16(&data[field::ID]),
            (val << 7),
            field::ID_PRIORITY_MASK,
            u16
        );
        NetworkEndian::write_u16(&mut data[field::ID], new_data);
    }

    #[inline]
    pub fn set_rtr(&mut self, value: bool) {
        if value {
            let data = self.buffer.borrow_mut();
            let old_val = NetworkEndian::read_u16(&data[field::ID]);
            NetworkEndian::write_u16(&mut data[field::ID], old_val | HEADER_RTR_MASK);
        }
    }

    /// Return a mutable pointer to the payload.
    #[inline]
    pub fn payload_mut(&mut self) -> &mut [u8] {
        let data = self.buffer.borrow_mut();
        &mut data[field::PAYLOAD]
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for Frame<T> {
    fn as_ref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl<T: AsRef<[u8]> + BorrowMut<[u8]>> fmt::Display for Frame<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CAN2.0 src_id={} prio={}",
            self.src_addr(),
            self.priority(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_src_addr() {
        let mut frame = Frame::new_unchecked([0u8; 10]);
        let mut addr = VlcbCanId::from_bytes(&[0x7F]);

        frame.set_src_addr(addr);
        assert_eq!(frame.src_addr(), addr);
        assert_eq!(NetworkEndian::read_u16(&frame.buffer[field::ID]), 0x007F);

        addr = VlcbCanId::from_bytes(&[0x00]);
        frame.set_src_addr(addr);
        assert_eq!(NetworkEndian::read_u32(&frame.buffer[field::ID]), 0x0);
    }

    // #[test]
    // fn test_priority() {
    //     let mut frame = Frame::new_unchecked([0u8; 10]);

    //     frame.set_priority(Prio0);
    //     assert_eq!(frame.priority(), Prio0);
    //     assert_eq!(NetworkEndian::read_u16(&frame.buffer[field::ID]), 0x0580);

    //     frame.set_priority(Prio11);
    //     assert_eq!(NetworkEndian::read_u16(&frame.buffer[field::ID]), 0x0);
    // }
}
