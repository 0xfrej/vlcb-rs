use core::cmp::min;
use embedded_time::Clock;

use crate::iface::Context;
use crate::socket::PollAt;

use crate::storage::Empty;
use crate::wire::VlcbRepr;

/// Error returned by [`Socket::bind`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BindError {
    InvalidState,
    Unaddressable,
}

impl core::fmt::Display for BindError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            BindError::InvalidState => write!(f, "invalid state"),
            BindError::Unaddressable => write!(f, "unaddressable"),
        }
    }
}

/// Error returned by [`Socket::send`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SendError {
    BufferFull,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SendError::BufferFull => write!(f, "buffer full"),
        }
    }
}

/// Error returned by [`Socket::recv`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RecvError {
    Exhausted,
    Truncated,
}

impl core::fmt::Display for RecvError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            RecvError::Exhausted => write!(f, "exhausted"),
            RecvError::Truncated => write!(f, "truncated"),
        }
    }
}

/// A Module packet metadata.
pub type PacketMetadata = crate::storage::PacketMetadata<()>;

/// A Module packet ring buffer.
pub type PacketBuffer<'a> = crate::storage::PacketBuffer<'a, ()>;

/// A Module CBUS socket.
///
/// This socket type is essentially filtered raw CBUS protocol
/// to be used by module implementations.
#[derive(Debug)]
pub struct Socket<'a> {
    rx_buffer: PacketBuffer<'a>,
    tx_buffer: PacketBuffer<'a>,
}

impl<'a> Socket<'a> {
    /// Create a module socket with the given buffers.
    pub fn new(rx_buffer: PacketBuffer<'a>, tx_buffer: PacketBuffer<'a>) -> Socket<'a> {
        Socket {
            rx_buffer,
            tx_buffer,
        }
    }

    /// Check whether the transmit buffer is full.
    #[inline]
    pub fn can_send(&self) -> bool {
        !self.tx_buffer.is_full()
    }

    /// Check whether the reception buffer is not empty.
    #[inline]
    pub fn can_recv(&self) -> bool {
        !self.rx_buffer.is_empty()
    }

    /// Return the maximum number packets the socket can receive.
    #[inline]
    pub fn packet_recv_capacity(&self) -> usize {
        self.rx_buffer.packet_capacity()
    }

    /// Return the maximum number packets the socket can transmit.
    #[inline]
    pub fn packet_send_capacity(&self) -> usize {
        self.tx_buffer.packet_capacity()
    }

    /// Return the maximum number of bytes inside the recv buffer.
    #[inline]
    pub fn payload_recv_capacity(&self) -> usize {
        self.rx_buffer.payload_capacity()
    }

    /// Return the maximum number of bytes inside the transmit buffer.
    #[inline]
    pub fn payload_send_capacity(&self) -> usize {
        self.tx_buffer.payload_capacity()
    }

    /// Enqueue a packet to send, and return a pointer to its payload.
    ///
    /// This function returns `Err(Error::Exhausted)` if the transmit buffer is full,
    /// and `Err(Error::Truncated)` if there is not enough transmit buffer capacity
    /// to ever send this packet.
    pub fn send(&mut self, size: usize) -> Result<&mut [u8], SendError> {
        let packet_buf = self
            .tx_buffer
            .enqueue(size, ())
            .map_err(|_| SendError::BufferFull)?;

        net_trace!("module: buffer to send {} octets", packet_buf.len());
        Ok(packet_buf)
    }

    /// Enqueue a packet to be send and pass the buffer to the provided closure.
    /// The closure then returns the size of the data written into the buffer.
    ///
    /// Also see [send](#method.send).
    pub fn send_with<F>(&mut self, max_size: usize, f: F) -> Result<usize, SendError>
    where
        F: FnOnce(&mut [u8]) -> usize,
    {
        let size = self
            .tx_buffer
            .enqueue_with_infallible(max_size, (), f)
            .map_err(|_| SendError::BufferFull)?;

        net_trace!("module: buffer to send {} octets", size);

        Ok(size)
    }

    /// Enqueue a packet to send, and fill it from a slice.
    ///
    /// See also [send](#method.send).
    pub fn send_slice(&mut self, data: &[u8]) -> Result<(), SendError> {
        self.send(data.len())?.copy_from_slice(data);
        Ok(())
    }

    /// Dequeue a packet, and return a pointer to the payload.
    ///
    /// This function returns `Err(Error::Exhausted)` if the receive buffer is empty.
    ///
    /// **Note:** The IP header is parsed and re-serialized, and may not match
    /// the header actually received bit for bit.
    pub fn recv(&mut self) -> Result<&[u8], RecvError> {
        let ((), packet_buf) = self.rx_buffer.dequeue().map_err(|_| RecvError::Exhausted)?;

        net_trace!("module: receive {} buffered octets", packet_buf.len());
        Ok(packet_buf)
    }

    /// Dequeue a packet, and copy the payload into the given slice.
    ///
    /// **Note**: when the size of the provided buffer is smaller than the size of the payload,
    /// the packet is dropped and a `RecvError::Truncated` error is returned.
    ///
    /// See also [recv](#method.recv).
    pub fn recv_slice(&mut self, data: &mut [u8]) -> Result<usize, RecvError> {
        let buffer = self.recv()?;
        if data.len() < buffer.len() {
            return Err(RecvError::Truncated);
        }

        let length = min(data.len(), buffer.len());
        data[..length].copy_from_slice(&buffer[..length]);
        Ok(length)
    }

    /// Peek at a packet in the receive buffer and return a pointer to the
    /// payload without removing the packet from the receive buffer.
    /// This function otherwise behaves identically to [recv](#method.recv).
    ///
    /// It returns `Err(Error::Exhausted)` if the receive buffer is empty.
    pub fn peek(&mut self) -> Result<&[u8], RecvError> {
        let ((), packet_buf) = self.rx_buffer.peek().map_err(|_| RecvError::Exhausted)?;

        net_trace!("module: receive {} buffered octets", packet_buf.len());

        Ok(packet_buf)
    }

    /// Peek at a packet in the receive buffer, copy the payload into the given slice,
    /// and return the amount of octets copied without removing the packet from the receive buffer.
    /// This function otherwise behaves identically to [recv_slice](#method.recv_slice).
    ///
    /// **Note**: when the size of the provided buffer is smaller than the size of the payload,
    /// no data is copied into the provided buffer and a `RecvError::Truncated` error is returned.
    ///
    /// See also [peek](#method.peek).
    pub fn peek_slice(&mut self, data: &mut [u8]) -> Result<usize, RecvError> {
        let buffer = self.peek()?;
        if data.len() < buffer.len() {
            return Err(RecvError::Truncated);
        }

        let length = min(data.len(), buffer.len());
        data[..length].copy_from_slice(&buffer[..length]);
        Ok(length)
    }

    pub(crate) fn process<C>(&mut self, cx: &mut Context<C>, vlcb_repr: &VlcbRepr, payload: &[u8])
    where
        C: Clock,
    {
        todo!(); /*
                 let header_len = cbus_repr.header_len();
                 let total_len = header_len + payload.len();

                 net_trace!("raw: receiving {} octets", total_len);

                 match self.rx_buffer.enqueue(total_len, ()) {
                     Ok(buf) => {
                         cbus_repr.emit(&mut buf[..header_len]);
                         buf[header_len..].copy_from_slice(payload);
                     }
                     Err(_) => net_trace!("raw: buffer full, dropped incoming packet"),
                 }*/
    }

    pub(crate) fn dispatch<F, E, C>(&mut self, cx: &mut Context<C>, emit: F) -> Result<(), E>
    where
        F: FnOnce(&mut Context<C>, (VlcbRepr, &[u8])) -> Result<(), E>,
        C: Clock,
    {
        todo!();
        // let res = self.tx_buffer.dequeue_with(|&mut (), buffer| {
        //     match IpVersion::of_packet(buffer) {
        //         #[cfg(feature = "proto-ipv4")]
        //         Ok(IpVersion::Ipv4) => {
        //             let mut packet = match Ipv4Packet::new_checked(buffer) {
        //                 Ok(x) => x,
        //                 Err(_) => {
        //                     net_trace!("raw: malformed ipv6 packet in queue, dropping.");
        //                     return Ok(());
        //                 }
        //             };
        //             if packet.next_header() != ip_protocol {
        //                 net_trace!("raw: sent packet with wrong ip protocol, dropping.");
        //                 return Ok(());
        //             }
        //             if _checksum_caps.ipv4.tx() {
        //                 packet.fill_checksum();
        //             } else {
        //                 // make sure we get a consistently zeroed checksum,
        //                 // since implementations might rely on it
        //                 packet.set_checksum(0);
        //             }
        //
        //             let packet = Ipv4Packet::new_unchecked(&*packet.into_inner());
        //             let ipv4_repr = match Ipv4Repr::parse(&packet, _checksum_caps) {
        //                 Ok(x) => x,
        //                 Err(_) => {
        //                     net_trace!("raw: malformed ipv4 packet in queue, dropping.");
        //                     return Ok(());
        //                 }
        //             };
        //             net_trace!("raw:{}:{}: sending", ip_version, ip_protocol);
        //             emit(cx, (IpRepr::Ipv4(ipv4_repr), packet.payload()))
        //         }
        //         #[cfg(feature = "proto-ipv6")]
        //         Ok(IpVersion::Ipv6) => {
        //             let packet = match Ipv6Packet::new_checked(buffer) {
        //                 Ok(x) => x,
        //                 Err(_) => {
        //                     net_trace!("raw: malformed ipv6 packet in queue, dropping.");
        //                     return Ok(());
        //                 }
        //             };
        //             if packet.next_header() != ip_protocol {
        //                 net_trace!("raw: sent ipv6 packet with wrong ip protocol, dropping.");
        //                 return Ok(());
        //             }
        //             let packet = Ipv6Packet::new_unchecked(&*packet.into_inner());
        //             let ipv6_repr = match Ipv6Repr::parse(&packet) {
        //                 Ok(x) => x,
        //                 Err(_) => {
        //                     net_trace!("raw: malformed ipv6 packet in queue, dropping.");
        //                     return Ok(());
        //                 }
        //             };
        //
        //             net_trace!("raw:{}:{}: sending", ip_version, ip_protocol);
        //             emit(cx, (IpRepr::Ipv6(ipv6_repr), packet.payload()))
        //         }
        //         Err(_) => {
        //             net_trace!("raw: sent packet with invalid IP version, dropping.");
        //             Ok(())
        //         }
        //     }
        // });
        // match res {
        //     Err(Empty) => Ok(()),
        //     Ok(Err(e)) => Err(e),
        //     Ok(Ok(())) => Ok(()),
        // }
    }

    pub(crate) fn poll_at<C>(&self, _cx: &mut Context<C>) -> PollAt<C>
    where
        C: Clock,
    {
        if self.tx_buffer.is_empty() {
            PollAt::Ingress
        } else {
            PollAt::Now
        }
    }
}
