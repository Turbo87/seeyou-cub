use insta::assert_debug_snapshot;
use seeyou_cub::{Header, Item, RawItemData};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Seek, SeekFrom};

#[test]
fn raw_api_read_all_items() {
    let mut file =
        File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture");

    let header = Header::read(&mut file).unwrap();

    // Read all items to verify we can parse the entire file
    let mut count = 0;
    for i in 0..header.hdr_items {
        let offset = header.header_offset as u64 + (i as u64 * header.size_of_item as u64);
        file.seek(SeekFrom::Start(offset)).unwrap();

        let item =
            Item::read(&mut file, &header).unwrap_or_else(|_| panic!("Failed to read item {}", i));

        let offset = header.data_offset as u64 + item.points_offset as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();

        let _item_data = RawItemData::read(&mut file, &header)
            .unwrap_or_else(|_| panic!("Failed to read item data for item {}", i));

        count += 1;
    }

    assert_eq!(count, 1368);
}

#[test]
fn parse_france_fixture_raw_api() {
    let mut file =
        File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open fixture");

    let header = Header::read(&mut file).expect("Failed to read header");

    assert_debug_snapshot!("raw_header", header);

    // Read all items
    let mut items = Vec::new();
    for i in 0..header.hdr_items {
        let offset = header.header_offset as u64 + (i as u64 * header.size_of_item as u64);
        file.seek(SeekFrom::Start(offset)).unwrap();

        let item =
            Item::read(&mut file, &header).unwrap_or_else(|_| panic!("Failed to read item {}", i));

        items.push(item);
    }

    assert_eq!(items.len(), 1368);
    assert_debug_snapshot!("raw_items_first_5", &items[0..5]);
    assert_debug_snapshot!("raw_items_last_5", &items[items.len() - 5..]);

    // Airspace type distribution
    let mut class_counts = HashMap::new();
    for item in &items {
        *class_counts.entry(item.class()).or_insert(0) += 1;
    }
    let mut class_counts = class_counts.iter().collect::<Vec<_>>();
    class_counts.sort_by(|a, b| a.1.cmp(b.1).reverse());
    assert_debug_snapshot!("raw_class_counts", class_counts);

    let mut style_counts = HashMap::new();
    for item in &items {
        *style_counts.entry(item.style()).or_insert(0) += 1;
    }
    let mut style_counts = style_counts.iter().collect::<Vec<_>>();
    style_counts.sort_by(|a, b| a.1.cmp(b.1).reverse());
    assert_debug_snapshot!("raw_style_counts", style_counts);

    // First item complete data
    let first_item = items.first().unwrap();
    let offset = header.data_offset as u64 + first_item.points_offset as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    let first_item_data =
        RawItemData::read(&mut file, &header).expect("Failed to read first item data");
    assert_debug_snapshot!("raw_first_item_data", first_item_data);

    // Last item complete data
    let last_item = items.last().unwrap();
    let offset = header.data_offset as u64 + last_item.points_offset as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    let last_item_data =
        RawItemData::read(&mut file, &header).expect("Failed to read last item data");
    assert_debug_snapshot!("raw_last_item_data", last_item_data);

    // Collect all names and auto-select representatives
    let mut all_names = Vec::new();
    let mut min_points_item: (usize, usize) = (0, usize::MAX); // (index, point_ops_count)
    let mut max_points_item: (usize, usize) = (0, 0);

    for (i, item) in items.iter().enumerate() {
        let offset = header.data_offset as u64 + item.points_offset as u64;
        file.seek(SeekFrom::Start(offset)).unwrap();

        let item_data = RawItemData::read(&mut file, &header)
            .unwrap_or_else(|_| panic!("Failed to read item data for item {}", i));

        // Decode name from raw bytes
        let name = item_data
            .name
            .as_ref()
            .map(|bytes| String::from_utf8_lossy(bytes).to_string())
            .unwrap_or_default();
        all_names.push(name);

        // Track min/max by point_ops count
        let point_ops_count = item_data.point_ops.len();
        if min_points_item.1 > point_ops_count {
            min_points_item = (i, point_ops_count);
        }
        if max_points_item.1 < point_ops_count {
            max_points_item = (i, point_ops_count);
        }
    }

    all_names.sort();
    assert_debug_snapshot!("raw_all_airspace_names", all_names);

    // Snapshot smallest airspace by point_ops count
    let item = &items[min_points_item.0];
    let offset = header.data_offset as u64 + item.points_offset as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    let item_data =
        RawItemData::read(&mut file, &header).expect("Failed to read smallest item data");
    assert_debug_snapshot!("raw_representative_smallest_by_points", item_data);

    // Snapshot largest airspace by point_ops count
    let item = &items[max_points_item.0];
    let offset = header.data_offset as u64 + item.points_offset as u64;
    file.seek(SeekFrom::Start(offset)).unwrap();
    let item_data =
        RawItemData::read(&mut file, &header).expect("Failed to read largest item data");
    assert_debug_snapshot!("raw_representative_largest_by_points", item_data);
}
