/// Byte ordering for integer fields
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    LE, // Little Endian
    BE, // Big Endian
}

impl ByteOrder {
    /// Parse from PcByteOrder byte (offset 130 in header)
    /// 0 = BE, anything else = LE
    pub fn from_pc_byte_order(byte: u8) -> Self {
        if byte == 0 {
            ByteOrder::BE
        } else {
            ByteOrder::LE
        }
    }
}

/// Airspace style/type (extracted from Item.Type field)
///
/// Indicates the specific type of airspace such as controlled zones,
/// restricted areas, danger areas, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubStyle {
    Unknown,
    ControlZone,
    RestrictedArea,
    ProhibitedArea,
    DangerArea,
    TemporaryReservedArea,
    TerminalControlArea,
    TrafficInformationZone,
    Airway,
    ControlArea,
    GliderSector,
    TransponderMandatoryZone,
    MilitaryAerodromeTrafficZone,
    RadioMandatoryZone,
    Notam,
    Advisory,
    AirDefenceIdentificationZone,
    FlightInformationRegion,
    DelegatedFir,
    TrafficInformationArea,
    SpecialRulesZone,
    TemporaryFlightRestriction,
    AerodromeTrafficZone,
    FlightInformationServiceArea,
    LegacyRmz,
    AerialSportingAndRecreationArea,
    TransponderRecommendedZone,
    VfrRoute,
    Alert,
    TemporarySegregatedArea,
    Warning,
}

impl CubStyle {
    /// Parse from combined Type byte value (lowest 4 bits + highest bit)
    pub fn from_type_byte(byte: u8) -> Self {
        let value = (byte & 0x0F) | (byte & 0x80);
        match value {
            0x00 => CubStyle::Unknown,
            0x01 => CubStyle::ControlZone,
            0x02 => CubStyle::RestrictedArea,
            0x03 => CubStyle::ProhibitedArea,
            0x04 => CubStyle::DangerArea,
            0x05 => CubStyle::TemporaryReservedArea,
            0x06 => CubStyle::TerminalControlArea,
            0x07 => CubStyle::TrafficInformationZone,
            0x08 => CubStyle::Airway,
            0x09 => CubStyle::ControlArea,
            0x0a => CubStyle::GliderSector,
            0x0b => CubStyle::TransponderMandatoryZone,
            0x0c => CubStyle::MilitaryAerodromeTrafficZone,
            0x0d => CubStyle::RadioMandatoryZone,
            0x0f => CubStyle::Notam,
            0x80 => CubStyle::Advisory,
            0x81 => CubStyle::AirDefenceIdentificationZone,
            0x82 => CubStyle::FlightInformationRegion,
            0x83 => CubStyle::DelegatedFir,
            0x84 => CubStyle::TrafficInformationArea,
            0x85 => CubStyle::SpecialRulesZone,
            0x86 => CubStyle::TemporaryFlightRestriction,
            0x87 => CubStyle::AerodromeTrafficZone,
            0x88 => CubStyle::FlightInformationServiceArea,
            0x89 => CubStyle::LegacyRmz,
            0x8a => CubStyle::AerialSportingAndRecreationArea,
            0x8b => CubStyle::TransponderRecommendedZone,
            0x8c => CubStyle::VfrRoute,
            0x8d => CubStyle::Alert,
            0x8e => CubStyle::TemporarySegregatedArea,
            0x8f => CubStyle::Warning,
            _ => CubStyle::Unknown,
        }
    }
}

/// Airspace class (extracted from Item.Type field, bits 5-7)
///
/// ICAO airspace classification (A through G) indicating
/// the level of air traffic control services provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubClass {
    Unknown,
    ClassA,
    ClassB,
    ClassC,
    ClassD,
    ClassE,
    ClassF,
    ClassG,
}

impl CubClass {
    /// Extract from Type byte (bits 5-7)
    pub fn from_type_byte(byte: u8) -> Self {
        let value = (byte >> 4) & 0x07;
        match value {
            0 => CubClass::Unknown,
            1 => CubClass::ClassA,
            2 => CubClass::ClassB,
            3 => CubClass::ClassC,
            4 => CubClass::ClassD,
            5 => CubClass::ClassE,
            6 => CubClass::ClassF,
            7 => CubClass::ClassG,
            _ => CubClass::Unknown,
        }
    }
}

/// Altitude reference style
///
/// Indicates how altitude values should be interpreted:
/// relative to ground level, mean sea level, or as flight levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltStyle {
    Unknown,
    AboveGroundLevel,
    MeanSeaLevel,
    FlightLevel,
    Unlimited,
    Notam,
}

impl AltStyle {
    /// Parse from 4-bit value
    pub fn from_nibble(value: u8) -> Self {
        match value & 0x0F {
            0 => AltStyle::Unknown,
            1 => AltStyle::AboveGroundLevel,
            2 => AltStyle::MeanSeaLevel,
            3 => AltStyle::FlightLevel,
            4 => AltStyle::Unlimited,
            5 => AltStyle::Notam,
            _ => AltStyle::Unknown,
        }
    }
}

/// Extended airspace type (from ExtendedType field if non-zero)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedType {
    UpperInfoRegion,
    MilitaryTrainingRoute,
    HelicopterTrafficZone,
    AreaControlCenterSector,
    LowerTrafficArea,
    UpperTrafficArea,
    MilitaryTrainingArea,
    OverflightRestriction,
    TraTsaFeedingRoute,
    VfrSector,
}

impl ExtendedType {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(ExtendedType::UpperInfoRegion),
            0x02 => Some(ExtendedType::MilitaryTrainingRoute),
            0x03 => Some(ExtendedType::HelicopterTrafficZone),
            0x04 => Some(ExtendedType::AreaControlCenterSector),
            0x05 => Some(ExtendedType::LowerTrafficArea),
            0x06 => Some(ExtendedType::UpperTrafficArea),
            0x07 => Some(ExtendedType::MilitaryTrainingArea),
            0x08 => Some(ExtendedType::OverflightRestriction),
            0x00 => Some(ExtendedType::TraTsaFeedingRoute),
            0x0a => Some(ExtendedType::VfrSector),
            _ => None,
        }
    }
}

/// NOTAM type (from ExtraData bits 28-29)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotamType {
    None,
    Cancel,
    New,
    Replace,
}

impl NotamType {
    pub fn from_bits(bits: u32) -> Self {
        match (bits >> 28) & 0x03 {
            0 => NotamType::None,
            1 => NotamType::Cancel,
            2 => NotamType::New,
            3 => NotamType::Replace,
            _ => NotamType::None,
        }
    }
}

/// NOTAM traffic type (from ExtraData bits 4-6)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotamTraffic {
    Miscellaneous,
    Ifr,
    Vfr,
    IfrAndVfr,
    Checklist,
}

impl NotamTraffic {
    pub fn from_bits(bits: u32) -> Self {
        match (bits >> 4) & 0x07 {
            0 => NotamTraffic::Miscellaneous,
            1 => NotamTraffic::Ifr,
            2 => NotamTraffic::Vfr,
            3 => NotamTraffic::IfrAndVfr,
            4 => NotamTraffic::Checklist,
            _ => NotamTraffic::Miscellaneous,
        }
    }
}

/// NOTAM scope (from ExtraData bits 0-3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotamScope {
    Unknown,
    Aerodrome,
    EnRoute,
    AerodromeAndEnRoute,
    NavWarning,
    AerodromeAndNavWarning,
    Checklist,
}

impl NotamScope {
    pub fn from_bits(bits: u32) -> Self {
        match bits & 0x0F {
            0 => NotamScope::Unknown,
            1 => NotamScope::Aerodrome,
            2 => NotamScope::EnRoute,
            3 => NotamScope::AerodromeAndEnRoute,
            4 => NotamScope::NavWarning,
            5 => NotamScope::AerodromeAndNavWarning,
            8 => NotamScope::Checklist,
            _ => NotamScope::Unknown,
        }
    }
}

/// Days active flags (bits 52-63 of ActiveTime)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaysActive {
    bits: u16,
}

impl DaysActive {
    pub fn from_bits(bits: u16) -> Self {
        Self { bits }
    }

    pub fn sunday(&self) -> bool {
        self.bits & 0x001 != 0
    }
    pub fn monday(&self) -> bool {
        self.bits & 0x002 != 0
    }
    pub fn tuesday(&self) -> bool {
        self.bits & 0x004 != 0
    }
    pub fn wednesday(&self) -> bool {
        self.bits & 0x008 != 0
    }
    pub fn thursday(&self) -> bool {
        self.bits & 0x010 != 0
    }
    pub fn friday(&self) -> bool {
        self.bits & 0x020 != 0
    }
    pub fn saturday(&self) -> bool {
        self.bits & 0x040 != 0
    }
    pub fn holidays(&self) -> bool {
        self.bits & 0x080 != 0
    }
    pub fn aup(&self) -> bool {
        self.bits & 0x100 != 0
    }
    pub fn irregular(&self) -> bool {
        self.bits & 0x200 != 0
    }
    pub fn by_notam(&self) -> bool {
        self.bits & 0x400 != 0
    }
    pub fn is_unknown(&self) -> bool {
        self.bits == 0
    }
}

/// Optional data type identifier in CubPoint sequences
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubDataId {
    IcaoCode,
    SecondaryFrequency,
    ExceptionRules,
    NotamRemarks,
    NotamId,
    NotamInsertTime,
}

impl CubDataId {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(CubDataId::IcaoCode),
            1 => Some(CubDataId::SecondaryFrequency),
            2 => Some(CubDataId::ExceptionRules),
            3 => Some(CubDataId::NotamRemarks),
            4 => Some(CubDataId::NotamId),
            5 => Some(CubDataId::NotamInsertTime),
            _ => None,
        }
    }
}

/// NOTAM subject and action codes (decoded from ExtraData)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotamCodes {
    pub subject: (char, char), // First and last letter
    pub action: (char, char),  // First and last letter
}

impl NotamCodes {
    /// Decode from ExtraData field (bits 8-27 encode letters as 1-26)
    pub fn from_extra_data(extra_data: u32) -> Option<Self> {
        // Check if this is NOTAM data (bits 30-31 == 0)
        if (extra_data >> 30) != 0 {
            return None;
        }

        let decode_letter = |bits: u32| -> Option<char> {
            match bits {
                1..=26 => Some((b'A' + (bits - 1) as u8) as char),
                _ => None,
            }
        };

        let subject_first = decode_letter((extra_data >> 23) & 0x1F)?;
        let subject_last = decode_letter((extra_data >> 18) & 0x1F)?;
        let action_first = decode_letter((extra_data >> 13) & 0x1F)?;
        let action_last = decode_letter((extra_data >> 8) & 0x1F)?;

        Some(NotamCodes {
            subject: (subject_first, subject_last),
            action: (action_first, action_last),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_order_from_pc_byte_order() {
        assert_eq!(ByteOrder::from_pc_byte_order(0), ByteOrder::BE);
        assert_eq!(ByteOrder::from_pc_byte_order(1), ByteOrder::LE);
        assert_eq!(ByteOrder::from_pc_byte_order(255), ByteOrder::LE);
    }

    #[test]
    fn cub_style_from_type_byte() {
        assert_eq!(CubStyle::from_type_byte(0x00), CubStyle::Unknown);
        assert_eq!(CubStyle::from_type_byte(0x01), CubStyle::ControlZone);
        assert_eq!(CubStyle::from_type_byte(0x04), CubStyle::DangerArea);
        assert_eq!(CubStyle::from_type_byte(0x80), CubStyle::Advisory);
        assert_eq!(CubStyle::from_type_byte(0x8f), CubStyle::Warning);
    }

    #[test]
    fn cub_class_from_type_byte() {
        assert_eq!(CubClass::from_type_byte(0b00000000), CubClass::Unknown);
        assert_eq!(CubClass::from_type_byte(0b00010000), CubClass::ClassA);
        assert_eq!(CubClass::from_type_byte(0b01000000), CubClass::ClassD);
        assert_eq!(CubClass::from_type_byte(0b01110000), CubClass::ClassG);
    }

    #[test]
    fn alt_style_from_nibble() {
        assert_eq!(AltStyle::from_nibble(0), AltStyle::Unknown);
        assert_eq!(AltStyle::from_nibble(1), AltStyle::AboveGroundLevel);
        assert_eq!(AltStyle::from_nibble(3), AltStyle::FlightLevel);
        assert_eq!(AltStyle::from_nibble(15), AltStyle::Unknown);
    }

    #[test]
    fn notam_type_from_bits() {
        assert_eq!(NotamType::from_bits(0b00 << 28), NotamType::None);
        assert_eq!(NotamType::from_bits(0b01 << 28), NotamType::Cancel);
        assert_eq!(NotamType::from_bits(0b10 << 28), NotamType::New);
        assert_eq!(NotamType::from_bits(0b11 << 28), NotamType::Replace);
    }

    #[test]
    fn days_active() {
        let days = DaysActive::from_bits(0x001 | 0x004 | 0x040);
        assert!(days.sunday());
        assert!(!days.monday());
        assert!(days.tuesday());
        assert!(days.saturday());
        assert!(!days.holidays());
    }

    #[test]
    fn notam_codes_decode() {
        // Example: subject "AA", action "BB"
        let extra_data = (1 << 23) |  // subject first: A
            (1 << 18) |  // subject last: A
            (2 << 13) |  // action first: B
            (2 << 8); // action last: B

        let codes = NotamCodes::from_extra_data(extra_data).unwrap();
        assert_eq!(codes.subject, ('A', 'A'));
        assert_eq!(codes.action, ('B', 'B'));
    }
}
