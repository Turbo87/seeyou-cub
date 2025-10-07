# seeyou-cub

A Rust parser for the SeeYou CUB binary file format, which stores airspace data for flight navigation software.

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
seeyou-cub = "0.0.0"
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

    // Parse geometry for first airspace
    if let Some(first) = items.first() {
        for point in reader.read_points(&header, first, &mut warnings) {
            let pt = point?;
            println!("  Point: {} {}", pt.lon, pt.lat);
            if let Some(name) = &pt.name {
                println!("    Name: {}", name);
            }
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

With this feature enabled, `Item::start_date()` and `Item::end_date()` return `jiff::civil::DateTime`.

## File Format

The CUB format specification is available in the `docs/CUB_file_format.md` file.

## License

TBD