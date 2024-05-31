use core::fmt;

/// Size of an CBUS CAN ID in octets.
pub const CANID_SIZE: usize = 1;
pub const CANID_MASK: u8 = 0x7f;

/// A 7-bit CAN ID for CBUS.
///
/// Used to identify nodes on a CAN network
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VlcbCanId(pub [u8; CANID_SIZE]);

impl VlcbCanId {
    /// Construct an CAN address from an octet.
    ///
    /// # Panics
    /// The function panics if `data` is not one octet long.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut bytes = [0; CANID_SIZE];
        bytes.copy_from_slice(data);
        Self(bytes.map(|x| x & CANID_MASK))
    }

    /// Return an CAN address as an octet.
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for VlcbCanId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02X}", self.0[0])
    }
}

impl From<VlcbCanId> for u8 {
    fn from(value: VlcbCanId) -> Self {
        value.0[0]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_address() {
        let addr = VlcbCanId::from_bytes(&[0xFF]);
        assert_eq!(addr.as_bytes(), &[0x7F]);
        assert_eq!(addr.to_string(), "7F");
    }
}
