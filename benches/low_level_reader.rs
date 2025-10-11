use criterion::{Criterion, criterion_group, criterion_main};
use seeyou_cub::raw::{HEADER_SIZE, Header, Item, ItemData};
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

fn low_level_reader_benchmark(c: &mut Criterion) {
    c.bench_function("low_level_reader", |b| {
        b.iter(|| {
            let file = File::open("tests/fixtures/france_2024.07.02.cub").unwrap();
            let mut reader = BufReader::new(file);

            let header = Header::read(&mut reader).unwrap();

            let mut items = Vec::new();
            for i in 0..header.hdr_items {
                let offset = HEADER_SIZE as u64 + (i as u64 * header.size_of_item as u64);
                reader.seek(SeekFrom::Start(offset)).unwrap();

                let item = Item::read(&mut reader, &header).unwrap();

                let data_offset = header.data_offset as u64 + item.points_offset as u64;
                reader.seek(SeekFrom::Start(data_offset)).unwrap();

                let item_data = ItemData::read(&mut reader, &header).unwrap();

                items.push((item, item_data));
            }

            items
        });
    });
}

criterion_group!(benches, low_level_reader_benchmark);
criterion_main!(benches);
