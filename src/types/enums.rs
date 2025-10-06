/// Byte ordering for integer fields
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    LE,  // Little Endian
    BE,  // Big Endian
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
}
