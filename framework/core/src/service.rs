use vlcb_defs::ServiceType;

pub trait VlcbService {
    /// Runs the service initialization
    #[must_use]
    fn init() {}

    /// Returns the service ID
    ///
    /// By default it returns [`ServiceType::Internal`] which means that the service
    /// is should not communicate
    fn service_id() -> ServiceType {
        ServiceType::Internal
    }

    /// Returns the service version
    fn service_version() -> u8 {
        0
    }
}
