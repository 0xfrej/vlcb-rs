use vlcb_core::fast_clock::{FastClockMonth, FastClockWeekday};
use vlcb_defs::CbusOpCodes;

use super::{construct, PacketPayload};

/// Bus Halt
///
/// Commonly broadcasted to all nodes to indicate CBUS is not available and no
/// further packets should be sent until a [`CbusOpCodes::BON`] or
/// [`CbusOpCodes::ARST`] is received.
pub fn bus_halt() -> PacketPayload {
    construct::no_data(CbusOpCodes::HLT)
}

/// Bus on
///
/// Commonly broadcasted to all nodes to indicate CBUS is available following a
/// [`CbusOpCodes::HLT`].
pub fn bus_resume() -> PacketPayload {
    construct::no_data(CbusOpCodes::BON)
}

/// Fast Clock
///
/// Used to implement a fast clock for the layout.
///
/// `hours` parameter should be in 24 hour format
///
/// `accel_coefficient` is a time acceleration coefficient value `0` freezes the clock
///     `1` is realtime and `2` and above means accelerated by the factor of N
///
/// # Panics
/// This method panics when `mins` is larger than 59
/// This method panics when `hours` is larger than 23
/// This method panics when `month_day` is outside of 1-31 range (inclusive)
pub fn fast_clock(
    mins: u8,
    hours: u8,
    accel_coefficient: u8,
    week_day: FastClockWeekday,
    month: FastClockMonth,
    month_day: u8,
    temperature: i8,
) -> PacketPayload {
    // TODO: add panics
    let mut wdmon: u8 = week_day.into();

    wdmon |= (month as u8) << 3;

    construct::six_bytes(CbusOpCodes::FCLK, mins, hours, wdmon, accel_coefficient, month_day, temperature as u8)
}

pub mod response {
    use vlcb_defs::CbusOpCodes;

    use super::super::{construct, PacketPayload};

    /// General Acknowledgement
    ///
    /// Positive response to query / request performed or report of availability on-line.
    pub fn ack() -> PacketPayload {
        construct::no_data(CbusOpCodes::ACK)
    }

    /// General No Ack
    ///
    /// Negative response to query / request denied.
    pub fn nack() -> PacketPayload {
        construct::no_data(CbusOpCodes::NAK)
    }
}



/// Helper opcodes that can be used during debugging in development environments.
/// These should never be used in production builds!
pub mod debug {
    use vlcb_defs::CbusOpCodes;
    use super::{construct, PacketPayload};

    /// Debug with one data byte
    ///
    /// The byte is a freeform status value for debugging during CBUS module development.
    /// Not used during normal operation
    pub fn send_debug(data: u8) -> PacketPayload {
        construct::one_byte(CbusOpCodes::DBG1, data)
    }
}