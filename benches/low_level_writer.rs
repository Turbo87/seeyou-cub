use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use seeyou_cub::raw::{HEADER_SIZE, Header, Item, ItemData};
use std::fs::File;
use std::io::{Cursor, Seek, SeekFrom};

fn low_level_writer_benchmark(c: &mut Criterion) {
    c.bench_function("low_level_writer", |b| {
        b.iter_batched(
            || {
                // Setup: Read France fixture into raw structures (not timed)
                let mut file =
                    File::open("tests/fixtures/france_2024.07.02.cub").expect("Failed to open");
                let header = Header::read(&mut file).expect("Failed to read header");

                let mut items = Vec::new();
                let mut item_data_list = Vec::new();

                for i in 0..header.hdr_items {
                    let offset = HEADER_SIZE as u64 + (i as u64 * header.size_of_item as u64);
                    file.seek(SeekFrom::Start(offset)).unwrap();

                    let item = Item::read(&mut file, &header).unwrap();

                    let data_offset = header.data_offset as u64 + item.points_offset as u64;
                    file.seek(SeekFrom::Start(data_offset)).unwrap();

                    let item_data = ItemData::read(&mut file, &header).unwrap();

                    items.push(item);
                    item_data_list.push(item_data);
                }

                (header, items, item_data_list)
            },
            |(header, items, item_data_list)| {
                // Timed: Write to in-memory buffer
                let mut cursor = Cursor::new(Vec::new());

                header.write(&mut cursor).unwrap();

                for item in &items {
                    item.write(&mut cursor, &header).unwrap();
                }

                for item_data in &item_data_list {
                    item_data.write(&mut cursor, &header).unwrap();
                }

                cursor
            },
            BatchSize::LargeInput,
        );
    });
}

criterion_group!(benches, low_level_writer_benchmark);
criterion_main!(benches);
