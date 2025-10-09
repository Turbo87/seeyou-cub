use crate::error::Result;
use crate::new_api::types::{Point, PointOp};

/// Resolve point operations into geographic coordinates
///
/// Processes a sequence of point operations (origin moves and new points) and converts
/// them from raw i16 offsets to f64 lat/lon coordinates in radians.
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
pub fn resolve_point_ops(
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

                let point = Point::new(lat as f64, lon as f64);
                if !point.is_valid() {
                    return Err(crate::error::Error::CoordinateOutOfRange { point });
                }

                points.push(point);
            }
        }
    }

    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_compact_debug_snapshot;

    #[test]
    fn resolve_simple_points() {
        let ops = vec![
            PointOp::NewPoint { x: 100, y: 200 },
            PointOp::NewPoint { x: 150, y: 250 },
        ];

        let scale = 0.0001;
        let origin_lon = 0.1; // radians
        let origin_lat = 0.2; // radians

        let points = resolve_point_ops(&ops, scale, origin_lon, origin_lat).unwrap();
        assert_eq!(points.len(), 2);

        // Verify: origin + (offset * scale) in radians
        let expected_lon_1 = (0.1_f32 + 100.0 * 0.0001) as f64;
        let expected_lat_1 = (0.2_f32 + 200.0 * 0.0001) as f64;

        assert!((points[0].lon - expected_lon_1).abs() < 1e-9);
        assert!((points[0].lat - expected_lat_1).abs() < 1e-9);

        let expected_lon_2 = (0.1_f32 + 150.0 * 0.0001) as f64;
        let expected_lat_2 = (0.2_f32 + 250.0 * 0.0001) as f64;

        assert!((points[1].lon - expected_lon_2).abs() < 1e-9);
        assert!((points[1].lat - expected_lat_2).abs() < 1e-9);
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

        let points = resolve_point_ops(&ops, scale, origin_lon, origin_lat).unwrap();
        assert_eq!(points.len(), 2);

        // The first point should be relative to passed-in origin in radians
        let expected_lon_1 = (100.0_f32 * 0.0001) as f64;
        let expected_lat_1 = (200.0_f32 * 0.0001) as f64;

        assert!((points[0].lon - expected_lon_1).abs() < 1e-9);
        assert!((points[0].lat - expected_lat_1).abs() < 1e-9);

        // The second point should be relative to the moved origin in radians
        let expected_lon_2 = (1000.0_f32 * 0.0001 + 50.0 * 0.0001) as f64;
        let expected_lat_2 = (2000.0_f32 * 0.0001 + 100.0 * 0.0001) as f64;

        assert!((points[1].lon - expected_lon_2).abs() < 1e-9);
        assert!((points[1].lat - expected_lat_2).abs() < 1e-9);
    }

    #[test]
    fn resolve_out_of_range_fails() {
        let ops = vec![
            PointOp::NewPoint { x: 10000, y: 20000 }, // Will overflow valid range
        ];

        let scale = 1.0; // Large scale to trigger overflow
        let origin_lon = 0.0;
        let origin_lat = 0.0;

        let result = resolve_point_ops(&ops, scale, origin_lon, origin_lat);
        assert_compact_debug_snapshot!(result.unwrap_err(), @"CoordinateOutOfRange { point: Point { lat: 20000.0, lon: 10000.0 } }");
    }
}
