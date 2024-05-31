use crate::phy::Medium;
use vlcb_core::can::VlcbCanId;
use cfg_if::cfg_if;
use core::fmt;

mod field {
    pub type Field = core::ops::Range<usize>;
    pub type Rest = core::ops::RangeFrom<usize>;
    pub type Until = core::ops::RangeInclusive<usize>;
    pub type Single = usize;
}

mod vlcb;

cfg_if! {
    if #[cfg(feature = "medium-can")] {
        pub(crate) mod can;

        pub use self::can::{
            Frame as CanFrame,
            HEADER_LEN as CAN_HEADER_LEN,
        };
    }
}

pub use self::vlcb::{Packet as VlcbPacketWire, Repr as VlcbRepr, VLCB_MAX_PAYLOAD};

/// Parsing of a packet failed.
///
/// Either it's malformed, or not supported by this library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wire::Error")
    }
}

pub type Result<T> = core::result::Result<T, Error>;

/// Representation of a hardware address, such as an CBUS CAN ID.
#[cfg(feature = "medium-can")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HardwareAddress {
    #[cfg(feature = "medium-can")]
    CAN(VlcbCanId),
}

impl Default for HardwareAddress {
    #[allow(clippy::needless_return)]
    fn default() -> Self {
        cfg_if! {
            if #[cfg(feature = "medium-can")] {
                return Self::CAN(VlcbCanId::default());
            } else {
                compile_error! (
                    "You must enable at least one medium feature"
                )
            }
        }
    }
}

#[cfg(feature = "medium-can")]
impl HardwareAddress {
    pub const fn as_bytes(&self) -> &[u8] {
        match self {
            #[cfg(feature = "medium-can")]
            HardwareAddress::CAN(node) => node.as_bytes(),
        }
    }

    #[cfg(feature = "medium-can")]
    pub(crate) fn can_or_panic(&self) -> VlcbCanId {
        match self {
            HardwareAddress::CAN(node) => *node,
            #[allow(unreachable_patterns)]
            _ => panic!("HardwareAddress is not CAN."),
        }
    }

    #[inline]
    pub(crate) fn medium(&self) -> Medium {
        match self {
            #[cfg(feature = "medium-can")]
            HardwareAddress::CAN(_) => Medium::CAN,
        }
    }
}

impl fmt::Display for HardwareAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(feature = "medium-can")]
            HardwareAddress::CAN(node) => write!(f, "{}", node),
        }
    }
}

#[cfg(feature = "medium-can")]
impl From<VlcbCanId> for HardwareAddress {
    fn from(addr: VlcbCanId) -> Self {
        HardwareAddress::CAN(addr)
    }
}

cfg_if! {
    if #[cfg(feature = "medium-can")] {
        pub const MAX_HARDWARE_ADDRESS_LEN: usize = 2;
    } else {
        core::compile_error!("At least one medium feature needs to be enabled for deciding which MAX_HARDWARE_ADDRESS_LEN value to use");
    }
}

/// Unparsed hardware address.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawHardwareAddress {
    len: u8,
    data: [u8; MAX_HARDWARE_ADDRESS_LEN],
}

impl RawHardwareAddress {
    pub fn from_bytes(addr: &[u8]) -> Self {
        let mut data = [0u8; MAX_HARDWARE_ADDRESS_LEN];
        data[..addr.len()].copy_from_slice(addr);

        Self {
            len: addr.len() as u8,
            data,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }

    pub const fn len(&self) -> usize {
        self.len as usize
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn parse(&self, medium: Medium) -> Result<HardwareAddress> {
        match medium {
            #[cfg(feature = "medium-can")]
            Medium::CAN => {
                if self.len() < 2 {
                    return Err(Error);
                }
                let addr = VlcbCanId::from_bytes(self.as_bytes());

                Ok(addr.into())
            }
        }
    }
}

impl fmt::Display for RawHardwareAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, &b) in self.as_bytes().iter().enumerate() {
            if i != 0 {
                write!(f, ".")?;
            }
            write!(f, "{b:02X}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "medium-can")]
impl From<VlcbCanId> for RawHardwareAddress {
    fn from(addr: VlcbCanId) -> Self {
        Self::from_bytes(addr.as_bytes())
    }
}

impl From<HardwareAddress> for RawHardwareAddress {
    fn from(addr: HardwareAddress) -> Self {
        Self::from_bytes(addr.as_bytes())
    }
}
