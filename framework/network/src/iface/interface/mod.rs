// Before working on this file you should read the parts
// of the CBUS specifications.

#[cfg(feature = "medium-can")]
mod can;

#[cfg(feature = "socket-event")]
mod event;

mod vlcb;

use super::vlcb_packet::*;
use core::convert::Infallible;
use core::marker::PhantomData;

use vlcb_core::vlcb::VlcbNodeNumber;
use core::result::Result;
use embedded_time::{Clock, Instant};
use nb::Error::WouldBlock;

use crate::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};

use crate::iface::SocketSet;
use crate::socket::Socket;
use crate::wire::{VlcbPacketWire, HardwareAddress};

macro_rules! check {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(_) => {
                // concat!/stringify! doesn't work with defmt macros
                #[cfg(not(feature = "defmt"))]
                net_trace!(concat!("iface: malformed ", stringify!($e)));
                #[cfg(feature = "defmt")]
                net_trace!("iface: malformed");
                return Default::default();
            }
        }
    };
}

use check;

pub struct PollContext<'a, D: Device + ?Sized, C: Clock> {
    timestamp: Instant<C>,
    device: &'a mut D,
    sockets: &'a mut SocketSet<'a>,
}

impl<'a, D: Device, C: Clock> PollContext<'a, D, C> {
    pub fn new(timestamp: Instant<C>, device: &'a mut D, sockets: &'a mut SocketSet<'a>) -> Self {
        Self {
            timestamp,
            device,
            sockets,
        }
    }
}

/// A Network Interface Entity.
///
/// This entity is logically associated with multiple other data structures.
pub struct Interface<C: Clock> {
    pub(crate) inner: InterfaceInner<C>,
}

/// The hardware-agnostic component of a network interface.
///
/// By decoupling the physical device from the data necessary for computation and dispatching,
/// separate borrowing of these entities can be facilitated. For instance, the transmission (tx) and reception (rx) tokens
/// holds a mutable borrow of the `device` until they are utilized, which restricts the invocation of other
/// methods on the `Interface` during this period (as its `device` field is borrowed exclusively). Nevertheless,
/// there is still the allowance to invoke methods on its `inner` field.
pub struct InterfaceInner<C: Clock> {
    caps: DeviceCapabilities,
    addr: VlcbNodeNumber,
    hw_addr: HardwareAddress,
    now: Instant<C>,
}

impl<C: Clock> Interface<C> {
    /// Create a network interface.
    pub fn new<D>(device: &D, addr: VlcbNodeNumber, hw_addr: HardwareAddress) -> Self
    where
        D: Device,
    {
        let caps = device.capabilities();

        Interface {
            inner: InterfaceInner {
                caps,
                addr,
                hw_addr,
                now: Instant::new(C::T::from(0)),
            },
        }
    }

    /// Set the interface's address
    pub fn set_addr(&mut self, addr: VlcbNodeNumber) {
        self.inner.addr = addr
    }

    /// Set the interface's hardware address
    pub fn set_hw_addr(&mut self, addr: HardwareAddress) {
        self.inner.hw_addr = addr
    }

    /// Get the interface's address
    pub fn addr(&self) -> VlcbNodeNumber {
        self.inner.addr
    }

    /// Get the interface's hardware address
    pub fn hw_addr(&self) -> HardwareAddress {
        self.inner.hw_addr
    }

    /// Get the device capabilities of this interface
    pub fn device_caps(&self) -> &DeviceCapabilities {
        &self.inner.caps
    }

    /// Get the socket context.
    ///
    /// The context is needed for some socket methods.
    pub fn context(&mut self) -> &mut InterfaceInner<C> {
        &mut self.inner
    }

    /// Process queued packets in the specified sockets for transmission and
    /// receive incoming packets queued in the device.
    ///
    /// This function provides a boolean result indicating if any packets
    /// were processed or transmitted, thereby indicating if
    /// the availability status of any socket could have been altered.
    ///
    /// # Panics
    /// This method panics on debug builds when passed device in the `ctx` does not
    /// match the interface device capabilities
    pub fn poll<D>(&mut self, ctx: PollContext<D, C>) -> bool
    where
        D: Device,
    {
        self.inner.now = ctx.timestamp;

        debug_assert!(
            ctx.device.capabilities() == self.inner.caps,
            "Passed in device does not satisfy the device capabilities on this interface",
        );

        let mut readiness_may_have_changed = false;

        loop {
            let mut did_something = false;

            did_something |= self.ingress_packets(ctx.device, ctx.sockets);
            did_something |= self.egress_packets(ctx.device, ctx.sockets);

            if did_something {
                readiness_may_have_changed = true;
            } else {
                break;
            }
        }

        readiness_may_have_changed
    }

    fn ingress_packets<D>(&mut self, device: &mut D, sockets: &mut SocketSet<'_>) -> bool
    where
        D: Device + ?Sized,
    {
        let mut processed_any = false;

        while let Some((rx_token, tx_token)) = device.receive() {
            rx_token.consume(|frame| {
                match self.inner.caps.medium {
                    #[cfg(feature = "medium-can")]
                    Medium::CAN => {
                        if let Some(packet) = self.inner.process_can(
                            sockets,
                            frame,
                        ) {
                            if let Err(err) =
                                self.inner.dispatch_vlcb(tx_token, packet)
                            {
                                net_debug!("Failed to send response: {:?}", err);
                            }
                        }
                    }
                }
                processed_any = true;
            });
        }

        processed_any
    }

    fn egress_packets<D>(
        &mut self,
        device: &mut D,
        sockets: &mut SocketSet<'_>,
    ) -> bool
    where
        D: Device + ?Sized,
    {
        enum EgressError {
            Exhausted,
            Dispatch(DispatchError),
        }

        let mut emitted_any = false;
        for item in sockets.items_mut() {
            let mut respond =
                |inner: &mut InterfaceInner<C>, response: VlcbPacket| -> Result<(), EgressError> {
                    let t = device.transmit().ok_or_else(|| {
                        net_debug!("failed to transmit CBUS: device exhausted");
                        EgressError::Exhausted
                    })?;

                    inner
                        .dispatch_vlcb(t, response)
                        .map_err(EgressError::Dispatch)?;

                    emitted_any = true;

                    Ok(())
                };

            let result = match &mut item.socket {
                #[cfg(feature = "socket-event")]
                Socket::Event(socket) => {
                    socket.dispatch(&mut self.inner, |inner, (cbus, event)| {
                        respond(inner, VlcbPacket::new(vlcb, VlcbPayload::Event(event)))
                    })
                }
                #[cfg(feature = "socket-module")]
                Socket::Module(socket) => {
                    socket.dispatch(&mut self.inner, |inner, (cbus, payload)| {
                        respond(inner, VlcbPacket::new(cbus, VlcbPayload::Module(payload)))
                    })
                },
            };

            match result {
                Err(EgressError::Exhausted) => break, // Device buffer full.
                Err(EgressError::Dispatch(e)) => {
                    net_debug!("dispatch error: {:?}", e)
                }
                Ok(()) => {}
            }
        }
        emitted_any
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum DispatchError {}
