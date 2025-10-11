use criterion::{Criterion, criterion_group, criterion_main};
use seeyou_cub::CubReader;

fn high_level_reader_benchmark(c: &mut Criterion) {
    c.bench_function("high_level_reader", |b| {
        b.iter(|| {
            let mut reader = CubReader::from_path("tests/fixtures/france_2024.07.02.cub").unwrap();
            let airspaces: Vec<_> = reader.read_airspaces().collect::<Result<_, _>>().unwrap();
            airspaces
        });
    });
}

criterion_group!(benches, high_level_reader_benchmark);
criterion_main!(benches);
