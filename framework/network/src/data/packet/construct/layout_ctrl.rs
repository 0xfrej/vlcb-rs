pub mod produce {
  use vlcb_core::cbus::{EventId, EventType};
  use vlcb_defs::CbusOpCodes;
  use heapless::Vec;

  use super::super::{construct, PacketPayload};

  /// Accessory event
  ///
  /// Indicates a event status using the full event number of 4 bytes.
  /// Produces different opcodes depending on the event type (long/short)
  ///
  /// Depending on the `event_type` the produced packet will either be of accessory on, accessory response on
  /// or the "off" counterparts.
  ///
  /// `payload` can be either omitted or have a size no less than one and no more than 3
  /// The payload should already be formatted in big endian
  ///
  /// If `response` is specified as `true` the packet sent will be only a response.
  /// This is used to respond to event requests such as [`CbusOpCodes::AREQ`]
  ///
  /// # Panics
  /// If payload has greater lenght than 3 and less than 1
  pub fn accessory(event_type: EventType, event: EventId, payload: Option<&[u8]>) -> PacketPayload {
      if let Some(payload) = payload {
          let l = payload.len();
          if l < 1 || l > 3 {
              panic!(
                  "payload should not be bigger than 3 octets and smaller than 1, given ({})",
                  l
              );
          }
      }

      let opc = match (event_type, event.is_short(), payload.map_or(0, |v| v.len())) {
          (EventType::AccessoryOn, false, 0) => CbusOpCodes::ACON,
          (EventType::AccessoryOn, false, 1) => CbusOpCodes::ACON1,
          (EventType::AccessoryOn, false, 2) => CbusOpCodes::ACON2,
          (EventType::AccessoryOn, false, 3) => CbusOpCodes::ACON3,
          (EventType::AccessoryOn, true, 0) => CbusOpCodes::ASON,
          (EventType::AccessoryOn, true, 1) => CbusOpCodes::ASON1,
          (EventType::AccessoryOn, true, 2) => CbusOpCodes::ASON2,
          (EventType::AccessoryOn, true, 3) => CbusOpCodes::ASON3,
          (EventType::AccessoryStatusOn, false, 0) => CbusOpCodes::ARON,
          (EventType::AccessoryStatusOn, false, 1) => CbusOpCodes::ARON1,
          (EventType::AccessoryStatusOn, false, 2) => CbusOpCodes::ARON2,
          (EventType::AccessoryStatusOn, false, 3) => CbusOpCodes::ARON3,
          (EventType::AccessoryStatusOn, true, 0) => CbusOpCodes::ARSON,
          (EventType::AccessoryStatusOn, true, 1) => CbusOpCodes::ARSON1,
          (EventType::AccessoryStatusOn, true, 2) => CbusOpCodes::ARSON2,
          (EventType::AccessoryStatusOn, true, 3) => CbusOpCodes::ARSON3,

          (EventType::AccessoryOff, false, 0) => CbusOpCodes::ACOF,
          (EventType::AccessoryOff, false, 1) => CbusOpCodes::ACOF1,
          (EventType::AccessoryOff, false, 2) => CbusOpCodes::ACOF2,
          (EventType::AccessoryOff, false, 3) => CbusOpCodes::ACOF3,
          (EventType::AccessoryOff, true, 0) => CbusOpCodes::ASOF,
          (EventType::AccessoryOff, true, 1) => CbusOpCodes::ASOF1,
          (EventType::AccessoryOff, true, 2) => CbusOpCodes::ASOF2,
          (EventType::AccessoryOff, true, 3) => CbusOpCodes::ASOF3,
          (EventType::AccessoryStatusOff, false, 0) => CbusOpCodes::AROF,
          (EventType::AccessoryStatusOff, false, 1) => CbusOpCodes::AROF1,
          (EventType::AccessoryStatusOff, false, 2) => CbusOpCodes::AROF2,
          (EventType::AccessoryStatusOff, false, 3) => CbusOpCodes::AROF3,
          (EventType::AccessoryStatusOff, true, 0) => CbusOpCodes::ARSOF,
          (EventType::AccessoryStatusOff, true, 1) => CbusOpCodes::ARSOF1,
          (EventType::AccessoryStatusOff, true, 2) => CbusOpCodes::ARSOF2,
          (EventType::AccessoryStatusOff, true, 3) => CbusOpCodes::ARSOF3,
          _ => unreachable!()
      };

      //TODO: maybe use unchecked instead
      let mut data: Vec<u8, 8> = Vec::new();
      data.push(opc.into()).unwrap();
      data.extend_from_slice(event.as_bytes()).unwrap();
      construct::new(data.as_slice())
  }

  pub fn accessory_data() -> PacketPayload {
    todo!()
    /*
    Accessory node data event (ACDAT)
    Format:
    [<MjPri><MinPri=3><CANID>]<F6><NN hi><NNlo>
    <data1><data2><data3><data4><data5>
    <Dat1> is the high byte of the node number
    <Dat2> is the low byte of the node number
    <Dat3> is the first node data byte
    <Dat4> is the second node data byte
    <Dat5> is the third node data byte
    <Dat6> is the fourth node data byte
    <Dat7> is the fifth node data byte
    Indicates an event from this node with 5 bytes of data.
    For example, this can be used to send the 40 bits of an RFID tag. There is no
    event number in order to allow space for 5 bytes of data in the packet, so there
    can only be one data event per node. */
    /*
    Device data event (short mode) (DDES)
    Format:
    [<MjPri><MinPri=3><CANID>]<FA><DN hi><DN lo>
    <data1><data2><data3><data4><data5>
    <Dat1> is the high byte of the device number
    <Dat2> is the low byte of the device number
    <Dat3> is the first device data byte
    <Dat4> is the second device data byte
    <Dat5> is the third device data byte
    <Dat6> is the fourth device data byte
    <Dat7> is the fifth device data byte
    Function is the same as F6 but uses device addressing so can relate data to a
    device attached to a node. e.g. one of several RFID readers attached to a single
    node. */
  }
}
pub mod command {
  use vlcb_core::cbus::EventId;
  use vlcb_defs::CbusOpCodes;

  use super::super::{construct, PacketPayload};

  /// Unlearn an event in learn mode
  ///
  /// Sent by a configuration tool to remove an event from a node.
  /// # Panics
  /// This method panics if the event is short
  pub fn unlearn(event: EventId) -> PacketPayload {
      if event.is_short() {
          panic!("The event must be long");
      }
      let data = event.as_bytes();
      construct::four_bytes(CbusOpCodes::EVULN, data[0], data[1], data[2], data[3])
  }

  pub fn teach() -> PacketPayload {
      /*
      Teach an event in learn mode (EVLRN)
      Format:
      [<MjPri><MinPri=3><CANID>]<D2><NN hi><NN lo><EN hi><EN lo>
      <EV#><EV val>
      Sent by a configuration tool to a node in learn mode to teach it an event. Also
      teaches it the associated event variables (EVs) by the EV index (EV#). This
      command is repeated for each EV required */

      /*
       * Teach an event in learn mode using event indexing (EVLRNI)
          Format:
          [<MjPri><MinPri=3><CANID>]<F5><NN hi><NN lo><EN hi><EN lo>
          <EN#><EV#><EV val>
          Sent by a configuration tool to a node in learn mode to teach it an event. The
          event index must be known. Also teaches it the associated event variables.(EVs).
          This command is repeated for each EV required.
       */
      todo!()
  }
}
pub mod query {
  use vlcb_core::cbus::EventId;
  use vlcb_defs::CbusOpCodes;
  use super::super::{construct, PacketPayload};

  /// Accessory Request Event
  ///
  /// Indicates a ‘request’ event using the full event number of 4 bytes. (long event)
  /// A request event is used to elicit a status response from a producer when it is required to
  /// know the ‘state’ of the producer without producing an ON or OFF event and to trigger an
  /// event from a ‘combi’ node.
  pub fn accessory(event: EventId) -> PacketPayload {
      let opc = match event.is_short() {
          true => CbusOpCodes::ASRQ,
          false => CbusOpCodes::AREQ,
      };

      let data = event.as_bytes();
      //TODO: check if we need to set ms bytes of the event if it's short to 0
      construct::four_bytes(opc, data[0], data[1], data[2], data[3])
  }

  /// Request for read of an event variable
  pub fn event_variable() -> PacketPayload {
      /**
       * Request for read of an event variable (REVAL)
      Format:
      [<MjPri><MinPri=3><CANID>]<9C><NN hi><NN lo><EN#><EV#>
      This request differs from B2 (REQEV) as it doesn’t need to be in learn mode but does
      require the knowledge of the event index to which the EV request is directed.
      EN# is the event index. EV# is the event variable index. Response is B5 (NEVAL)
       */

      /**
       * Read event variable in learn mode (REQEV)
Format:
[<MjPri><MinPri=3><CANID>]<B2><NN hi><NN lo><EN hi>
<EN lo><EV# >
Allows a configuration tool to read stored event variables from a node. EV# is the
EV index. Reply is (EVANS)
       */
      todo!()
  }
}
pub mod response {
  use super::super::{construct, PacketPayload};

  /// Response to request for read of EV value
  pub fn event_variable() -> PacketPayload {
      // TODO: should probably be separate methods
      /**
       * Response to request for read of EV value (NEVAL)
          Format:
          [<MjPri><MinPri=3><CANID>]<B5><NN hi><NN lo><EN#>
          <EV#><EVval>
          NN is the node replying. EN# is the index of the event in that node. EV# is the index of the
          event variable. EVval is the value of that EV. This is response to 9C (REVAL)
       */
      todo!()

      /*
       * Format:
      [<MjPri><MinPri=3><CANID>]<D3><NN hi><NN lo><EN hi><EN lo>
      <EV#><EV val>
      A node response to a request from a configuration tool for the EVs associated
      with an event (REQEV). For multiple EVs, there will be one response per request.
       */
  }

  pub fn event() -> PacketPayload {
      /*
      Response to request to read node events (ENRSP)
      Format:
      [<MjPri><MinPri=3><CANID>]<F2><NN hi><NN lo>
      <EN3><EN2><EN1><EN0><EN#>
      Where the NN is that of the sending node. EN3 to EN0 are the four bytes of the stored
      event. EN# is the index of the event within the sending node. This is a response to either
      57 (NERD) or 72 (NENRD) */
      todo!()
  }

  pub fn accessory_node_data() -> PacketPayload {
//     Accessory node data Response (ARDAT)
// Format:
// [<MjPri><MinPri=3><CANID>]<F7><NN hi><NN lo>
// <data1><data2><data3><data4><data5>
// <Dat1> is the high byte of the node number
// <Dat2> is the low byte of the node number
// <Dat3> is the first node data byte
// <Dat4> is the second node data byte
// <Dat5> is the third node data byte
// <Dat6> is the fourth node data byte
// <Dat7> is the fifth node data byte
// Indicates a node data response. A response event is a reply to a status request
// (RQDAT) without producing a new data event.

// Device data response (short mode) (DDRS)
// Format:
// [<MjPri><MinPri=3><CANID>]<FB><DN hi><DN lo>
// <data1><data2><data3><data4><data5>
// <Dat1> is the high byte of the device number
// <Dat2> is the low byte of the device number
// <Dat3> is the first device data byte
// <Dat4> is the second device data byte
// <Dat5> is the third device data byte
// <Dat6> is the fourth device data byte
// <Dat7> is the fifth device data byte
// The response to a request for data from a device. (0x5B)
todo!()
  }
}