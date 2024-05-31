#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(unsafe_code)]

pub mod node_config;

pub trait Storage {
    /// Wipe storage clean
    fn wipe(&mut self);
}

/// A persistent storage trait for loading and storing data.
pub trait PersistentStorage {
    /// Loads the necessary data into the object.
    ///
    /// This method is used to load the required data into the object.
    /// It should be called before using any other methods that rely on the data being loaded.
    #[must_use]
    fn load(&mut self);

    fn is_dirty(&self) -> bool;

    fn flush(&mut self);

    fn force_flush(&mut self);
}