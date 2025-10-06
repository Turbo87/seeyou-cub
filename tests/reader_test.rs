use seeyou_cub::parse;
use std::fs::File;

#[test]
fn parse_france_fixture() {
    let file = File::open("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");

    let (cub, warnings) = parse(file).expect("Failed to parse CUB file");

    // Basic assertions
    assert!(cub.items().len() > 0, "Should have at least one airspace");

    // Check bounding box makes sense for France
    let (west, south, east, north) = cub.header().bounding_box();
    println!("Bounding box: W={} S={} E={} N={}", west, south, east, north);

    // Print some stats
    println!("Total airspaces: {}", cub.items().len());
    println!("Warnings: {}", warnings.len());
    for warning in &warnings {
        println!("  Warning: {:?}", warning);
    }

    // Check first few items
    for (i, item) in cub.items().iter().take(5).enumerate() {
        println!("Item {}: style={:?} class={:?} alt={}-{}",
            i, item.style(), item.class(), item.min_alt, item.max_alt);
    }
}

#[test]
fn parse_and_read_points() {
    let file = File::open("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");

    let (mut cub, _warnings) = parse(file).expect("Failed to parse CUB file");

    // Parse points for first item
    if let Some(first_item) = cub.items().first().cloned() {
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

        println!("First item has {} points", point_count);

        if let Some(point) = first_point {
            println!("First point: lon={} lat={}", point.lon, point.lat);
            if let Some(name) = point.name {
                println!("  Name: {}", name);
            }
            if let Some(freq) = point.frequency {
                println!("  Frequency: {} Hz", freq);
            }
        }

        // Check warnings from point parsing
        for warning in points.warnings() {
            println!("Point warning: {:?}", warning);
        }

        assert!(point_count > 0, "Should have at least one point");
    }
}

#[test]
fn iterate_all_airspaces() {
    let file = File::open("tests/fixtures/france_2024.07.02.cub")
        .expect("Failed to open fixture file");

    let (mut cub, _warnings) = parse(file).expect("Failed to parse CUB file");

    let mut total_points = 0;

    let items = cub.items().to_vec();
    for item in items {
        let points = cub.read_points(&item).expect("Failed to read points");
        let count = points.count();
        total_points += count;
    }

    println!("Total points across all airspaces: {}", total_points);
    assert!(total_points > 0);
}
