use crate::error::Result;
use crate::utils::io::{write_i16, write_u8};
use crate::{ByteOrder, Point};
use std::io::Write;

pub const POINT_OP_MOVE_ORIGIN: u8 = 0x81;
pub const POINT_OP_NEW_POINT: u8 = 0x01;

/// Raw point operation with i16 coordinates
///
/// Point operations are encoded in the CUB file as x/y offsets that must be:
/// 1. Accumulated relative to a movable origin
/// 2. Converted to lat/lon via lo_la_scale multiplication
///
/// This enum represents the raw operations before conversion.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PointOp {
    /// Move the origin to a new position relative to the current origin
    MoveOrigin { x: i16, y: i16 },
    /// Add a new point relative to the current origin
    NewPoint { x: i16, y: i16 },
}

impl std::fmt::Debug for PointOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PointOp::MoveOrigin { x, y } => write!(f, "MoveOrigin {{ x: {x:?}, y: {y:?} }}"),
            PointOp::NewPoint { x, y } => write!(f, "NewPoint {{ x: {x:?}, y: {y:?} }}"),
        }
    }
}

impl PointOp {
    /// Convert geographic points to point operations
    ///
    /// Converts a sequence of Point coordinates (lat/lon in radians) into raw i16 offset
    /// operations suitable for CUB file storage. Automatically inserts MoveOrigin operations
    /// when offsets exceed i16 range.
    ///
    /// # Arguments
    ///
    /// * `points` - Sequence of points with lat/lon in radians
    /// * `lo_la_scale` - Scaling factor (converts radians to i16)
    /// * `origin_lon` - Initial longitude origin in radians
    /// * `origin_lat` - Initial latitude origin in radians
    ///
    /// # Returns
    ///
    /// Vector of point operations ready for file writing
    pub fn from_points(
        points: &[Point],
        lo_la_scale: f32,
        origin_lon: f32,
        origin_lat: f32,
    ) -> Result<Vec<PointOp>> {
        let mut ops = Vec::new();
        let mut current_origin_lon = origin_lon;
        let mut current_origin_lat = origin_lat;

        for point in points {
            // Keep moving origin until point fits in i16 range
            loop {
                let lon_offset = (point.lon - current_origin_lon) / lo_la_scale;
                let lat_offset = (point.lat - current_origin_lat) / lo_la_scale;

                // Check if offset fits in i16 range
                if lon_offset >= i16::MIN as f32
                    && lon_offset <= i16::MAX as f32
                    && lat_offset >= i16::MIN as f32
                    && lat_offset <= i16::MAX as f32
                {
                    // Offset fits - emit NewPoint and move to next point
                    ops.push(PointOp::NewPoint {
                        x: lon_offset.round() as i16,
                        y: lat_offset.round() as i16,
                    });
                    break;
                }

                // Offset too large - move origin closer by i16::MAX steps
                let move_x = lon_offset.clamp(i16::MIN as f32, i16::MAX as f32).round() as i16;
                let move_y = lat_offset.clamp(i16::MIN as f32, i16::MAX as f32).round() as i16;

                ops.push(PointOp::MoveOrigin {
                    x: move_x,
                    y: move_y,
                });

                current_origin_lon += move_x as f32 * lo_la_scale;
                current_origin_lat += move_y as f32 * lo_la_scale;
            }
        }

        Ok(ops)
    }

    /// Resolve point operations into geographic coordinates
    ///
    /// Processes a sequence of point operations (origin moves and new points) and converts
    /// them from raw i16 offsets to f32 lat/lon coordinates in radians.
    ///
    /// # Arguments
    ///
    /// * `point_ops` - Sequence of point operations from raw file parsing
    /// * `lo_la_scale` - Scaling factor from header (converts i16 to radians)
    /// * `origin_lon` - Initial longitude origin in radians (from item.left)
    /// * `origin_lat` - Initial latitude origin in radians (from item.bottom)
    ///
    /// # Returns
    ///
    /// Vector of resolved points in radians, or error if any coordinate is out of range
    pub fn resolve(
        point_ops: &[PointOp],
        lo_la_scale: f32,
        mut origin_lon: f32,
        mut origin_lat: f32,
    ) -> Result<Vec<Point>> {
        let mut points = Vec::new();

        for op in point_ops {
            match op {
                PointOp::MoveOrigin { x, y } => {
                    // Update origin by scaled offset (stay in radians)
                    origin_lon += (*x as f32) * lo_la_scale;
                    origin_lat += (*y as f32) * lo_la_scale;
                }
                PointOp::NewPoint { x, y } => {
                    // Calculate point position in radians
                    let lon = origin_lon + (*x as f32) * lo_la_scale;
                    let lat = origin_lat + (*y as f32) * lo_la_scale;

                    let point = Point::lat_lon(lat, lon);
                    if !point.is_valid() {
                        return Err(crate::error::Error::CoordinateOutOfRange { point });
                    }

                    points.push(point);
                }
            }
        }

        Ok(points)
    }

    pub fn write<W: Write>(&self, writer: &mut W, byte_order: ByteOrder) -> std::io::Result<()> {
        match self {
            PointOp::MoveOrigin { x, y } => {
                write_u8(writer, POINT_OP_MOVE_ORIGIN)?;
                write_i16(writer, *x, byte_order)?;
                write_i16(writer, *y, byte_order)
            }
            PointOp::NewPoint { x, y } => {
                write_u8(writer, POINT_OP_NEW_POINT)?;
                write_i16(writer, *x, byte_order)?;
                write_i16(writer, *y, byte_order)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_lt;
    use insta::{assert_compact_debug_snapshot, assert_debug_snapshot};

    #[test]
    fn point_op_size() {
        // PointOp should be small (enum discriminant + 2 x i16)
        assert_eq!(std::mem::size_of::<PointOp>(), 6);
    }

    #[test]
    fn point_op_construction() {
        let origin = PointOp::MoveOrigin { x: 100, y: 200 };
        let point = PointOp::NewPoint { x: 10, y: 20 };

        match origin {
            PointOp::MoveOrigin { x, y } => {
                assert_eq!(x, 100);
                assert_eq!(y, 200);
            }
            _ => panic!("Expected MoveOrigin"),
        }

        match point {
            PointOp::NewPoint { x, y } => {
                assert_eq!(x, 10);
                assert_eq!(y, 20);
            }
            _ => panic!("Expected NewPoint"),
        }
    }

    #[test]
    fn resolve_simple_points() {
        let ops = vec![
            PointOp::NewPoint { x: 100, y: 200 },
            PointOp::NewPoint { x: 150, y: 250 },
        ];

        let scale = 0.0001;
        let origin_lon = 0.1; // radians
        let origin_lat = 0.2; // radians

        let points = PointOp::resolve(&ops, scale, origin_lon, origin_lat).unwrap();
        assert_eq!(points.len(), 2);

        // Verify: origin + (offset * scale) in radians
        let expected_lon_1 = 0.1_f32 + 100.0 * 0.0001;
        let expected_lat_1 = 0.2_f32 + 200.0 * 0.0001;

        assert_lt!((points[0].lon - expected_lon_1).abs(), 1e-6);
        assert_lt!((points[0].lat - expected_lat_1).abs(), 1e-6);

        let expected_lon_2 = 0.1_f32 + 150.0 * 0.0001;
        let expected_lat_2 = 0.2_f32 + 250.0 * 0.0001;

        assert_lt!((points[1].lon - expected_lon_2).abs(), 1e-6);
        assert_lt!((points[1].lat - expected_lat_2).abs(), 1e-6);
    }

    #[test]
    fn resolve_with_origin_move() {
        let ops = vec![
            PointOp::NewPoint { x: 100, y: 200 },
            PointOp::MoveOrigin { x: 1000, y: 2000 },
            PointOp::NewPoint { x: 50, y: 100 },
        ];

        let scale = 0.0001;
        let origin_lon = 0.0;
        let origin_lat = 0.0;

        let points = PointOp::resolve(&ops, scale, origin_lon, origin_lat).unwrap();
        assert_eq!(points.len(), 2);

        // The first point should be relative to passed-in origin in radians
        let expected_lon_1 = 100.0_f32 * 0.0001;
        let expected_lat_1 = 200.0_f32 * 0.0001;

        assert_lt!((points[0].lon - expected_lon_1).abs(), 1e-6);
        assert_lt!((points[0].lat - expected_lat_1).abs(), 1e-6);

        // The second point should be relative to the moved origin in radians
        let expected_lon_2 = 1000.0_f32 * 0.0001 + 50.0 * 0.0001;
        let expected_lat_2 = 2000.0_f32 * 0.0001 + 100.0 * 0.0001;

        assert_lt!((points[1].lon - expected_lon_2).abs(), 1e-6);
        assert_lt!((points[1].lat - expected_lat_2).abs(), 1e-6);
    }

    #[test]
    fn resolve_out_of_range_fails() {
        let ops = vec![
            PointOp::NewPoint { x: 10000, y: 20000 }, // Will overflow valid range
        ];

        let scale = 1.0; // Large scale to trigger overflow
        let origin_lon = 0.0;
        let origin_lat = 0.0;

        let result = PointOp::resolve(&ops, scale, origin_lon, origin_lat);
        assert_compact_debug_snapshot!(result.unwrap_err(), @"CoordinateOutOfRange { point: Point { lat: 20000.0, lon: 10000.0 } }");
    }

    // Tests for from_points()

    #[test]
    fn from_points_empty() {
        let points = vec![];
        let scale = 0.0001;
        let origin_lon = 0.0;
        let origin_lat = 0.0;

        let ops = PointOp::from_points(&points, scale, origin_lon, origin_lat).unwrap();
        assert_compact_debug_snapshot!(ops, @"[]");
    }

    #[test]
    fn from_points_simple() {
        let points = vec![
            Point::lat_lon(0.22, 0.11), // lat, lon in radians
            Point::lat_lon(0.24, 0.13),
        ];
        let scale = 0.0001;
        let origin_lon = 0.1;
        let origin_lat = 0.2;

        let ops = PointOp::from_points(&points, scale, origin_lon, origin_lat).unwrap();

        // First point: lon offset = (0.11 - 0.1) / 0.0001 = 100
        //              lat offset = (0.22 - 0.2) / 0.0001 = 200
        // Second point: lon offset = (0.13 - 0.1) / 0.0001 = 300
        //               lat offset = (0.24 - 0.2) / 0.0001 = 400
        assert_compact_debug_snapshot!(ops, @"[NewPoint { x: 100, y: 200 }, NewPoint { x: 300, y: 400 }]");
    }

    #[test]
    fn from_points_requires_move_origin() {
        let points = vec![
            Point::lat_lon(0.2, 0.1),
            // Large jump that exceeds i16::MAX when scaled
            Point::lat_lon(0.2 + 40000.0 * 0.0001, 0.1 + 50000.0 * 0.0001),
        ];
        let scale = 0.0001;
        let origin_lon = 0.1;
        let origin_lat = 0.2;

        let ops = PointOp::from_points(&points, scale, origin_lon, origin_lat).unwrap();

        // First point at origin (0,0), then move origin by i16::MAX, then emit remaining offset
        assert_debug_snapshot!(ops, @r"
        [
            NewPoint { x: 0, y: 0 },
            MoveOrigin { x: 32767, y: 32767 },
            NewPoint { x: 17233, y: 7233 },
        ]
        ");
    }

    #[test]
    fn from_points_multiple_move_origins() {
        // Create points that require multiple origin moves per point
        // With scale=0.00001, i16::MAX covers only 0.32767 radians
        // So a jump of 3.0 radians requires ~9 MoveOrigin operations
        let points = vec![
            Point::lat_lon(0.0, 0.0),
            Point::lat_lon(0.0, 3.0), // 3.0 radians = 300,000 units at scale=0.00001
        ];
        let scale = 0.00001;
        let origin_lon = 0.0;
        let origin_lat = 0.0;

        let ops = PointOp::from_points(&points, scale, origin_lon, origin_lat).unwrap();

        // Jump of 300,000 units requires 9 MoveOrigin ops (9 × 32767) plus final NewPoint with remainder
        assert_debug_snapshot!(ops, @r"
        [
            NewPoint { x: 0, y: 0 },
            MoveOrigin { x: 32767, y: 0 },
            MoveOrigin { x: 32767, y: 0 },
            MoveOrigin { x: 32767, y: 0 },
            MoveOrigin { x: 32767, y: 0 },
            MoveOrigin { x: 32767, y: 0 },
            MoveOrigin { x: 32767, y: 0 },
            MoveOrigin { x: 32767, y: 0 },
            MoveOrigin { x: 32767, y: 0 },
            MoveOrigin { x: 32767, y: 0 },
            NewPoint { x: 5097, y: 0 },
        ]
        ");
    }

    #[test]
    fn from_points_round_trip() {
        // Original points
        let original = vec![
            Point::lat_lon(0.8, 0.4),
            Point::lat_lon(0.85, 0.45),
            Point::lat_lon(0.9, 0.5),
        ];

        let scale = 0.0001;
        let origin_lon = 0.3;
        let origin_lat = 0.7;

        // Convert to point ops
        let ops = PointOp::from_points(&original, scale, origin_lon, origin_lat).unwrap();

        // Convert back to points
        let reconstructed = PointOp::resolve(&ops, scale, origin_lon, origin_lat).unwrap();

        // Compare (allow small floating point error)
        assert_eq!(reconstructed.len(), original.len());
        for (orig, recon) in original.iter().zip(reconstructed.iter()) {
            assert_lt!((orig.lat - recon.lat).abs(), 1e-5,);
            assert_lt!((orig.lon - recon.lon).abs(), 1e-5,);
        }
    }

    #[test]
    fn from_points_round_trip_with_move_origin() {
        // Points that will require MoveOrigin (but stay in valid range)
        // With scale=0.0001, i16::MAX covers 3.2767 radians
        // So we need jumps larger than that. Use scale=0.00005 instead.
        // Max valid lat: ±π/2 = ±1.5708, max valid lon: ±π = ±3.14159
        let original = vec![
            Point::lat_lon(0.1, 0.2),
            Point::lat_lon(1.5, 3.0), // Jump of 2.8 lon requires MoveOrigin with smaller scale
        ];

        let scale = 0.00005; // i16::MAX * 0.00005 = 1.638 radians coverage
        let origin_lon = 0.0;
        let origin_lat = 0.0;

        let ops = PointOp::from_points(&original, scale, origin_lon, origin_lat).unwrap();

        let num_moves = ops
            .iter()
            .filter(|op| matches!(op, PointOp::MoveOrigin { .. }))
            .count();
        assert_eq!(num_moves, 1);

        let reconstructed = PointOp::resolve(&ops, scale, origin_lon, origin_lat).unwrap();

        assert_eq!(reconstructed.len(), original.len());
        for (orig, recon) in original.iter().zip(reconstructed.iter()) {
            assert_lt!((orig.lat - recon.lat).abs(), 1e-4);
            assert_lt!((orig.lon - recon.lon).abs(), 1e-4);
        }
    }

    #[test]
    fn from_points_round_trip_various_scales() {
        let original = vec![Point::lat_lon(0.5, 0.3), Point::lat_lon(0.52, 0.32)];

        let origin_lon = 0.2;
        let origin_lat = 0.4;

        for scale in [0.00001, 0.0001, 0.001, 0.01] {
            let ops = PointOp::from_points(&original, scale, origin_lon, origin_lat).unwrap();
            let reconstructed = PointOp::resolve(&ops, scale, origin_lon, origin_lat).unwrap();

            assert_eq!(reconstructed.len(), original.len());
            for (orig, recon) in original.iter().zip(reconstructed.iter()) {
                assert_lt!((orig.lat - recon.lat).abs(), scale * 0.6);
                assert_lt!((orig.lon - recon.lon).abs(), scale * 0.6);
            }
        }
    }
}
