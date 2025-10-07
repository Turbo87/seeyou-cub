use insta::assert_debug_snapshot;
use seeyou_cub::CubReader;
use std::collections::HashMap;

#[test]
fn parse_france_fixture() {
    let mut reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");
    let mut warnings = Vec::new();

    let header = reader
        .read_header(&mut warnings)
        .expect("Failed to parse header");

    assert_debug_snapshot!("header", header);

    let items: Vec<_> = reader
        .read_items(&header, &mut warnings)
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse items");

    assert_eq!(items.len(), 1368);
    assert_debug_snapshot!("items_first_5", &items[0..5]);
    assert_debug_snapshot!("items_last_5", &items[items.len() - 5..]);

    // Airspace type distribution
    let mut class_counts = HashMap::new();
    for item in &items {
        *class_counts.entry(item.class()).or_insert(0) += 1;
    }
    let mut class_counts = class_counts.iter().collect::<Vec<_>>();
    class_counts.sort_by(|a, b| a.1.cmp(b.1).reverse());
    assert_debug_snapshot!("class_counts", class_counts);

    let mut style_counts = HashMap::new();
    for item in &items {
        *style_counts.entry(item.style()).or_insert(0) += 1;
    }
    let mut style_counts = style_counts.iter().collect::<Vec<_>>();
    style_counts.sort_by(|a, b| a.1.cmp(b.1).reverse());
    assert_debug_snapshot!("style_counts", style_counts);

    // First item complete data
    let first_item = items.first().unwrap();
    let first_item_data = reader
        .read_item_data(&header, first_item, &mut warnings)
        .expect("Failed to parse first item data");
    assert_debug_snapshot!("first_item_data", first_item_data);

    // Last item complete data
    let last_item = items.last().unwrap();
    let last_item_data = reader
        .read_item_data(&header, last_item, &mut warnings)
        .expect("Failed to parse last item data");
    assert_debug_snapshot!("last_item_data", last_item_data);

    // Collect all names and auto-select representatives
    let mut all_names = Vec::new();
    let mut min_points_item: (usize, usize) = (0, usize::MAX); // (index, point_count)
    let mut max_points_item: (usize, usize) = (0, 0);

    for (i, item) in items.iter().enumerate() {
        let item_data = reader
            .read_item_data(&header, item, &mut warnings)
            .unwrap_or_else(|_| panic!("Failed to parse item data for item {}", i));

        let name = item_data.name.unwrap_or_default();
        all_names.push(name.clone());

        // Track min/max by point count
        let point_count = item_data.points.len();
        if min_points_item.1 > point_count {
            min_points_item = (i, point_count);
        }
        if max_points_item.1 < point_count {
            max_points_item = (i, point_count);
        }
    }

    all_names.sort();
    assert_debug_snapshot!("all_airspace_names", all_names);

    // Snapshot smallest airspace by point count
    let item = &items[min_points_item.0];
    let item_data = reader
        .read_item_data(&header, item, &mut warnings)
        .expect("Failed to parse smallest item data");
    assert_debug_snapshot!("representative_smallest_by_points", item_data);

    // Snapshot largest airspace by point count
    let item = &items[max_points_item.0];
    let item_data = reader
        .read_item_data(&header, item, &mut warnings)
        .expect("Failed to parse largest item data");
    assert_debug_snapshot!("representative_largest_by_points", item_data);

    // Ensure that there are no warnings
    assert_eq!(warnings.len(), 0);
}
