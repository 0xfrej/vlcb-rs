use core::default;

use num_enum::{FromPrimitive, IntoPrimitive};
use vlcb_defs::VlcbServiceTypes;

pub trait VlcbService {
    /// Runs the service initialization
    #[must_use]
    fn init() {}

    /// Returns the service ID
    ///
    /// By default it returns [`VlcbServiceTypes::NONE`] which means that the service
    /// is should not communicate
    fn service_id() -> VlcbServiceTypes {
        VlcbServiceTypes::NONE
    }

    /// Returns the service version
    fn service_version() -> u8 {
        0
    }
}