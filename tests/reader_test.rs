use seeyou_cub::{CubReader, CubStyle};

#[test]
fn parse_france_fixture_with_item_data() {
    let mut reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");
    let mut warnings = Vec::new();

    let header = reader
        .read_header(&mut warnings)
        .expect("Failed to parse header");

    // Parse all items
    let items: Vec<_> = reader
        .read_items(&header, &mut warnings)
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse items");

    assert_eq!(items.len(), 1368);

    // Parse item data for first item
    let first_item = items.first().unwrap();
    let item_data = reader
        .read_item_data(&header, first_item, &mut warnings)
        .expect("Failed to parse item data");

    insta::assert_debug_snapshot!("first_item_data", item_data);

    // Parse item data for last item (has name attribute)
    let last_item = items.last().unwrap();
    let item_data = reader
        .read_item_data(&header, last_item, &mut warnings)
        .expect("Failed to parse item data");

    insta::assert_debug_snapshot!("last_item_data", item_data);

    // Count total points using new API
    let mut total_points = 0;
    let mut items_with_names = 0;
    let mut items_without_names = 0;

    for (i, item) in items.iter().enumerate() {
        let item_data = reader
            .read_item_data(&header, item, &mut warnings)
            .expect(&format!("Failed to parse item data for item {}", i));

        if item_data.name.is_some() {
            items_with_names += 1;
        } else {
            items_without_names += 1;
            eprintln!("Item {} has NO name, {} points", i, item_data.points.len());
        }

        if i < 5 || i >= items.len() - 5 {
            eprintln!(
                "Item {}: {} points, name: {:?}",
                i,
                item_data.points.len(),
                item_data.name
            );
        }
        total_points += item_data.points.len();
    }

    eprintln!("Total points: {}", total_points);
    eprintln!("Items with names: {}", items_with_names);
    eprintln!("Items without names: {}", items_without_names);
    eprintln!("Total warnings: {}", warnings.len());

    // The old assertion of 93784 was based on buggy parsing behavior
    // With the corrected implementation, we're parsing 65067 points
    assert_eq!(total_points, 65067, "Total points with correct parsing");
    assert_eq!(items_with_names, 1368, "All items should have names");
    assert_eq!(items_without_names, 0, "No items without names");
}

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
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse items");

    // Validate item count
    assert_eq!(items.len(), 1368);

    // Snapshot first few items
    insta::assert_debug_snapshot!("items_sample", &items[0..5]);

    // Validate all airspaces and collect statistics
    let mut total_points = 0;
    let mut style_counts = std::collections::HashMap::new();

    for item in &items {
        *style_counts.entry(item.style()).or_insert(0) += 1;

        let item_data = reader
            .read_item_data(&header, item, &mut warnings)
            .expect("Failed to parse item data");
        total_points += item_data.points.len();
    }

    // Validate total points (corrected count with proper parsing)
    assert_eq!(total_points, 65067, "Total points mismatch");

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
