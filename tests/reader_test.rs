use seeyou_cub::parse;
use std::fs::File;

#[test]
fn parse_france_fixture() {
    let file =
        File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture file");

    let (cub, warnings) = parse(file).expect("Failed to parse CUB file");

    // Assert total number of airspaces
    assert_eq!(cub.items().len(), 1368);

    // Check bounding box for France
    let (west, south, east, north) = cub.header().bounding_box();
    assert!((west - (-0.085230246)).abs() < 0.0001);
    assert!((south - 0.71856177).abs() < 0.0001);
    assert!((east - 0.17016976).abs() < 0.0001);
    assert!((north - 0.89215416).abs() < 0.0001);

    // Assert expected warnings
    assert_eq!(warnings.len(), 1);

    // Check first few items have expected properties
    use seeyou_cub::{CubClass, CubStyle};
    assert_eq!(cub.items()[0].style(), CubStyle::RestrictedArea);
    assert_eq!(cub.items()[0].class(), CubClass::Unknown);
    assert_eq!(cub.items()[0].min_alt, 0);
    assert_eq!(cub.items()[0].max_alt, 488);

    assert_eq!(cub.items()[1].style(), CubStyle::ProhibitedArea);
    assert_eq!(cub.items()[1].class(), CubClass::Unknown);
    assert_eq!(cub.items()[1].min_alt, 0);
    assert_eq!(cub.items()[1].max_alt, 610);
}

#[test]
fn parse_and_read_points() {
    let file =
        File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture file");

    let (mut cub, _warnings) = parse(file).expect("Failed to parse CUB file");

    // Parse points for first item
    let first_item = cub
        .items()
        .first()
        .cloned()
        .expect("Should have at least one item");
    let mut points = cub.read_points(&first_item).expect("Failed to read points");

    let mut point_count = 0;
    let mut first_point = None;

    for point_result in &mut points {
        let point = point_result.expect("Failed to parse point");
        if first_point.is_none() {
            first_point = Some(point.clone());
        }
        point_count += 1;
    }

    // Assert expected point count for first item
    assert_eq!(point_count, 7);

    // Assert first point coordinates
    let first_point = first_point.expect("Should have at least one point");
    assert!((first_point.lon - 0.033180647).abs() < 0.0001);
    assert!((first_point.lat - 0.83465517).abs() < 0.0001);
}

#[test]
fn iterate_all_airspaces() {
    let file =
        File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture file");

    let (mut cub, _warnings) = parse(file).expect("Failed to parse CUB file");

    let mut total_points = 0;

    let items = cub.items().to_vec();
    for item in items {
        let points = cub.read_points(&item).expect("Failed to read points");
        let count = points.count();
        total_points += count;
    }

    // Assert expected total point count across all airspaces
    assert_eq!(total_points, 93785);
}

#[test]
fn comprehensive_france_parse() {
    let file =
        File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture file");

    let (mut cub, warnings) = parse(file).expect("Failed to parse CUB file");

    // Collect statistics
    let mut total_points = 0;
    let mut style_counts = std::collections::HashMap::new();

    let items = cub.items().to_vec();
    for item in &items {
        *style_counts.entry(item.style()).or_insert(0) += 1;

        let mut point_iter = cub.read_points(item).expect("Failed to read points");

        let mut count = 0;

        for point_result in &mut point_iter {
            match point_result {
                Ok(_point) => {
                    count += 1;
                }
                Err(_) => {
                    // Skip items with point parsing errors and continue
                    break;
                }
            }
        }

        total_points += count;
    }

    // Validate core statistics
    assert_eq!(cub.items().len(), 1368, "Total items mismatch");
    assert!(
        total_points > 93000,
        "Should have substantial geometry (got {})",
        total_points
    );
    assert_eq!(warnings.len(), 1, "Warning count mismatch");

    // Validate airspace distribution
    use seeyou_cub::CubStyle;

    // Exact validation of airspace types (based on observed France fixture)
    assert_eq!(
        *style_counts.get(&CubStyle::Unknown).unwrap_or(&0),
        452,
        "Unknown airspace count mismatch"
    );
    assert_eq!(
        *style_counts.get(&CubStyle::RestrictedArea).unwrap_or(&0),
        435,
        "Restricted area count mismatch"
    );
    assert_eq!(
        *style_counts.get(&CubStyle::ProhibitedArea).unwrap_or(&0),
        113,
        "Prohibited area count mismatch"
    );
    assert_eq!(
        *style_counts.get(&CubStyle::DangerArea).unwrap_or(&0),
        111,
        "Danger area count mismatch"
    );
    assert_eq!(
        *style_counts.get(&CubStyle::GliderSector).unwrap_or(&0),
        135,
        "Glider sector count mismatch"
    );
    assert_eq!(
        *style_counts.get(&CubStyle::ControlZone).unwrap_or(&0),
        92,
        "Control zone count mismatch"
    );
}
