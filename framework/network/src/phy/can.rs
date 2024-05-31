#![allow(unsafe_code)]

use core::cell::RefCell;
use core::fmt::Debug;

use byteorder::{ByteOrder, NetworkEndian};
use embedded_can::{Error, Id, StandardId};
use heapless::Vec;
use rclite::Rc;

use crate::phy;
use crate::wire::can::{HEADER_RTR_MASK};

use super::{Device, DeviceCapabilities, Medium};

// 11 bits the least significant bits for the ID value
// 1 most significant bit for RTR flag
// RTR frames are always sent with DLC of 0
const HEADER_LEN: usize = 2;
const MTU: usize = 8;
const FRAME_LEN: usize = HEADER_LEN + MTU;

/// An embedded-can device driver wrapper
#[derive(Debug)]
pub struct EmbeddedCan<D: embedded_can::nb::Can> {
    lower: Rc<RefCell<D>>,
}

impl<D: embedded_can::nb::Can> EmbeddedCan<D> {
    /// Creates an embedded-can device, bound to the given device driver
    pub fn new(device: D) -> Self {
        EmbeddedCan {
            lower: Rc::new(RefCell::new(device)),
        }
    }
}

impl<D: embedded_can::nb::Can> Device for EmbeddedCan<D> {
    type RxToken<'a> = RxToken
        where
            Self: 'a;
    type TxToken<'a> = TxToken<D>
        where
            Self: 'a;

    fn receive(&mut self) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut lower = self.lower.borrow_mut();
        match lower.receive() {
            Ok(frame) => {
                if let Some(buffer) = from_can_frame::<D::Frame>(frame) {
                    let rx = RxToken { buffer };
                    let tx = TxToken {
                        lower: self.lower.clone(),
                    };
                    return Some((rx, tx));
                }
                None
            }
            Err(nb::Error::WouldBlock) => None,
            Err(nb::Error::Other(err)) => panic!("{}", err.kind()),
        }
    }

    fn transmit(&mut self) -> Option<Self::TxToken<'_>> {
        Some(TxToken {
            lower: self.lower.clone(),
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            medium: Medium::CAN,
            ..DeviceCapabilities::default()
        }
    }
}

#[doc(hidden)]
pub struct RxToken {
    buffer: Vec<u8, FRAME_LEN>,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.buffer[..])
    }
}

#[doc(hidden)]
pub struct TxToken<D: embedded_can::nb::Can> {
    lower: Rc<RefCell<D>>,
}

impl<D: embedded_can::nb::Can> Clone for TxToken<D> {
    fn clone(&self) -> Self {
        Self {
            lower: Rc::clone(&self.lower),
        }
    }
}

impl<D: embedded_can::nb::Can> phy::TxToken for TxToken<D> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut lower = self.lower.borrow_mut();
        let mut buffer: Vec<u8, FRAME_LEN> = Vec::new();
        let result = f(&mut buffer[..len]);
        match lower.transmit(&into_can_frame::<D::Frame>(&buffer[..len])) {
            Ok(_) => {}
            Err(nb::Error::WouldBlock) => {
                net_debug!("phy: tx failed due to WouldBlock")
            }
            Err(nb::Error::Other(err)) => panic!("{}", err.kind()),
        }
        result
    }
}

fn into_can_frame<T: embedded_can::Frame>(buffer: &[u8]) -> T {
    let header = NetworkEndian::read_u16(buffer);
    let id = Id::Standard(StandardId::new(header & !HEADER_RTR_MASK).unwrap());
    if (header & HEADER_RTR_MASK) != 0 {
        T::new_remote(id, 0).unwrap()
    } else {
        T::new(id, &buffer[HEADER_LEN..]).unwrap()
    }
}

fn from_can_frame<T: embedded_can::Frame>(value: T) -> Option<Vec<u8, FRAME_LEN>> {
    match value.id() {
        // Nodes should operate properly even if network carries extended frames
        // If such frames are encountered simply ignore them
        Id::Standard(id) => {
            let mut data = Vec::<u8, FRAME_LEN>::new();

            // Safety: set the length of the vector to 2 to avoid copying from slices
            unsafe {
                data.set_len(2);
            }
            let mut header = id.as_raw();

            if value.is_remote_frame() {
                header |= HEADER_RTR_MASK;
            }

            NetworkEndian::write_u16(&mut data[0..HEADER_LEN], header);
            if value.is_data_frame() && value.dlc() > 0 {
                data.extend_from_slice(value.data()).unwrap();
            }
            Some(data)
        }
        Id::Extended(_) => None,
    }
}

#[cfg(test)]
mod test {
    use embedded_can::{ExtendedId, Frame};

    use super::*;

    struct TestFrame {
        id: Id,
        remote: bool,
        data: Vec<u8, 8>,
    }

    impl Frame for TestFrame {
        fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
            Some(TestFrame {
                id: id.into(),
                remote: false,
                data: Vec::from_slice(data).unwrap(),
            })
        }

        fn new_remote(_id: impl Into<Id>, _dlc: usize) -> Option<Self> {
            None
        }

        fn is_extended(&self) -> bool {
            matches!(self.id, Id::Extended(_))
        }

        fn is_remote_frame(&self) -> bool {
            self.remote
        }

        fn is_data_frame(&self) -> bool {
            !self.remote
        }

        fn id(&self) -> Id {
            self.id
        }

        fn dlc(&self) -> usize {
            self.data.len()
        }

        fn data(&self) -> &[u8] {
            &self.data
        }
    }

    #[test]
    fn test_into_can_frame() {
        let buffer = [
            0x00, 0xFF, // id
            0xAF, 0x00, 0xBF, 0x00, // data
            0xCF, 0x00, 0xDF, 0x00, // data
        ];

        let frame = into_can_frame::<TestFrame>(&buffer);
        assert_eq!(frame.id(), Id::Standard(StandardId::new(0x00FF).unwrap()));
        assert_eq!(frame.dlc(), 8);
        assert_eq!(
            frame.data(),
            &[0xAF, 0x00, 0xBF, 0x00, 0xCF, 0x00, 0xDF, 0x00]
        );
    }

    #[test]
    fn test_from_can_frame_correct_frame() {
        let buffer = [
            0x00, 0xFF, // id
            0xAF, 0x00, 0xBF, 0x00, // data
            0xCF, 0x00, 0xDF, 0x00, // data
        ];

        let frame = TestFrame {
            id: Id::Standard(StandardId::new(0x00FF).unwrap()),
            remote: false,
            data: Vec::from_slice(&buffer[4..]).unwrap(),
        };

        assert_eq!(from_can_frame::<TestFrame>(frame).unwrap(), buffer);
    }

    #[test]
    fn test_from_can_frame_remote_frame() {
        let buffer: [u8; FRAME_LEN] = [
            0x00, 0xFF, // id
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let frame = TestFrame {
            id: Id::Standard(StandardId::new(0x00FF).unwrap()),
            remote: true,
            data: Vec::new(),
        };

        assert_eq!(from_can_frame::<TestFrame>(frame).unwrap(), buffer);
    }

    #[test]
    fn test_from_can_frame_extended_frame() {
        let frame = TestFrame {
            id: Id::Extended(ExtendedId::new(0x1F00FF00).unwrap()),
            remote: false,
            data: Vec::new(),
        };

        assert_eq!(from_can_frame::<TestFrame>(frame), None);
    }
}
