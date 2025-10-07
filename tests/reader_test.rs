use seeyou_cub::{CubReader, CubStyle};

#[test]
fn parse_france_fixture() {
    let mut reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");
    let mut warnings = Vec::new();

    let header = reader
        .read_header(&mut warnings)
        .expect("Failed to parse header");

    // Validate warnings
    assert_eq!(warnings.len(), 0);

    // Snapshot the header
    insta::assert_debug_snapshot!("header", header);

    // Parse all items
    let items: Vec<_> = reader
        .read_items(&header, &mut warnings)
        .expect("Failed to create item iterator")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse items");

    // Validate item count
    assert_eq!(items.len(), 1368);

    // Snapshot first few items
    insta::assert_debug_snapshot!("items_sample", &items[0..5]);

    // Validate first item's points
    let first_item = items.first().unwrap();
    let points: Vec<_> = reader
        .read_points(&header, first_item, &mut warnings)
        .expect("Failed to read points")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse points");

    insta::assert_debug_snapshot!("first_item_points", points);

    let last_item = items.last().unwrap();
    let points: Vec<_> = reader
        .read_points(&header, last_item, &mut warnings)
        .expect("Failed to read points")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse points");

    insta::assert_debug_snapshot!("last_item_points", points);

    // Validate all airspaces and collect statistics
    let mut total_points = 0;
    let mut style_counts = std::collections::HashMap::new();

    for item in &items {
        *style_counts.entry(item.style()).or_insert(0) += 1;

        let mut point_iter = reader
            .read_points(&header, item, &mut warnings)
            .expect("Failed to read points");

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
