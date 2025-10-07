use seeyou_cub::parse;
use std::fs::File;

#[test]
fn parse_france_fixture() {
    let file =
        File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture file");

    let (mut cub, warnings) = parse(file).expect("Failed to parse CUB file");

    // Validate warnings
    assert_eq!(warnings.len(), 0);

    // Validate header
    assert_eq!(cub.items().len(), 1368);

    let (west, south, east, north) = cub.header().bounding_box();
    assert!((west - (-0.085230246)).abs() < 0.0001);
    assert!((south - 0.71856177).abs() < 0.0001);
    assert!((east - 0.17016976).abs() < 0.0001);
    assert!((north - 0.89215416).abs() < 0.0001);

    // Validate first few items
    use seeyou_cub::{CubClass, CubStyle};
    assert_eq!(cub.items()[0].style(), CubStyle::RestrictedArea);
    assert_eq!(cub.items()[0].class(), CubClass::Unknown);
    assert_eq!(cub.items()[0].min_alt, 0);
    assert_eq!(cub.items()[0].max_alt, 488);

    assert_eq!(cub.items()[1].style(), CubStyle::ProhibitedArea);
    assert_eq!(cub.items()[1].class(), CubClass::Unknown);
    assert_eq!(cub.items()[1].min_alt, 0);
    assert_eq!(cub.items()[1].max_alt, 610);

    // Validate first item's points
    let first_item = cub.items()[0].clone();
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

    assert_eq!(point_count, 7);

    let first_point = first_point.expect("Should have at least one point");
    assert!((first_point.lon - 0.033180647).abs() < 0.0001);
    assert!((first_point.lat - 0.83465517).abs() < 0.0001);

    // Validate all airspaces and collect statistics
    let mut total_points = 0;
    let mut style_counts = std::collections::HashMap::new();

    let items = cub.items().to_vec();
    for item in &items {
        *style_counts.entry(item.style()).or_insert(0) += 1;

        let mut point_iter = cub.read_points(item).expect("Failed to read points");

        for point_result in &mut point_iter {
            let _point = point_result.expect("Failed to parse point");
            total_points += 1;
        }
    }

    // Validate total points
    assert_eq!(total_points, 93784, "Total points mismatch");

    // Validate airspace type distribution
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
