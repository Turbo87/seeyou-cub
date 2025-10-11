use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use seeyou_cub::writer::CubWriter;
use seeyou_cub::{Airspace, CubReader};
use std::io::Cursor;

fn high_level_writer_benchmark(c: &mut Criterion) {
    c.bench_function("high_level_writer", |b| {
        b.iter_batched(
            || {
                // Setup: Read France fixture (not timed)
                CubReader::from_path("tests/fixtures/france_2024.07.02.cub")
                    .unwrap()
                    .read_airspaces()
                    .collect::<Result<Vec<Airspace>, _>>()
                    .unwrap()
            },
            |airspaces| {
                // Timed: Write to in-memory buffer
                let mut cursor = Cursor::new(Vec::new());
                CubWriter::new("Benchmark Test")
                    .add_airspaces(airspaces)
                    .write(&mut cursor)
                    .unwrap();
                cursor
            },
            BatchSize::LargeInput,
        );
    });
}

criterion_group!(benches, high_level_writer_benchmark);
criterion_main!(benches);
