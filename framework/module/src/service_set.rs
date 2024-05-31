use core::fmt;
use managed::ManagedSlice;
use vlcb_svc_all::{AnyService, Service};

/// Opaque struct with space for one service.
///
/// This is public, to allow using it for allocating space for storing
/// sockets when creating a Module.
#[derive(Default)]
pub struct ServiceStorage {
    inner: Option<Item>,
}

impl<'a> ServiceStorage {
    pub const EMPTY: Self = Self { inner: None };
}

pub(crate) struct Item {
    service: Service
}

/// An extensible set of services.
///
/// The lifetime `'a` is used when storing a `Service<'a>`.  If you're using
/// owned buffers for your sockets (passed in as `Vec`s) you can use
/// `ServiceSet<'static>`.
pub struct ServiceSet<'a> {
    services: ManagedSlice<'a, ServiceStorage>,
}

impl<'a> ServiceSet<'a> {
    /// Create a service set using the provided storage.
    pub fn new<ServicesT>(sockets: ServicesT) -> ServiceSet<'a>
    where
        ServicesT: Into<ManagedSlice<'a, ServiceStorage>>,
    {
        let services = sockets.into();
        ServiceSet { services }
    }

    /// Add a socket to the set, and return its handle.
    ///
    /// # Panics
    /// This function panics if the storage is fixed-size (not a `Vec`) and is full.
    pub fn add<T: AnyService>(&mut self, socket: T) {
        fn put(slot: &mut ServiceStorage, service: Service) {
            *slot = ServiceStorage {
                inner: Some(Item { service }),
            };
        }

        let socket = socket.upcast();

        for (_, slot) in self.services.iter_mut().enumerate() {
            if slot.inner.is_none() {
                return put(slot, socket);
            }
        }

        match &mut self.services {
            ManagedSlice::Borrowed(_) => panic!("adding a service to a full ServiceSet"),
            #[cfg(feature = "alloc")]
            ManagedSlice::Owned(sockets) => {
                sockets.push(ServiceStorage { inner: None });
                let index = sockets.len() - 1;
                put(&mut sockets[index], socket)
            }
        }
    }

    /// Get an iterator to the inner service items.
    pub fn iter(&self) -> impl Iterator<Item = &Service> {
        self.items().map(|i| &i.service)
    }

    /// Get a mutable iterator to the inner service items.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Service> {
        self.items_mut().map(|i| &mut i.service)
    }

    /// Iterate every service in this set.
    pub(crate) fn items(&self) -> impl Iterator<Item = &Item> + '_ {
        self.services.iter().filter_map(|x| x.inner.as_ref())
    }

    /// Iterate every service in this set.
    pub(crate) fn items_mut(&mut self) -> impl Iterator<Item = &mut Item> + '_ {
        self.services.iter_mut().filter_map(|x| x.inner.as_mut())
    }
}