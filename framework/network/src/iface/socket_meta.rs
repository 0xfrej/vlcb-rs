use super::SocketHandle;

// Credit: authors of https://github.com/smoltcp-rs/smoltcp

/// Socket metadata.
///
/// This includes things that only external code to the socket
/// is interested in, but which are more conveniently stored inside the socket
/// itself.
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct Meta {
    /// Handle of this socket within its enclosing `SocketSet`.
    pub(crate) handle: SocketHandle,
}
