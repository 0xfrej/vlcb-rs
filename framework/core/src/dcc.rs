use byteorder::{ByteOrder, NetworkEndian};
use num_enum::{FromPrimitive, IntoPrimitive};

pub struct LocoAddress([u8;2], bool);

impl LocoAddress {
    /// Constructs short DCC locomotive address
    pub fn new(addr: u8) -> Self {
        Self(
            [0x0, addr],
            false,
        )
    }

    /// Constructs long DCC locomotive address
    pub fn new_long(addr: u16) -> Self {
        let mut s = Self([0u8; 2], true);
        NetworkEndian::write_u16(&mut s.0, addr);
        s
    }

    /// Get the address type
    ///
    /// Returns true when the address is 14 bits long
    pub fn is_long(&self) -> bool {
        self.1
    }

    /// Returns the address data as two octets in big endian
    pub fn as_bytes(&self) -> [u8; 2] {
        self.0
    }

    /// Returns the address data as two octets in big endian with
    /// sanitization that is useful for constructing CBUS packets
    ///
    /// 7 bit addresses have most significant octet set to 0.
    /// 14 bit addresses have bits 6,7 of most significant octet set to 1.
    pub fn as_bytes_sanitized(&self) -> [u8; 2] {
        let mut bytes = self.as_bytes();

        if self.is_long() {
            bytes[0] |= 0xC0;
        } else {
            bytes[0] = 0x0;
        }

        bytes
    }
}


/// Loco state
#[derive(FromPrimitive, IntoPrimitive, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum EngineState {
    Active = 0,
    Consisted = 1,
    ConsistMaster = 2,
    #[default]
    Inactive = 3,
}

#[derive(FromPrimitive, IntoPrimitive, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum EngineFunctionRange {
    #[default]
    F0ToF4 = 1,
    F5ToF8 = 2,
    F9ToF12 = 3,
    F13ToF20 = 4,
    F21ToF28= 5,
}

#[derive(FromPrimitive, IntoPrimitive, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum SessionQueryMode {
    #[default]
    Default = 0x00,
    Steal = 0x01,
    Share = 0x02,
}