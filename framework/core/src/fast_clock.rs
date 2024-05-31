use num_enum::{FromPrimitive, IntoPrimitive};

/// Week day for fast clock implementation
///
/// The enum values represent the VLCB fast clock protocol specification
/// for week days.
///
/// Default value is `1` ([`FastClockWeekday::Sunday`])
#[derive(FromPrimitive, IntoPrimitive, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum FastClockWeekday {
    #[default]
    Sunday = 1,
    Monday = 2,
    Tuesday = 3,
    Wednesday = 4,
    Thursday = 5,
    Friday = 6,
    Saturday = 7,
}

/// Month for fast clock implementation
///
/// The enum values represent the VLCB fast clock protocol specification
/// for months.
///
/// Default value is `1` ([`FastClockMonth::January`])
#[derive(FromPrimitive, IntoPrimitive, Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum FastClockMonth {
    #[default]
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}
