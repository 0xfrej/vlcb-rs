use super::DispatchError;
use super::InterfaceInner;
use super::{check, PollContext};
use crate::iface::vlcb_packet::VlcbPacket;
use embedded_time::{Clock, Instant};

use crate::phy::{Device, TxToken};
use crate::iface::socket_set::SocketSet;
use crate::wire::{CanFrame, VlcbPacketWire};


// TODO: when sending any packets we need to add priority to them!!!

/*pub(super) enum CanControlEvent<C: Clock> {
    Poll,
    RequestEnumeration { now: Instant<C> },
}

#[derive(Debug, Copy, Clone, Default)]
pub(super) enum CanControlState<C: Clock> {
    #[default]
    Idle,
    StartingEnumeration {
        started_at: Instant<C>,
    },
    Enumerating {
        started_at: Instant<C>,
        responses: u128,
    },
}

impl<C: Clock> CanControlState<C> {
    pub(super) fn consume(self, evt: CanControlEvent<C>) -> Self {
        match (self, evt) {
            (Self::Idle, CanControlEvent::RequestEnumeration { now }) => {
                Self::StartingEnumeration { started_at: now }
            }
            (Self::StartingEnumeration { started_at }, CanControlEvent::Poll) => {
                Self::Enumerating {
                    started_at,
                    // CAN protocol should choose the lowest vacant value, but ID 0 is reserved
                    // for SLiM mode consumer nodes so by default we need to start at 1.
                    responses: 1,
                }
            }
            (x, _) => x,
        }
    }
}*/

impl<C: Clock> InterfaceInner<C> {
    #[cfg(feature = "medium-can")]
    pub(super) fn process_can<'frame>(
        &mut self,
        sockets: &mut SocketSet<'_>,
        frame: &[u8],
    ) -> Option<VlcbPacket<'frame>> {
        let can_frame = check!(CanFrame::new_checked(frame));

        let vlcb_packet = check!(VlcbPacketWire::new_checked(can_frame.payload()));

        /*

           //
          /// set flag if we find a CANID conflict with the frame's producer
          /// doesn't apply to RTR or zero-length frames, so as not to trigger an enumeration loop
          //


          //
          /// extract the CANID of the sending module
          //

          remoteCANID = getCANID(_msg.id);

        // start bus enumeration if required
        if (enumeration_required) {
          enumeration_required = false;
          CANenumeration();
        }

        // check CAN bus enumeration timer
        checkCANenum();


          // is this a CANID enumeration request from another node (RTR set) ?
          if (_msg.rtr) {
            // DEBUG_SERIAL << F("> CANID enumeration RTR from CANID = ") << remoteCANID << endl;
            // send an empty message to show our CANID
            _msg.len = 0;
            sendMessage(&_msg);
            continue;
          }

        if (remoteCANID == module_config->CANID && _msg.len > 0) {
            // DEBUG_SERIAL << F("> CAN id clash, enumeration required") << endl;
            enumeration_required = true;
          }

          // are we enumerating CANIDs ?
          if (bCANenum && _msg.len == 0) {

            // store this response in the responses array
            if (remoteCANID > 0) {
              // fix to correctly record the received CANID
              bitWrite(enum_responses[(remoteCANID / 16)], remoteCANID % 8, 1);
              // DEBUG_SERIAL << F("> stored CANID ") << remoteCANID << F(" at index = ") << (remoteCANID / 8) << F(", bit = ") << (remoteCANID % 8) << endl;
            }

            continue;
          }

          switch OPC from frame
          case OPC_CANID:
              // CAN -- set CANID
              // DEBUG_SERIAL << F("> CANID for nn = ") << nn << F(" with new CANID = ") << _msg.data[3] << endl;

              if (nn == module_config->nodeNum) {
                // DEBUG_SERIAL << F("> setting my CANID to ") << _msg.data[3] << endl;
                if (_msg.data[3] < 1 || _msg.data[3] > 99) {
                  sendCMDERR(7);
                } else {
                  module_config->setCANID(_msg.data[3]);
                }
              }

              break;

        case OPC_ENUM:
          // received ENUM -- start CAN bus self-enumeration
          // DEBUG_SERIAL << F("> ENUM message for nn = ") << nn << F(" from CANID = ") << remoteCANID << endl;
          // DEBUG_SERIAL << F("> my nn = ") << module_config->nodeNum << endl;

          if (nn == module_config->nodeNum && remoteCANID != module_config->CANID && !bCANenum) {
            // DEBUG_SERIAL << F("> initiating enumeration") << endl;
            CANenumeration();
          }

          break;

        */

        // self.process_cbus(sockets, &cbus_packet)
        todo!()
    }

    // #[cfg(feature = "medium-can")]
    // pub(super) fn dispatch_can<D, Tx, F>(
    //     &mut self,
    //     tx_token: Tx,
    //     buffer_len: usize,
    //     f: F,
    // ) -> Result<(), DispatchError>
    // where
    //     D: Device,
    //     Tx: TxToken,
    //     F: FnOnce(CanFrame<&mut [u8]>),
    // {
    //     let tx_len = CanFrame::<&[u8]>::buffer_len(buffer_len);
    //     tx_token.consume(tx_len, |tx_buffer| {
    //         debug_assert!(tx_buffer.as_ref().len() == tx_len);
    //         let mut frame = CanFrame::new_unchecked(tx_buffer);

    //         let src_addr = self.hw_addr.can_or_panic();
    //         frame.set_src_addr(src_addr);

    //         f(frame);

    //         Ok(())
    //     })
    // }
}
