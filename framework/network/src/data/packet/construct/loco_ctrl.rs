pub mod command {
    use vlcb_core::dcc::{EngineFunctionRange, EngineState};
    use vlcb_defs::{CbusErrs, CbusOpCodes, CbusStmodModes};
    use zerocopy::{ByteOrder, NetworkEndian};
    use super::super::{construct, PacketPayload};
    use heapless::Vec;

    /// Track Off
    ///
    /// Commonly broadcasted to all nodes by a command station to indicate track
    /// power is off and no further command packets should be sent, except inquiries.
    pub fn track_off() -> PacketPayload {
        construct::no_data(CbusOpCodes::TOF)
    }

    /// Track on
    ///
    /// Commonly broadcasted to all nodes by a command station to indicate track power is on.
    pub fn track_on() -> PacketPayload {
        construct::no_data(CbusOpCodes::TON)
    }

    /// Track stopped
    ///
    /// Commonly broadcast to all nodes by a command station to indicate all
    /// engines have been emergency stopped.
    pub fn emergency_stop() -> PacketPayload {
        construct::no_data(CbusOpCodes::ESTOP)
    }

    /// Request track off
    ///
    /// Sent to request change of track power state to “off”.
    pub fn request_track_off() -> PacketPayload {
        construct::no_data(CbusOpCodes::RTOF)
    }

    /// Request track on
    ///
    /// Sent to request change of track power state to “on”.
    pub fn request_track_on() -> PacketPayload {
        construct::no_data(CbusOpCodes::RTON)
    }

    /// Request emergency stop all
    ///
    /// Sent to request an emergency stop to all trains.
    /// Does not affect accessory control.
    /// See section 9.1.8 of the CBUS Developer's guide
    pub fn request_emergency_stop() -> PacketPayload {
        construct::no_data(CbusOpCodes::RESTP)
    }


    /// Release engine by handle
    ///
    /// Sent by a CAB to the Command Station. The engine with that Session
    /// number is removed from the active engine list.
    pub fn release_engine(session_id: u8) -> PacketPayload {
        construct::one_byte(CbusOpCodes::KLOC, session_id)
    }

    /// Session keep alive
    ///
    /// The cab sends a keep alive at regular intervals for the active session. The interval
    /// between keep alive messages must be less than the session timeout implemented by the
    /// command station.
    pub fn session_keep_alive(session_id: u8) -> PacketPayload {
        construct::one_byte(CbusOpCodes::DKEEP, session_id)
    }

    /// Request a new session for loco
    ///
    /// The command station responds with ([`CbusOpCodes::PLOC`]) if engine is free and is being
    /// assigned. Otherwise responds with Err: [`CbusErrs::LOCO_ADDR_TAKEN`] or
    /// Err: [`CbusErrs::LOCO_STACK_FULL`]. This command is typically sent by a cab
    /// to the command station following a change of the controlled decoder address.
    /// [`CbusOpCodes::RLOC`] is exactly equivalent to [`CbusOpCodes::GLOC`] with
    /// all flag bits set to zero, but command stations must continue to support
    /// [`CbusOpCodes::RLOC`] for backwards compatibility.
    pub fn allocate_engine_session(engine_addr: u16) -> PacketPayload {
        let mut payload = [0u8; 2];
        NetworkEndian::write_u16(&mut payload, engine_addr);
        construct::two_bytes(CbusOpCodes::RLOC, payload[0], payload[1])
    }

    /// Allocate loco (used to allocate to a shuttle in cancmd)
    pub fn allocate_loco(session_id: u8, allocation_id: u8) -> PacketPayload {
        construct::two_bytes(CbusOpCodes::ALOC, session_id, allocation_id)
    }

    /// Set Throttle mode
    pub fn set_throttle_mode(
        session_id: u8,
        throttle_mode: CbusStmodModes,
        service_mode: bool,
        sound_control_mode: bool,
    ) -> PacketPayload {
        let mut throttle_mode: u8 = throttle_mode.into();

        if service_mode {
            throttle_mode |= 0x04;
        }

        if sound_control_mode {
            throttle_mode |= 0x08;
        }

        construct::two_bytes(CbusOpCodes::STMOD, session_id, throttle_mode)
    }

    /// Add loco to a consist
    ///
    /// Adds a decoder to a consist.
    /// `consist` has the most significant bit set if consist direction is reversed.
    pub fn add_loco_to_consist(session_id: u8, consist: u8) -> PacketPayload {
        construct::two_bytes(CbusOpCodes::PCON, session_id, consist)
    }

    /// Remove loco from consist
    ///
    /// Removes a loco from a consist.
    pub fn remove_loco_from_consist(session_id: u8, consist: u8) -> PacketPayload {
        construct::two_bytes(CbusOpCodes::KCON, session_id, consist)
    }

    /// Set loco speed and dir
    ///
    /// The speed is an unsigned 7 bit number
    /// Sent by a CAB or equivalent to request an engine speed/dir change.
    pub fn set_loco_speed_dir(session_id: u8, speed: u8, is_reversed: bool) -> PacketPayload {
        let mut data = speed & 0x7F;

        if is_reversed {
            data |= 0x80;
        }

        construct::two_bytes(CbusOpCodes::DSPD, session_id, data)
    }

    /// Set engine flags
    ///
    /// The speed is an unsigned 7 bit number
    /// Sent by a cab to notify the command station of a change in engine flags.
    pub fn set_loco_flags(
        session_id: u8,
        throttle_mode: CbusStmodModes,
        lights_on: bool,
        relative_direction: bool,
        state: EngineState,
    ) -> PacketPayload {
        let mut data: u8 = throttle_mode.into();

        if lights_on {
            data |= 0x04;
        }

        if relative_direction {
            data |= 0x08
        }

        let state: u8 = state.into();
        data |= state << 4u8;

        construct::two_bytes(CbusOpCodes::DFLG, session_id, data)
    }

    /// Set Engine function on
    ///
    /// The `func_num` is an unsigned 7 bit integer
    ///
    /// Sent by a cab to turn on a specific loco function. This provides an alternative method to
    /// [`CbusOpCodes::DFUN`] for controlling loco functions. A command station must implement both methods.
    pub fn loco_func_on(session_id: u8, func_num: u8) -> PacketPayload {
        construct::two_bytes(CbusOpCodes::DFNON, session_id, func_num & 0x7F)
    }
    /// Set Engine function off
    ///
    /// The `func_num` is an unsigned 7 bit integer
    ///
    /// Sent by a cab to turn off a specific loco function. This provides an alternative method to
    /// [`CbusOpCodes::DFUN`] for controlling loco functions. A command station must implement both methods.
    pub fn loco_func_off(session_id: u8, func_num: u8) -> PacketPayload {
        construct::two_bytes(CbusOpCodes::DFNOF, session_id, func_num & 0x7F)
    }

    /// Set engine functions
    ///
    /// Sent by a CAB or equivalent to request an engine Fn state change.
    pub fn set_engine_funcs(
        session_id: u8,
        selection_range: EngineFunctionRange,
        data: u8,
    ) -> PacketPayload {
        construct::three_bytes(CbusOpCodes::DFUN, session_id, selection_range.into(), data)
    }

    /// Request 3-byte DCC Packet
    ///
    /// Requests a packet to be sent onto the track and repeated
    /// `times` amount.
    ///
    /// `times` must be at least of a value 1
    ///
    /// Note: a DCC packet has to be at least 3 and at most 6 octets long
    ///
    /// # Panics
    /// The function panics if `payload` is outside of exactly 3 to 6 octets long
    pub fn send_dcc_packet(times: u8, payload: &[u8]) -> PacketPayload {
        if times < 1 {
            panic!("repeat amount `times` must be greater or equal to 1");
        }

        let payload_len = payload.len();
        if payload_len < 3 || payload_len > 6 {
            panic!(
                "payload slice length ({}) must be at least 3 bytes long and must not be larger than 6",
                payload_len,
            );
        }

        let opc = match payload_len {
            3 => CbusOpCodes::RDCC3,
            4 => CbusOpCodes::RDCC4,
            5 => CbusOpCodes::RDCC5,
            6 => CbusOpCodes::RDCC6,
            _ => unreachable!(),
        };

        // TODO: maybe we could use unchecked because we know it cannot fail
        let mut data: Vec<u8, 8> = Vec::new();
        data.push(opc.into()).unwrap();
        data.push(times).unwrap();
        data.extend_from_slice(payload).unwrap();
        construct::new(data.as_slice())
    }

    pub fn write_cv_data() -> PacketPayload {
        todo!()

        // TODO: these should probably be separate functions
        /*
            Write CV (byte) in OPS mode (WCVO)
            Format:
            [<MjPri><MinPri=2><CANID>]<82><Session><High CV#><Low CV#><Val>
            <Dat1> is the session number of the loco to be written to
            <Dat2> is the MSB # of the CV to be written (supports CVs 1 - 65536)
            <Dat3> is the LSB # of the CV to be written
            <Dat4> is the byte value to be written
            Sent to the command station to write a DCC CV byte in OPS mode to specific loco.(on the
            main)
        */

        /*
        Write CV in Service mode (WCVS)
        Format:
        [<MjPri><MinPri=2><CANID>]<A2><Session><High CV#><LowCV#><Mode>
        <CVval>
        <Dat1> is the session number of the cab
        <Dat2> is the MSB # of the CV to be written (supports CVs 1 - 65536)
        <Dat3> is the LSB # of the CV to be written
        <Dat4> is the service write mode
        <Dat5> is the CV value to be written
        Sent to the command station to write a DCC CV in service mode.
        */

        /*
        Write CV (byte) in OPS mode by address (WCVOA)
        Format:
        [<MjPri><MinPri=2><CANID>]<C1><AddrH><AddrL><High CV#>
        <Low CV#><Mode><Val>
        <Dat1> and <Dat2> are [AddrH] and [AddrL] of the decoder, respectively.
        7 bit addresses have (AddrH=0).
        14 bit addresses have bits 7,8 of AddrH set to 1.
        <Dat3> is the MSB # of the CV to be written (supports CVs 1 - 65536)
        <Dat4> is the LSB # of the CV to be written
        <Dat5> is the programming mode to be used
        <Dat6> is the CV byte value to be written
        Sent to the command station to write a DCC CV byte in OPS mode to specific loco (on the
        main). Used by computer based ops mode programmer that does not have a valid throttle
        handle. */
    }

    pub fn write_cv_flag() -> PacketPayload {
        todo!()

        /*
            Write CV (bit) in OPS mode (WCVB)
            Format:
            [<MjPri><MinPri=2><CANID>]<83><Session><High CV#><Low CV#><Val>
            <Dat1> is the session number of the loco to be written to
            <Dat2> is the MSB # of the CV to be written (supports CVs 1 - 65536)
            <Dat3> is the LSB # of the CV to be written
            <Dat4> is the value to be written
            Reserved
            The format for Dat4 is that specified in RP 9.2.1 for OTM bit manipulation in a DCC
            packet.
            This is ‘111CDBBB’ where C is here is always 1 as only ‘writes’ are possible OTM.
            (unless some loco ACK scheme like RailCom is used). D is the bit value, either 0 or 1
            and BBB is the bit position in the CV byte. 000 to 111 for bits 0 to 7.
            Sent to the command station to write a DCC CV in OPS mode to specific loco.(on
            the main)
        */
    }
    }

    pub mod query {
    use vlcb_defs::{CbusOpCodes, CbusErrs};
    use zerocopy::{AsBytes, ByteOrder, NetworkEndian};
    use super::super::{construct, PacketPayload};
    use vlcb_core::dcc::{LocoAddress, SessionQueryMode};

    /// Request Command Station Status
    ///
    /// Sent to query the status of the command station. See description of ([`CbusOpCodes::STAT`]) for the
    /// response from the command station.
    pub fn command_station_status() -> PacketPayload {
        construct::no_data(CbusOpCodes::RSTAT)
    }

    /// Query engine by handle
    ///
    /// The command station responds with [`CbusOpCodes::PLOC`] if the session is assigned.
    /// Otherwise responds with ERR: [`CbusErrs::LOCO_NOT_FOUND`]. See section 12.5. of the
    /// CBUS Developer's guide.
    pub fn engine_report(session_id: u8) -> PacketPayload {
        construct::one_byte(CbusOpCodes::QLOC, session_id)
    }

    /// Query consist
    ///
    /// Allows enumeration of a consist. Command station responds with [`CbusOpCodes::PLOC`] if an
    /// engine exists at the specified index, otherwise responds with ERR: [`CbusErrs::CONSIST_EMPTY`]
    ///
    /// TODO: check if the returned error is CONSIST_EMPTY or LOCO_NOT_FOUND
    ///
    /// #Note
    /// A command station needs not support this opcode if it uses advanced consisting
    /// and has no way of reading back the CV currently containing the consist address in a loco.
    pub fn enumerate_consist(consist_addr: u8, engine_index: u8) -> PacketPayload {
        construct::two_bytes(CbusOpCodes::QCON, consist_addr, engine_index)
    }

    /// Request engine session
    ///
    /// `use_long_addresses` - set to true if the loco has an 14 bit address
    ///
    /// The command station responds with ([`CbusOpCodes::PLOC`]) if engine is free and is being
    /// assigned. Otherwise responds with (ERR): [`CbusErrs::LOCO_ADDR_TAKEN`]. or (ERR:) [`CbusErrs::LOCO_STACK_FULL`].
    /// This command is typically sent by a cab to the command station following
    /// a change of the controlled decoder address. [`CbusOpCodes::RLOC`] is exactly equivalent to [`CbusOpCodes::GLOC`] with all
    /// flag bits set to zero, but command stations must continue to support [`CbusOpCodes::RLOC`] for backwards
    /// compatibility.
    pub fn engine_session(
        loco_addr: LocoAddress,
    ) -> PacketPayload {
        let addr = loco_addr.as_bytes_sanitized();
        construct::two_bytes(CbusOpCodes::RLOC, addr[0], addr[1])
    }

    /// Get engine session (with support for steal/share)
    ///
    /// With [`SessionQueryMode::Default`] this request behaves as [`CbusOpCodes::RLOC`]
    ///
    /// The command station responds with ([`CbusOpCodes::PLOC`]) if the request is successful.
    /// Otherwise responds with (ERR): [`CbusErrs::LOCO_ADDR_TAKEN`]. (ERR:) [`CbusErrs::LOCO_STACK_FULL`] or (ERR) [`CbusErrs::SESSION_NOT_PRESENT`].
    /// The latter indicates that there is no current session to steal/share depending on the flag
    /// bits set in the request.
    /// GLOC with all flag bits set to zero is exactly equivalent to RLOC, but command stations
    /// must continue to support RLOC for backwards compatibility.
    /// See section 9.1.2. of the CBUS developer's guide for a detailed description of the use of DCC loco sessions
    pub fn engine_session_extended(
        loco_addr: LocoAddress,
        query_mode: SessionQueryMode,
    ) -> PacketPayload {
        let addr = loco_addr.as_bytes_sanitized();

        let flags: u8 = query_mode.into();

        construct::three_bytes(CbusOpCodes::GLOC, addr[0], addr[1], flags)
    }

    pub fn cv_data() -> PacketPayload {
        todo!()

        /*
            Read CV (QCVS)
            Format:
            [<MjPri><MinPri=2><CANID>]<84><Session><High CV#><Low CV#><Mode>
            <Dat1> is the session number of the cab
            <Dat2> is the MSB # of the CV read (supports CVs 1 - 65536)
            <Dat3> is the LSB # of the CV read
            <Dat4> is the programming mode to be used
            This command is used exclusively with service mode.
            Sent by the cab to the command station in order to read a CV value. The command
            station shall respond with a PCVS message containing the value read, or SSTAT if the
            CV cannot be read.
        */
    }

    pub fn cv_report() -> PacketPayload {
        todo!()

        /*
            Report CV (PCVS)
            Format:
            [<MjPri><MinPri=2><CANID>]<85><Session><High CV#><Low CV#><Val>
            <Dat1> is the session number of the cab
            <Dat2> is the MSB # of the CV read (supports CVs 1 - 65536)
            <Dat3> is the LSB # of the CV read
            <Dat4> is the read value
            This command is used exclusively with service mode.
            Sent by the command station to report a read CV.
        */
    }
}

pub mod response {
    use vlcb_defs::{CbusOpCodes};
    use super::super::{construct, PacketPayload};

    /// Service mode status
    ///
    /// Status returned by command station/programmer at end of programming
    /// operation that does not return data.
    pub fn service_mode_status(session_id: u8, status: u8) -> PacketPayload {
        construct::two_bytes(CbusOpCodes::SSTAT, session_id, status)
    }


    pub fn loco_report() -> PacketPayload {
        //                 Engine report (PLOC)
        // Format:
        // [<MjPri><MinPri=2><CANID>]<E1><Session><AddrH><AddrL>
        // <Speed/Dir><Fn1><Fn2><Fn3>
        // <Dat1> Session for engine assigned by the command station. This session
        // number is used in all referenced to the engine until it is released.
        // <Dat2> is the MS byte of the DCC address. For short addresses it is set to 0.
        // <Dat3> is the LS byte of the DCC address. If the engine is consisted, this is the
        // consist address.
        // <Dat4> is the Speed/Direction value. Bit 7 is the direction bit and bits 0-6 are the
        // speed value.
        // <Dat5> is the function byte F0 to F4
        // <Dat6> is the function byte F5 to F8
        // <Dat7> is the function byte F9 to F12
        // A report of an engine entry sent by the command station. Sent in response to
        // QLOC or as an acknowledgement of acquiring an engine requested by a cab
        // (RLOC or GLOC)
    todo!()
    }

    pub fn command_station_report() -> PacketPayload {
        /*
        E3 Command Station status report (STAT)
        Format:
        [<MjPri><MinPri=2><CANID>]<E3><NN hi><NN lo><CS num><flags>
        <Major rev><Minor rev><Build no.>
        <NN hi> <NN lo> Gives node id of command station, so further info can be got from
        parameters or interrogating NVs
        <CS num> For future expansion - set to zero at present
        <flags> Flags as defined below
        <Major rev> Major revision number
        <Minor rev> Minor revision letter
        <Build no.> Build number, always 0 for a released version.
        <flags> is status defined by the bits below.
        bits:
        0 - Hardware Error (self test)
        1 - Track Error
        2 - Track On/ Off
        3 - Bus On/ Halted
        4 - EM. Stop all performed
        5 - Reset done
        6 - Service mode (programming) On/ Off
        7 – reserved
        Sent by the command station in response to RSTAT. */
        todo!()
    }

    pub mod error {
        use vlcb_core::dcc::LocoAddress;
        use vlcb_defs::{CbusErrs, CbusOpCodes};
        use super::super::super::{construct, PacketPayload};

        /// Loco stack full error
        pub fn stack_full(loco_addr: LocoAddress) -> PacketPayload {
            let addr = loco_addr.as_bytes_sanitized();
            construct::three_bytes(CbusOpCodes::ERR, addr[0], addr[1], CbusErrs::LOCO_STACK_FULL.into())
        }

        /// Loco address is already taken
        pub fn addr_taken(loco_addr: LocoAddress) -> PacketPayload {
            let addr = loco_addr.as_bytes_sanitized();
            construct::three_bytes(CbusOpCodes::ERR, addr[0], addr[1], CbusErrs::LOCO_ADDR_TAKEN.into())
        }

        /// Session is not present
        pub fn session_not_found(session_id: u8) -> PacketPayload {
            construct::three_bytes(CbusOpCodes::ERR, session_id, 0, CbusErrs::SESSION_NOT_PRESENT.into())
        }

        /// Consist is empty
        pub fn consist_is_empty(session_id: u8) -> PacketPayload {
            construct::three_bytes(CbusOpCodes::ERR, session_id, 0, CbusErrs::SESSION_NOT_PRESENT.into())
        }

        /// Loco not found
        pub fn loco_not_found(session_id: u8) -> PacketPayload {
            construct::three_bytes(CbusOpCodes::ERR, session_id, 0, CbusErrs::LOCO_NOT_FOUND.into())
        }

        /// CAN bus error
        ///
        /// This would be sent out in the unlikely event that the command
        /// station buffers overflow.
        pub fn can_error() -> PacketPayload {
            construct::three_bytes(CbusOpCodes::ERR, 0, 0, CbusErrs::CMD_RX_BUF_OFLOW.into())
        }

        /// Invalid request
        ///
        /// Indicates an invalid or inconsistent request. For example, a GLOC
        /// request with both steal and share flags set.
        pub fn invalid_request(loco_addr: LocoAddress) -> PacketPayload {
            let addr = loco_addr.as_bytes_sanitized();
            construct::three_bytes(CbusOpCodes::ERR, addr[0], addr[1], CbusErrs::INVALID_REQUEST.into())
        }

        /// Session cancelled
        ///
        /// Sent to a cab to cancel the session when another cab is stealing that session.
        pub fn session_cancelled(session_id: u8) -> PacketPayload {
            construct::three_bytes(CbusOpCodes::ERR, session_id, 0, CbusErrs::SESSION_CANCELLED.into())
        }
    }
}