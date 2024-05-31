pub enum Service {
    Mns(vlcb_svc_mns::Service)
}

/// A conversion trait for module services.
pub trait AnyService{
    fn upcast(self) -> Service;
    fn downcast<'c>(service: &'c Service) -> Option<&'c Self>
    where
        Self: Sized;
    fn downcast_mut<'c>(service: &'c mut Service) -> Option<&'c mut Self>
    where
        Self: Sized;
}

macro_rules! from_service {
    ($service:ty, $variant:ident) => {
        impl AnyService for $service {
            fn upcast(self) -> Service {
                Service::$variant(self)
            }

            fn downcast<'c>(socket: &'c Service) -> Option<&'c Self> {
                #[allow(unreachable_patterns)]
                match socket {
                    Service::$variant(socket) => Some(socket),
                    _ => None,
                }
            }

            fn downcast_mut<'c>(socket: &'c mut Service) -> Option<&'c mut Self> {
                #[allow(unreachable_patterns)]
                match socket {
                    Service::$variant(socket) => Some(socket),
                    _ => None,
                }
            }
        }
    };
}

from_service!(vlcb_svc_mns::Service, Mns);