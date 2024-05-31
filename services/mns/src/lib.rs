use vlcb_core::service::VlcbService;

pub struct Service {

}

impl VlcbService for Service {
    fn service_id() -> vlcb_defs::ServiceType {
        vlcb_defs::ServiceType::MinimumNodeService
    }

    fn service_version() -> u8 {
        1
    }
}
