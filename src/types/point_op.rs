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

#[cfg(test)]
mod tests {
    use super::*;

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
}
