# seeyou-cub

A Rust parser for the [SeeYou CUB binary file format](docs/CUB_file_format.md), 
which stores airspace data for flight navigation software.

## Features

- Parse CUB files from any `Read + Seek` source
- Lenient parsing with warning collection
- Lazy geometry parsing for memory efficiency
- Support for both little-endian and big-endian files
- Optional `jiff` integration for date/time handling

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
seeyou-cub = "0.1.0"
```

Basic example:

```rust
use seeyou_cub::CubReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = CubReader::from_path("airspace.cub")?;
    let mut warnings = Vec::new();

    // Parse header
    let header = reader.read_header(&mut warnings)?;

    // Parse all items
    let items: Vec<_> = reader
        .read_items(&header, &mut warnings)
        .collect::<Result<Vec<_>, _>>()?;

    println!("Loaded {} airspaces", items.len());

    // Inspect warnings
    for warning in &warnings {
        eprintln!("Warning: {:?}", warning);
    }

    // Access airspace metadata
    for item in &items {
        println!("{:?} {:?}: {}-{} meters",
            item.style(),
            item.class(),
            item.min_alt,
            item.max_alt,
        );
    }

    // Parse complete data (geometry + metadata) for first airspace
    if let Some(first) = items.first() {
        let item_data = reader.read_item_data(&header, first, &mut warnings)?;

        if let Some(name) = &item_data.name {
            println!("  Name: {}", name);
        }

        for point in &item_data.points {
            println!("  Point: {} {}", point.lon, point.lat);
        }
    }

    Ok(())
}
```

## Optional Features

### `datetime`

Enable `jiff` integration for convenient date/time handling:

```toml
[dependencies]
seeyou-cub = { version = "0.0.0", features = ["datetime"] }
```

With this feature enabled, `Item::start_date()` and `Item::end_date()` return 
`jiff::civil::DateTime`.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dually licensed as above, without any additional terms or conditions.
