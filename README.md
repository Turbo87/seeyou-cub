# seeyou-cub

A Rust parser for the [SeeYou CUB binary file format](docs/CUB_file_format.md),
which stores airspace data for flight navigation software.

## Features

- **Two-tier API**: High-level iterator for convenience, low-level functions for control
- **Memory efficient**: Lazy parsing with no internal caching
- **UTF-8 with fallback**: Decodes strings as UTF-8 with Extended ASCII fallback
- **Coordinate conversion**: Automatic conversion from raw i16 offsets to lat/lon

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
seeyou-cub = "0.1.0"
```

### High-Level API

The high-level API provides an iterator that yields fully-decoded `Airspace` structs:

```rust,no_run
use seeyou_cub::CubReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = CubReader::from_path("airspace.cub")?;

    // Iterate over all airspaces
    for result in reader.read_airspaces() {
        let airspace = result?;

        // All fields are decoded and ready to use
        if let Some(name) = &airspace.name {
            println!("{}: {:?} {:?}", name, airspace.style, airspace.class);
            println!("  Altitude: {} - {} meters", airspace.min_alt, airspace.max_alt);
            println!("  Points: {}", airspace.points.len());
        }
    }

    Ok(())
}
```

### Low-Level API

The low-level API provides direct access to raw file data with minimal transformation:

```rust,no_run
use seeyou_cub::raw::{Header, Item, ItemData};
use std::fs::File;
use std::io::{Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("airspace.cub")?;

    // Read header
    let header = Header::read(&mut file)?;
    println!("CUB file: {:?}", header.title);

    // Read items (airspace metadata)
    for i in 0..header.hdr_items {
        let offset = header.header_offset as u64 + (i as u64 * header.size_of_item as u64);
        file.seek(SeekFrom::Start(offset))?;

        let item = Item::read(&mut file, &header)?;

        // Read raw item data (point operations + raw bytes)
        let data_offset = header.data_offset as u64 + item.points_offset as u64;
        file.seek(SeekFrom::Start(data_offset))?;

        let item_data = ItemData::read(&mut file, &header)?;

        // Access raw point operations (i16 offsets, not yet converted to lat/lon)
        println!("Point operations: {}", item_data.point_ops.len());

        // Strings are ByteString (raw bytes with human-readable debug output)
        if let Some(name_bytes) = &item_data.name {
            let name = String::from_utf8_lossy(name_bytes.as_bytes());
            println!("Name: {}", name);
        }
    }

    Ok(())
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dually licensed as above, without any additional terms or conditions.
