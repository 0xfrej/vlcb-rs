use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct NodeFlags: u8 {
        const Heartbeat = 0b00000001;
        const EventAck = 0b00000010;
    }
}