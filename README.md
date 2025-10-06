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
use seeyou_cub::parse;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("airspace.cub")?;
    let (mut cub, warnings) = parse(file)?;

    println!("Loaded {} airspaces", cub.items().len());

    // Inspect warnings
    for warning in warnings {
        eprintln!("Warning: {:?}", warning);
    }

    // Access airspace metadata
    for item in cub.items() {
        println!("{:?} {:?}: {}-{} meters",
            item.style(),
            item.class(),
            item.min_alt,
            item.max_alt,
        );
    }

    // Parse geometry for first airspace
    if let Some(first) = cub.items().first() {
        for point in cub.read_points(first)? {
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