/// Bounding box for geographic areas
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub left: f32,   // west longitude (radians)
    pub top: f32,    // north latitude (radians)
    pub right: f32,  // east longitude (radians)
    pub bottom: f32, // south latitude (radians)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_construction() {
        let bbox = BoundingBox {
            left: -0.1,
            top: 0.9,
            right: 0.1,
            bottom: 0.8,
        };

        assert_eq!(bbox.left, -0.1);
        assert_eq!(bbox.top, 0.9);
        assert_eq!(bbox.right, 0.1);
        assert_eq!(bbox.bottom, 0.8);
    }
}
