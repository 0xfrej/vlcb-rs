use vlcb_core::service::VlcbService;

pub struct Service {

}

impl VlcbService for Service {
    fn service_id() -> vlcb_defs::VlcbServiceTypes {
        vlcb_defs::VlcbServiceTypes::MNS
    }

    fn service_version() -> u8 {
        1
    }
}
