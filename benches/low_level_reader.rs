use criterion::{Criterion, criterion_group, criterion_main};
use seeyou_cub::raw::{Header, Item, ItemData};
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

fn low_level_reader_benchmark(c: &mut Criterion) {
    c.bench_function("low_level_reader", |b| {
        b.iter(|| {
            let file = File::open("tests/fixtures/france_2024.07.02.cub").unwrap();
            let mut reader = BufReader::new(file);

            let header = Header::read(&mut reader).unwrap();

            let mut items = Vec::new();
            let mut item_data = Vec::new();
            for _ in 0..header.hdr_items {
                items.push(Item::read(&mut reader, &header).unwrap());
            }

            for item in &items {
                let data_offset = header.data_offset as u64 + item.points_offset as u64;
                reader.seek(SeekFrom::Start(data_offset)).unwrap();

                item_data.push(ItemData::read(&mut reader, &header).unwrap());
            }

            items
        });
    });
}

criterion_group!(benches, low_level_reader_benchmark);
criterion_main!(benches);
