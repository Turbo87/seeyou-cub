# seeyou-cub

A Rust library for reading and writing the [SeeYou CUB binary file format](docs/CUB_file_format.md),
which stores airspace data for flight navigation software.

## Features

- **Read and write**: Full support for reading and writing CUB files
- **Two-tier API**: High-level reader/writer for convenience, low-level functions for control
- **Memory efficient**: Lazy parsing with no internal caching
- **UTF-8 with fallback**: Decodes strings as UTF-8 with Extended ASCII fallback
- **Coordinate conversion**: Automatic conversion between raw i16 offsets and lat/lon
- **Builder pattern**: Ergonomic writer API with automatic calculations

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
seeyou-cub = "0.3.0"
```

### High-Level API

#### Reading

The high-level API provides an iterator that yields fully-decoded `Airspace` structs:

```rust,no_run
use seeyou_cub::CubReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = CubReader::from_path("airspace.cub")?;

    // Iterate over all airspaces
    for result in reader.read_airspaces() {
        let airspace = result?;

        // All fields are decoded and ready to use
        println!("{}: {:?} {:?}", airspace.name, airspace.style, airspace.class);
        println!("  Altitude: {} - {} meters", airspace.min_alt, airspace.max_alt);
        println!("  Points: {}", airspace.points.len());
    }

    Ok(())
}
```

#### Writing

The high-level writer uses a builder pattern with automatic calculations:

```rust,no_run
use seeyou_cub::writer::CubWriter;
use seeyou_cub::{Airspace, Point, CubStyle, CubClass, AltStyle, DaysActive};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let airspace = Airspace {
        name: "My Airspace".to_string(),
        points: vec![
            Point::lat_lon(0.8, 0.4),
            Point::lat_lon(0.81, 0.41),
            Point::lat_lon(0.82, 0.42),
        ],
        style: CubStyle::DangerArea,
        class: CubClass::ClassD,
        min_alt: 0,
        max_alt: 5000,
        min_alt_style: AltStyle::MeanSeaLevel,
        max_alt_style: AltStyle::MeanSeaLevel,
        days_active: DaysActive::all(),
        ..Default::default()
    };

    CubWriter::new("My Airspace Data")
        .add_airspace(airspace)
        .write_to_path("output.cub")?;

    Ok(())
}
```

### Low-Level API

The low-level API provides direct access to raw file data with minimal transformation:

```rust,no_run
use seeyou_cub::raw::{Header, Item, ItemData, HEADER_SIZE};
use std::fs::File;
use std::io::{Seek, SeekFrom};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("airspace.cub")?;

    // Read header
    let header = Header::read(&mut file)?;
    println!("CUB file: {:?}", header.title);

    // Read items (airspace metadata)
    for i in 0..header.hdr_items {
        let offset = HEADER_SIZE as u64 + (i as u64 * header.size_of_item as u64);
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
