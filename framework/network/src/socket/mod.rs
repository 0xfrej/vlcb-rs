/*! Buffered communication.

The `socket` module deals with *buffering* and separating VLCB protocol into separate
sub protocols when it's convenient.
It provides interfaces for accessing buffers of data, and protocol state machines
for filling and emptying these buffers.
 */

use crate::iface::Context;
use embedded_time::{Clock, Instant};

#[cfg(feature = "socket-module")]
pub mod module;

/// Gives an indication on the next time the socket should be polled.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub(crate) enum PollAt<C: Clock> {
    /// The socket needs to be polled immediately.
    Now,
    /// The socket needs to be polled at given [Instant][struct.Instant].
    Time(Instant<C>),
    /// The socket does not need to be polled unless there are external changes.
    Ingress,
}

/// An VLCB protocol-based networking socket abstraction.
///
/// This enumeration represents the diverse set of socket types adhering to the VLCB protocol.
/// To derive a concrete socket type from the `Socket` enum, make use of the [AnySocket] trait.
/// For instance, to acquire `event::Socket`, invoke `event::Socket::downcast(socket)`.
///
/// However, it's commonly more efficient to employ [SocketSet::get].
///
/// [AnySocket]: trait.AnySocket.html
/// [SocketSet::get]: struct.SocketSet.html#method.get
#[derive(Debug)]
pub enum Socket<'a> {
    #[cfg(feature = "socket-module")]
    Module(module::Socket<'a>),
}

impl<'a> Socket<'a> {
    pub(crate) fn poll_at<C: Clock>(&self, cx: &mut Context<C>) -> PollAt<C> {
        match self {
            #[cfg(feature = "socket-module")]
            Socket::Module(_s) => todo!(),
        }
    }
}

/// A conversion trait for network sockets.
pub trait AnySocket<'a> {
    fn upcast(self) -> Socket<'a>;
    fn downcast<'c>(socket: &'c Socket<'a>) -> Option<&'c Self>
    where
        Self: Sized;
    fn downcast_mut<'c>(socket: &'c mut Socket<'a>) -> Option<&'c mut Self>
    where
        Self: Sized;
}

macro_rules! from_socket {
    ($socket:ty, $variant:ident) => {
        impl<'a> AnySocket<'a> for $socket {
            fn upcast(self) -> Socket<'a> {
                Socket::$variant(self)
            }

            fn downcast<'c>(socket: &'c Socket<'a>) -> Option<&'c Self> {
                #[allow(unreachable_patterns)]
                match socket {
                    Socket::$variant(socket) => Some(socket),
                    _ => None,
                }
            }

            fn downcast_mut<'c>(socket: &'c mut Socket<'a>) -> Option<&'c mut Self> {
                #[allow(unreachable_patterns)]
                match socket {
                    Socket::$variant(socket) => Some(socket),
                    _ => None,
                }
            }
        }
    };
}

#[cfg(feature = "socket-module")]
from_socket!(module::Socket<'a>, Module);