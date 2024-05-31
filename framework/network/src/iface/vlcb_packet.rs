use crate::wire::*;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VlcbPacket<'p> {
    header: VlcbRepr,
    payload: VlcbPayload<'p>,
}

impl<'p> VlcbPacket<'p> {
    pub fn new(vlcb_repr: VlcbRepr, payload: VlcbPayload<'p>) -> Self {
        Self {
            header: vlcb_repr,
            payload,
        }
    }

    pub(crate) fn vlcb_repr(&self) -> VlcbRepr {
        self.header
    }

    pub(crate) fn payload(&self) -> &VlcbPayload {
        &self.payload
    }

    pub(crate) fn emit_payload(&self, vlcb_repr: &VlcbRepr, payload: &mut [u8]) {
        match self.payload() {
            #[cfg(feature = "socket-module")]
            VlcbPayload::Module(inner_payload) => vlcb_repr
                .emit(&mut VlcbPacketWire::new_unchecked(payload), |buf| {
                    buf.copy_from_slice(inner_payload)
                }),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum VlcbPayload<'p> {
    #[cfg(feature = "socket-module")]
    Module(&'p [u8]),
}
