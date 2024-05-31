use byteorder::{ByteOrder, NetworkEndian};

/// Size of an CBUS node number in octets.
pub const NODENUM_SIZE: usize = 2;

/// A two-octet CBUS node number.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct VlcbNodeNumber(pub [u8; NODENUM_SIZE]);

impl VlcbNodeNumber {
    /// Construct an CBUS node number from parts.
    pub const fn new(a0: u8, a1: u8) -> Self {
        Self([a0, a1])
    }

    /// Construct an CBUS node number from a sequence of octets, in big-endian.
    ///
    /// # Panics
    /// The function panics if `data` is not two octets long.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut bytes = [0; NODENUM_SIZE];
        bytes.copy_from_slice(data);
        Self(bytes)
    }

    /// Return an CBUS node number as a sequence of octets, in big-endian.
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Default for VlcbNodeNumber {
    fn default() -> Self {
        Self([0u8; NODENUM_SIZE])
    }
}

/// Size of an CBUS P / C event in octets.
pub const EVENT_SIZE: usize = 4;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EventType {
    /// Event type unknown to the library implementation
    Unknown,

    /// Accessory event state change to "ON"
    AccessoryOn,

    /// Accessory event state change to "OFF"
    AccessoryOff,

    /// Accessory event state response "ON"
    /// used for responding to accessory state queries
    /// without producing event state change events.
    AccessoryStatusOn,

    /// Accessory event state response "OFF"
    /// used for responding to accessory state queries
    /// without producing event state change events.
    AccessoryStatusOff
}

/// A four-octet CBUS P / C event.
#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EventId {
    data: [u8; EVENT_SIZE],
    is_short: bool,
}

// TODO: drop is_short - we don't need this because actually we will send them anyway, it's just that OPCODE will specify that the consumer will ignore it
// Though we will need another or modified funcitonality of this to make it work properly -> event store should ignore the node number, etc
// it would be great if we could retain the sanitization where needed but allow the full 4 bytes to be passed around in the stack!
impl EventId {
    /// Construct an CBUS P / C event from parts.
    pub const fn new(short: bool, a0: u8, a1: u8, a2: u8, a3: u8) -> Self {
        Self {
            data: [a0, a1, a2, a3],
            is_short: short,
        }
    }

    /// Construct a long CBUS P / C event from a sequence of octets, in big-endian.
    ///
    /// # Panics
    /// The function panics if `data` is not four octets long.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut bytes = [0; EVENT_SIZE];
        bytes.copy_from_slice(data);
        Self {
            data: bytes,
            is_short: false
        }
    }

    /// Construct a short CBUS P / C event from a sequence of octets, in big-endian.
    ///
    /// Short event is essentially the same size af "long" events, but node number is ignored.
    /// The data is still 4 octets long, but with the node number part null-ed.
    ///
    /// # Panics
    /// The function panics if `data` is not two octets long.
    pub fn short_from_bytes(data: &[u8]) -> Self {
        let mut bytes = [0; EVENT_SIZE];
        bytes[2..].copy_from_slice(&data[2..]);
        Self {
            data: bytes,
            is_short: true
        }
    }

    /// Construct an CBUS P / C event from a node number and event id.
    pub fn from_node_and_id(node_num: &VlcbNodeNumber, evt_id: u16, short: bool) -> Self {
        let mut bytes = [0; EVENT_SIZE];
        bytes.copy_from_slice(node_num.as_bytes());
        NetworkEndian::write_u16(&mut bytes[2..], evt_id);
        Self {
            data: bytes,
            is_short: short,
        }
    }

    /// Return an CBUS P / C event as a sequence of octets, in big-endian.
    pub const fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Return a CBUS node number
    pub fn node_num(&self) -> VlcbNodeNumber {
        VlcbNodeNumber::from_bytes(&self.data[0..2])
    }

    /// Return a CBUS event number
    pub fn event_num(&self) -> u16 {
        NetworkEndian::read_u16(&self.data[2..])
    }

    /// Check whether the event is short
    pub fn is_short(&self) -> bool {
        self.is_short
    }

    /// Check whether the event is long
    pub fn is_long(&self) -> bool {
        !self.is_short
    }
}
