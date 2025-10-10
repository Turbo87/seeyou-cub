/// A geographic coordinate point
///
/// Represents a single point in an airspace boundary with lat/lon coordinates in radians.
/// This is the high-level representation after converting raw i16 x/y offsets.
#[derive(Clone, PartialEq)]
pub struct Point {
    /// Latitude in radians (positive = North)
    pub lat: f32,
    /// Longitude in radians (positive = East)
    pub lon: f32,
}

impl Point {
    /// Create a new point with given lat/lon in radians
    pub fn new(lat: f32, lon: f32) -> Self {
        Self { lat, lon }
    }

    /// Check if coordinates are within valid ranges
    pub fn is_valid(&self) -> bool {
        self.lat >= -std::f32::consts::FRAC_PI_2
            && self.lat <= std::f32::consts::FRAC_PI_2
            && self.lon >= -std::f32::consts::PI
            && self.lon <= std::f32::consts::PI
    }
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point {{ lat: {:?}, lon: {:?} }}", self.lat, self.lon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_creation() {
        // Paris: 48.8566°N, 2.3522°E in radians
        let point = Point::new(0.8529413854, 0.0410370585);
        assert_eq!(point.lat, 0.8529413854);
        assert_eq!(point.lon, 0.0410370585);
    }

    #[test]
    fn point_is_valid() {
        // Paris: 48.8566°N, 2.3522°E in radians
        assert!(Point::new(0.8529413854, 0.0410370585).is_valid());
        assert!(Point::new(0.0, 0.0).is_valid()); // Null Island
        assert!(Point::new(std::f32::consts::FRAC_PI_2, std::f32::consts::PI).is_valid()); // Edge cases
        assert!(Point::new(-std::f32::consts::FRAC_PI_2, -std::f32::consts::PI).is_valid()); // Edge cases

        assert!(!Point::new(std::f32::consts::FRAC_PI_2 + 0.1, 0.0).is_valid()); // Invalid lat
        assert!(!Point::new(-std::f32::consts::FRAC_PI_2 - 0.1, 0.0).is_valid()); // Invalid lat
        assert!(!Point::new(0.0, std::f32::consts::PI + 0.1).is_valid()); // Invalid lon
        assert!(!Point::new(0.0, -std::f32::consts::PI - 0.1).is_valid()); // Invalid lon
    }
}
