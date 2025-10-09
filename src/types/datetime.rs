/// Simple date and time representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTime {
    pub day: u8,
    pub month: u8,
    pub year: u32,
    pub hour: u8,
    pub minute: u8,
}
