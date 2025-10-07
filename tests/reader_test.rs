use seeyou_cub::{CubStyle, parse};
use std::fs::File;

#[test]
fn parse_france_fixture() {
    let file =
        File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture file");

    let (mut cub, warnings) = parse(file).expect("Failed to parse CUB file");

    // Validate warnings
    assert_eq!(warnings.len(), 0);

    // Snapshot the header
    insta::assert_debug_snapshot!("header", cub.header());

    // Validate item count
    assert_eq!(cub.items().len(), 1368);

    // Snapshot first few items
    insta::assert_debug_snapshot!("items_sample", &cub.items()[0..5]);

    // Validate first item's points
    let first_item = cub.items().first().unwrap().clone();
    let points: Vec<_> = cub
        .read_points(&first_item)
        .expect("Failed to read points")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse points");

    insta::assert_debug_snapshot!("first_item_points", points);

    let last_item = cub.items().last().unwrap().clone();
    let points: Vec<_> = cub
        .read_points(&last_item)
        .expect("Failed to read points")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse points");

    insta::assert_debug_snapshot!("last_item_points", points);

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
