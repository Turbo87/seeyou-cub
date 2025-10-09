use insta::assert_debug_snapshot;
use seeyou_cub::CubReader;
use std::collections::HashMap;

#[test]
fn parse_france_fixture() {
    let mut reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");

    let header = reader.header();
    assert_debug_snapshot!("header", header);

    let results: Vec<_> = reader
        .read_airspaces()
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse airspaces");

    // Extract airspaces and warnings
    let airspaces: Vec<_> = results.iter().map(|(a, _w)| a).collect();
    let all_warnings: Vec<_> = results.iter().flat_map(|(_a, w)| w).collect();

    assert_eq!(airspaces.len(), 1368);
    assert_eq!(all_warnings.len(), 0);

    // Snapshot first and last 5 airspaces
    assert_debug_snapshot!("airspaces_first_5", &airspaces[0..5]);
    assert_debug_snapshot!("airspaces_last_5", &airspaces[airspaces.len() - 5..]);

    // Airspace type distribution
    let mut class_counts = HashMap::new();
    for airspace in &airspaces {
        *class_counts.entry(airspace.class).or_insert(0) += 1;
    }
    let mut class_counts = class_counts.iter().collect::<Vec<_>>();
    class_counts.sort_by(|a, b| a.1.cmp(b.1).reverse());
    assert_debug_snapshot!("class_counts", class_counts);

    let mut style_counts = HashMap::new();
    for airspace in &airspaces {
        *style_counts.entry(airspace.style).or_insert(0) += 1;
    }
    let mut style_counts = style_counts.iter().collect::<Vec<_>>();
    style_counts.sort_by(|a, b| a.1.cmp(b.1).reverse());
    assert_debug_snapshot!("style_counts", style_counts);

    // First and last airspace complete data
    let first_airspace = airspaces.first().unwrap();
    assert_debug_snapshot!("first_airspace", first_airspace);

    let last_airspace = airspaces.last().unwrap();
    assert_debug_snapshot!("last_airspace", last_airspace);

    // Collect all names and auto-select representatives
    let mut all_names = Vec::new();
    let mut min_points_airspace: (usize, usize) = (0, usize::MAX); // (index, point_count)
    let mut max_points_airspace: (usize, usize) = (0, 0);

    for (i, airspace) in airspaces.iter().enumerate() {
        let name = airspace.name.as_deref().unwrap_or("");
        all_names.push(name.to_string());

        // Track min/max by point count
        let point_count = airspace.points.len();
        if min_points_airspace.1 > point_count {
            min_points_airspace = (i, point_count);
        }
        if max_points_airspace.1 < point_count {
            max_points_airspace = (i, point_count);
        }
    }

    all_names.sort();
    assert_debug_snapshot!("all_airspace_names", all_names);

    // Snapshot smallest airspace by point count
    let smallest_airspace = &airspaces[min_points_airspace.0];
    assert_debug_snapshot!("representative_smallest_by_points", smallest_airspace);

    // Snapshot largest airspace by point count
    let largest_airspace = &airspaces[max_points_airspace.0];
    assert_debug_snapshot!("representative_largest_by_points", largest_airspace);
}
