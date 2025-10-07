use seeyou_cub::CubReader;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <cub-file>", args[0]);
        std::process::exit(1);
    }

    let mut reader = CubReader::from_path(&args[1])?;
    let mut warnings = Vec::new();

    let header = reader.read_header(&mut warnings)?;

    println!("=== CUB File Info ===");
    println!("Header: {}", header.title);

    let (w, s, e, n) = header.bounding_box();
    println!("Bounds: W={:.4} S={:.4} E={:.4} N={:.4}", w, s, e, n);

    let items: Vec<_> = reader
        .read_items(&header, &mut warnings)
        .collect::<Result<Vec<_>, _>>()?;

    println!("Airspaces: {}", items.len());

    println!("\n=== First 10 Airspaces ===");
    for (i, item) in items.iter().take(10).enumerate() {
        println!("{}. {:?} {:?}", i + 1, item.style(), item.class());
        println!(
            "   Altitude: {} - {} meters ({:?} - {:?})",
            item.min_alt,
            item.max_alt,
            item.min_alt_style(),
            item.max_alt_style()
        );

        // Parse points
        let points: Vec<_> = reader
            .read_points(&header, item, &mut warnings)
            .collect::<Result<Vec<_>, _>>()?;

        println!("   Points: {}", points.len());

        if let Some(first_pt) = points.first() {
            if let Some(name) = &first_pt.name {
                println!("   Name: {}", name);
            }
            if let Some(freq) = first_pt.frequency {
                println!("   Frequency: {} Hz", freq);
            }
        }
    }

    if !warnings.is_empty() {
        println!("\n=== Warnings ({}) ===", warnings.len());
        for warning in &warnings {
            println!("  {:?}", warning);
        }
    }

    Ok(())
}
