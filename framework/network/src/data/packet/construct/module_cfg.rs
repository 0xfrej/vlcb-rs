pub mod command {
    use vlcb_core::{can::VlcbCanId, vlcb::VlcbNodeNumber};
    use vlcb_defs::{OpCode, CommandError};
    use zerocopy::{ByteOrder, NetworkEndian};
    use super::super::{construct, PacketPayload};

    /// System reset
    ///
    /// Commonly broadcasted to all nodes to indicate a full system reset.
    pub fn restart_all_nodes() -> PacketPayload {
        construct::no_data(OpCode::RestartAllNodes)
    }

    /// Reset node (as in restart)
    ///
    /// Causes module to carry out a software reset to restart the firmware.
    /// No settings are affected.
    pub fn restart(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::RestartNode, bytes[0], bytes[1])
    }

    /// Set Node Number
    ///
    /// Commonly broadcasted to all nodes to indicate a full system reset.
    pub fn set_node_number(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::SetNodeNumber, bytes[0], bytes[1])
    }

    /// Reset to manufacturer's defaults
    ///
    /// Causes the module to reset settings to manufacturers defaults.
    /// The module should retain any node number and remain in FLiM mode.
    /// What the manufacturers defaults are will be defined for each module, but should be
    /// equivalent to putting a new module into FLiM, with no events taught, only default events
    /// defined (if any) and all Nvs returned to their default values.
    pub fn reset_to_factory(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::ResetModuleToFactory, bytes[0], bytes[1])
    }

    /// Request Node number in setup mode
    ///
    /// Sent by a node that is in setup/configuration mode and requests assignment of a node
    /// number (NN). The node allocating node numbers responds with (SNN) which contains the
    /// newly assigned node number. The `node_num` is an the existing node number, if the
    /// node has one. If it does not yet have a node number, you should pass [`None`] into the argument.
    pub fn allocate_node_number(node_num: Option<VlcbNodeNumber>) -> PacketPayload {
            // If it does not yet have a node number, these bytes should be set to zero.
            let mut bytes = [0u8; 2];

            if let Some(nn) = node_num {
                bytes.copy_from_slice(nn.as_bytes());
            }

            construct::two_bytes(OpCode::RequestNewNodeNumber, bytes[0], bytes[1])
    }

    /// Put node into learn mode
    ///
    /// Sent by a configuration tool to take node out of learn mode and revert to normal
    /// operation.
    pub fn start_learn_mode(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::PutNodeIntoLearnMode, bytes[0], bytes[1])
    }

    /// Release node from learn mode
    ///
    /// Sent by a configuration tool to take node out of learn mode and revert to normal
    /// operation.
    pub fn end_learn_mode(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::ReleaseNodeFromLearnMode, bytes[0], bytes[1])
    }

    /// Put node into boot mode
    ///
    /// For SliM nodes with no NN then the NN of the command is must be zero. For SLiM
    /// nodes with an NN, and all FLiM nodes the command must contain the NN of the target
    /// node. Sent by a configuration tool to prepare for loading a new program.
    pub fn reboot_into_bootloader(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::RebootIntoBootloader, bytes[0], bytes[1])
    }

    /// Force can_id self enumeration
    ///
    /// For nodes in FLiM using CAN as transport.. This OPC will force a self-enumeration cycle
    /// for the specified node. A new CAN_ID will be allocated if needed. Following the [`OpCode::ENUM`]
    /// sequence, the node should issue a [`OpCode::NNACK`] to confirm completion and verify the new
    /// CAN_ID. If no CAN_ID values are available, an error message [`CommandError::INVALID_EVENT`] will be issued instead.
    pub fn force_can_enumeration(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::ForceCanEnumeration, bytes[0], bytes[1])
    }

    /// Set a CAN_ID in existing FLiM node
    ///
    /// Used to force a specified CAN_ID into a node. Value range is from 1 to 0x63 (99 decimal)
    /// This OPC must be used with care as duplicate CAN_IDs are not allowed.. Values outside
    /// the permitted range will produce an error 7 message.and the CAN_ID will not change.
    pub fn set_can_id(node_num: VlcbNodeNumber, can_id: VlcbCanId) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::three_bytes(OpCode::SetNodeCanId, bytes[0], bytes[1], can_id.into())
    }

    pub fn set_node_var(node_num: VlcbNodeNumber, nv_index: u8, value: u8) -> PacketPayload {
        /*
            Set a node variable (NVSET)
            Format:
            [<MjPri><MinPri=3><CANID>]<96><NN hi><NN lo><NV# ><NV val>
            Sent by a configuration tool to set a node variable. NV# is the NV index
            number.
        */
        todo!()
    }
}

pub mod query {
    use vlcb_core::vlcb::VlcbNodeNumber;
    use vlcb_defs::OpCode;
    use zerocopy::{ByteOrder, NetworkEndian};
    use super::super::{construct, PacketPayload};

    /// Query node number
    ///
    /// Sent by a node to elicit a PNN reply from each node on the bus that has a node number.
    /// See OpCode 0xB6
    pub fn node_info() -> PacketPayload {
        construct::no_data(OpCode::QueryNodeInfo)
    }

    /// Request node parameters
    ///
    /// Sent to a node while in ‘setup’ mode to read its parameter set. Used
    /// when initially configuring a node. See section 7.2.3 of the CBUS Developer's guide.
    pub fn node_parameters() -> PacketPayload {
        construct::no_data(OpCode::QueryNodeParameters)
    }

    /// Request module type name
    ///
    /// Sent by a node to request the name of the type of module that is in setup mode. The
    /// module in setup mode will reply with opcode NAME. See OpCode 0xE2
    pub fn module_name() -> PacketPayload {
        construct::no_data(OpCode::QueryModuleName)
    }

    /// Request node data event
    ///
    /// Sent by one node to read the data event from another node.(eg: RFID data).
    /// Response is 0xF7 ([`OpCode::ARDAT`]).
    pub fn node_data(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::QueryNodeData, bytes[0], bytes[1])
    }

    /// Request short data frame
    ///
    /// To request a ‘data set’ from a device using the short event method.
    /// Response is 0xFB ([`OpCode::DDRS`])
    pub fn device_data(device_number: u16) -> PacketPayload {
        // TODO: are we sure the `device_number` is not a `node_num` ?
        let mut bytes: [u8; 2] = [0u8; 2];

        NetworkEndian::write_u16(&mut bytes, device_number);

        construct::two_bytes(OpCode::RequestDeviceDataShortMode, bytes[0], bytes[1])
    }

    /// Request read of a node variable
    ///
    /// `index` is the index for the node variable value requested. Response is [`OpCode::NVANS`].
    pub fn node_variable(node_num: VlcbNodeNumber, index: u8) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::three_bytes(OpCode::QueryNodeVariable, bytes[0], bytes[1], index)
    }

    /// Request read of a node parameter by index
    ///
    /// `index` is the index for the parameter requested. Index 0 returns the number of available
    /// parameters.
    /// Response is 0x9B ([`OpCode::PARAN`]) See section 7.2.3 of the
    /// CBUS Developer's guide for details of the node parameters.
    pub fn node_parameter(node_num: VlcbNodeNumber, index: u8) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::three_bytes(OpCode::QueryNodeParameterByIndex, bytes[0], bytes[1], index)
    }
}

pub mod response {
    use vlcb_core::vlcb::VlcbNodeNumber;
    use vlcb_defs::{CommandError, OpCode};
    use super::super::{construct, PacketPayload};

    /// Write acknowledge
    ///
    /// Sent by a node to indicate the completion of a write to memory operation. All nodes must
    /// issue [`OpCode::WRACK`] when a write operation to node variables, events or event variables has
    /// completed. This allows for teaching nodes where the processing time may be slow.
    pub fn write_ack(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::WriteAck, bytes[0], bytes[1])
    }

    /// Error messages from nodes during configuration
    ///
    /// Sent by node if there is an error when a configuration command is sent.
    /// See Section 12.4 of the CBUS developer's guide for details of the error codes.
    pub fn config_error(node_num: VlcbNodeNumber, err: CommandError) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::three_bytes(OpCode::NodeConfigurationError, bytes[0], bytes[1], err.into())
    }

    /// Event space left reply from node
    ///
    /// A one byte value giving the number of available events left in that node.
    pub fn available_event_slots(node_num: VlcbNodeNumber, slots_available: u8) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::three_bytes(OpCode::AvailableEventSlots, bytes[0], bytes[1], slots_available)
    }

    /// Number of events stored in node
    ///
    /// Response to request 0x58 ([`OpCode::RQEVN`])
    pub fn saved_events_amount(node_num: VlcbNodeNumber, saved_events: u8) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::three_bytes(OpCode::LearnedEventCount, bytes[0], bytes[1], saved_events)
    }

    /// Response to a request for a node variable value
    pub fn node_variable() -> PacketPayload {
        /*
            Response to a request for a node variable value (NVANS)
            Format:
            [<MjPri><MinPri=3><CANID>]<97><NN hi><NN lo><NV# ><NV val>
            Sent by node in response to request. (NVRD)
        */
        todo!()
    }

    /// Response to request for individual node parameter
    pub fn node_parameter() -> PacketPayload {
        todo!()
        /*
         * Response to request for individual node parameter (PARAN)
            Format:
            [<MjPri><MinPri=3><CANID>]<9B><NN hi><NN lo><Para#><Para val>
            NN is the node number of the sending node. Para# is the index of the parameter and
            Para val is the parameter value.
        */
    }

    pub fn node_info() -> PacketPayload {
    //             Response to Query Node (PNN)
        // Format:
        // [<MjPri><MinPri=3><CANID>]<B6><NN Hi><NN Lo><Manuf Id><Module Id><Flags>
        // <NN Hi> is the high byte of the node number
        // <NN Lo> is the low byte of the node number
        // <Manuf Id> is the Manufacturer id as defined in the node parameters
        // <Module Id> is the Module Type Id id as defined in the node parameters
        // <Flags> is the node flags as defined in the node parameters, see Section 7.2.3.
        // The Flags byte contains bit flags as follows:
        // Bit 0: Set to 1 for consumer node
        // Bit 1: Set to 1 for producer node
        // Bit 2: Set to 1 for FLiM mode
        // Bit 3: Set to 1 for Bootloader compatible
        // If a module is both a producer and a consumer then it is referred to as a “combi” node and
        // both flags will be set.
        // Every node should send this message in response to a QNN message.
        todo!()
    }

    pub fn node_name() -> PacketPayload {
        /**
         * Format:
            [<MjPri><MinPri=3><CANID>]<E2><char1><char2><char3><char4>
            <char5><char6><char7>
            A node response while in ‘setup’ mode for its name string. Reply to (RQMN). The
            string for the module type is returned in char1 to char7, space filled to 7 bytes. The
            Module Name prefix , currently either CAN or ETH, depends on the Interface Protocol
            parameter, it is not included in the response, see section 7.2.3 for the definition of the
            parameters.
        */
        todo!()
    }

    pub fn node_params() -> PacketPayload {
    //             Response to request for node parameters (PARAMS)
        // Format:
        // [<MjPri><MinPri=3><CANID>]<EF><PARA 1><PARA 2><PARA 3>
        // <PARA 4><PARA 5><PARA 6><PARA 7>
        // A node response while in ‘setup’ mode for its parameter string. Reply to (RQNP)

        // _msg.len = 8;
        //           _msg.data[0] = OPC_PARAMS;    // opcode
        //           _msg.data[1] = _mparams[1];     // manf code -- MERG
        //           _msg.data[2] = _mparams[2];     // minor code ver
        //           _msg.data[3] = _mparams[3]little;     // module ident
        //           _msg.data[4] = _mparams[4];     // number of events
        //           _msg.data[5] = _mparams[5];     // events vars per event
        //           _msg.data[6] = _mparams[6];     // number of NVs
        //           _msg.data[7] = _mparams[7];     // major code ver
        todo!()
    }
}

pub mod ctrl {
    use vlcb_core::vlcb::VlcbNodeNumber;
    use vlcb_defs::OpCode;
    use super::super::{construct, PacketPayload};

    /// Node number release
    ///
    /// Sent by node when taken out of service. e.g. when reverting to SLiM mode.
    pub fn release_node_number(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::NodeNumberReleased, bytes[0], bytes[1])
    }

    /// Node number acknowledge
    ///
    /// Sent by a node to verify its presence and confirm its node id. This message is sent to
    /// acknowledge an [`OpCode::SNN`].
    pub fn ack_node_number(node_num: VlcbNodeNumber) -> PacketPayload {
        let bytes = node_num.as_bytes();
        construct::two_bytes(OpCode::NodeNumberAck, bytes[0], bytes[1])
    }
}
