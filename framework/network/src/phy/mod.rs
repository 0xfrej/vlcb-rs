use vlcb_defs::BusType;
use cfg_if::cfg_if;

#[cfg(feature = "medium-can")]
pub mod can;

/// A description of device capabilities.
///
/// Higher-level protocols may use this information to determine how to behave.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct DeviceCapabilities {
    /// Medium of the device.
    ///
    /// This indicates what kind of packet the sent/received bytes are, and determines
    /// some behaviors of Interface.
    pub medium: Medium,
}

/// Type of medium of a device.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Medium {
    /// CAN medium. Devices of this type send and receive CAN frames.
    #[cfg(feature = "medium-can")]
    CAN,
}

impl Default for Medium {
    fn default() -> Medium {
        cfg_if! {
            if #[cfg(feature = "medium-can")] {
                Medium::CAN
            }
            else {
                compile_error!("No medium feature enabled");
            }
        }
    }
}

impl From<Medium> for BusType {
    fn from(value: Medium) -> Self {
        match value {
            Medium::CAN => Self::CAN,
        }
    }
}

/// Interface for sending and receiving raw network frames.
///
/// This interface revolves around _tokens_, specialized types facilitating the reception
/// and transmission of individual packets. The `receive` and `transmit` functions focus
/// on token construction, while the actual sending and receiving operations occur
/// when the tokens are consumed.
pub trait Device {
    type RxToken<'a>: RxToken
    where
        Self: 'a;
    type TxToken<'a>: TxToken
    where
        Self: 'a;

    /// Create a pair of tokens, comprising one receive token and one transmit token.
    ///
    /// Including an extra transmit token enables the generation of a reply packet using
    /// the information from the received packet. This functionality proves useful in scenarios
    /// where all received bytes must be sent back without resorting to heap allocation.
    fn receive(&mut self) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)>;

    /// Create a transmit token.
    fn transmit(&mut self) -> Option<Self::TxToken<'_>>;

    /// Get a description of device capabilities.
    fn capabilities(&self) -> DeviceCapabilities;
}

/// A token to receive a single network packet.
pub trait RxToken {
    /// Utilize the token for receiving a singular network packet.
    ///
    /// This method acquires a packet and subsequently invokes the provided closure `f`
    /// with the raw packet bytes as its argument.
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;
}

/// A token to transmit a single network packet.
pub trait TxToken: Clone {
    /// Utilize the token to dispatch a solitary network packet.
    ///
    /// This method creates a transmit buffer of size `len` and invokes the supplied closure `f`
    /// with a mutable reference to that buffer. The closure's responsibility is to construct
    /// a valid network packet (such as an CAN packet) within the buffer.
    /// Upon the closure's completion, the transmit buffer is dispatched.
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R;
}
