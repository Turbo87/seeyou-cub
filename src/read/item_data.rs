use crate::error::{Error, Result, Warning};
use crate::read::io::*;
use crate::types::{CubDataId, Header, Item, ItemData, Point};
use std::io::{Read, Seek, SeekFrom};

/// Parse complete item data (geometry + metadata) for an airspace item
pub fn read_item_data<R: Read + Seek>(
    reader: &mut R,
    header: &Header,
    item: &Item,
    warnings: &mut Vec<Warning>,
) -> Result<ItemData> {
    // Calculate offset for first point
    let points_offset = header.data_offset as u64 + item.points_offset as u64;
    reader.seek(SeekFrom::Start(points_offset))?;

    // Initialize origin to item's bottom-left
    let mut origin_x = item.left;
    let mut origin_y = item.bottom;

    let byte_order = header.byte_order();

    let mut points = Vec::new();

    // Phase 1: Parse geometry points
    loop {
        let flag = match read_u8(reader) {
            Ok(flag) => flag,
            Err(Error::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Hit EOF - no attributes section
                return Ok(ItemData {
                    points,
                    name: None,
                    frequency: None,
                    frequency_name: None,
                    icao_code: None,
                    secondary_frequency: None,
                    exception_rules: None,
                    notam_remarks: None,
                    notam_id: None,
                    notam_insert_time: None,
                });
            }
            Err(e) => return Err(e),
        };

        match flag {
            0x81 => {
                // Origin update
                let delta_x = read_i16(reader, byte_order)? as f32 * header.lo_la_scale;
                let delta_y = read_i16(reader, byte_order)? as f32 * header.lo_la_scale;
                origin_x += delta_x;
                origin_y += delta_y;
            }

            0x01 => {
                // Geometry point
                let x = read_i16(reader, byte_order)? as f32 * header.lo_la_scale;
                let y = read_i16(reader, byte_order)? as f32 * header.lo_la_scale;

                let lon = origin_x + x;
                let lat = origin_y + y;

                points.push(Point { lon, lat });
            }

            flag if (flag & 0x40) != 0 => {
                // Attribute section - parse and return
                return parse_attributes(reader, header, flag, warnings, points);
            }

            _ => {
                return Err(Error::UnexpectedPointFlag(flag));
            }
        }
    }
}

struct OptionalFields {
    icao_code: Option<String>,
    secondary_frequency: Option<u32>,
    exception_rules: Option<String>,
    notam_remarks: Option<String>,
    notam_id: Option<String>,
    notam_insert_time: Option<u32>,
}

impl OptionalFields {
    fn new() -> Self {
        Self {
            icao_code: None,
            secondary_frequency: None,
            exception_rules: None,
            notam_remarks: None,
            notam_id: None,
            notam_insert_time: None,
        }
    }
}

/// Parse attribute section starting with given flag
fn parse_attributes<R: Read + Seek>(
    reader: &mut R,
    header: &Header,
    first_flag: u8,
    warnings: &mut Vec<Warning>,
    points: Vec<Point>,
) -> Result<ItemData> {
    let byte_order = header.byte_order();

    let mut name = None;
    let mut frequency = None;
    let mut frequency_name = None;
    let mut optional = OptionalFields::new();

    // First attribute: name
    if (first_flag & 0x40) != 0 {
        // Skip remaining bytes of point structure
        skip_bytes(reader, (header.size_of_point - 1) as usize)?;

        let name_len = (first_flag & 0x3F) as usize;
        if name_len > 0 {
            name = Some(read_string(reader, name_len)?);
        }
    }

    // Check for frequency attribute
    let next_flag = match read_u8(reader) {
        Ok(flag) => flag,
        Err(Error::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            // EOF after name - no frequency or optional data
            return Ok(ItemData {
                points,
                name,
                frequency,
                frequency_name,
                icao_code: optional.icao_code,
                secondary_frequency: optional.secondary_frequency,
                exception_rules: optional.exception_rules,
                notam_remarks: optional.notam_remarks,
                notam_id: optional.notam_id,
                notam_insert_time: optional.notam_insert_time,
            });
        }
        Err(e) => return Err(e),
    };

    if (next_flag & 0xC0) == 0xC0 {
        // Frequency attribute
        let freq_name_len = (next_flag & 0x3F) as usize;
        frequency = Some(read_u32(reader, byte_order)?);

        if freq_name_len > 0 {
            frequency_name = Some(read_string(reader, freq_name_len)?);
        }

        // Read next flag for optional data
        parse_optional_data(reader, warnings, &mut optional)?;
    } else if next_flag == 0xA0 {
        // Optional data without frequency
        parse_optional_data_with_flag(reader, next_flag, warnings, &mut optional)?;
    }
    // If neither frequency nor optional data, we're done (flag not recognized, ignore)

    Ok(ItemData {
        points,
        name,
        frequency,
        frequency_name,
        icao_code: optional.icao_code,
        secondary_frequency: optional.secondary_frequency,
        exception_rules: optional.exception_rules,
        notam_remarks: optional.notam_remarks,
        notam_id: optional.notam_id,
        notam_insert_time: optional.notam_insert_time,
    })
}

/// Parse optional data records (0xA0 flags)
fn parse_optional_data<R: Read + Seek>(
    reader: &mut R,
    warnings: &mut Vec<Warning>,
    optional: &mut OptionalFields,
) -> Result<()> {
    loop {
        let flag = match read_u8(reader) {
            Ok(flag) => flag,
            Err(Error::IoError(ref e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        if flag != 0xA0 {
            // Not optional data, we're done
            return Ok(());
        }

        parse_optional_data_record(reader, warnings, optional)?;
    }
}

/// Parse optional data with flag already read
fn parse_optional_data_with_flag<R: Read + Seek>(
    reader: &mut R,
    flag: u8,
    warnings: &mut Vec<Warning>,
    optional: &mut OptionalFields,
) -> Result<()> {
    if flag == 0xA0 {
        parse_optional_data_record(reader, warnings, optional)?;
        parse_optional_data(reader, warnings, optional)?;
    }
    Ok(())
}

/// Parse single optional data record
fn parse_optional_data_record<R: Read>(
    reader: &mut R,
    warnings: &mut Vec<Warning>,
    optional: &mut OptionalFields,
) -> Result<()> {
    let data_id = read_u8(reader)?;
    let b1 = read_u8(reader)?;
    let b2 = read_u8(reader)?;
    let b3 = read_u8(reader)?;

    let data_id_enum = CubDataId::from_byte(data_id);

    match data_id_enum {
        Some(CubDataId::IcaoCode) => {
            let len = b3 as usize;
            optional.icao_code = Some(read_string(reader, len)?);
        }

        Some(CubDataId::SecondaryFrequency) => {
            let value = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);
            optional.secondary_frequency = Some(value);
        }

        Some(CubDataId::ExceptionRules) => {
            let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
            optional.exception_rules = Some(read_string(reader, len)?);
        }

        Some(CubDataId::NotamRemarks) => {
            let len = (((b2 as u16) << 8) | (b3 as u16)) as usize;
            optional.notam_remarks = Some(read_string(reader, len)?);
        }

        Some(CubDataId::NotamId) => {
            let len = b3 as usize;
            optional.notam_id = Some(read_string(reader, len)?);
        }

        Some(CubDataId::NotamInsertTime) => {
            let b4 = read_u8(reader)?;
            let value =
                ((b1 as u32) << 24) | ((b2 as u32) << 16) | ((b3 as u32) << 8) | (b4 as u32);
            optional.notam_insert_time = Some(value);
        }

        None => {
            warnings.push(Warning::UnknownPointFlag(data_id));
        }
    }

    Ok(())
}
