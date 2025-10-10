use seeyou_cub::CubReader;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <cub-file>", args[0]);
        std::process::exit(1);
    }

    let mut reader = CubReader::from_path(&args[1])?;

    println!("=== CUB File Info ===");
    println!("Title: {}", reader.title());

    let header = reader.header();

    let (w, s, e, n) = header.bounding_box();
    println!(
        "Bounds: W={:.4} S={:.4} E={:.4} N={:.4}",
        w.to_degrees(),
        s.to_degrees(),
        e.to_degrees(),
        n.to_degrees()
    );

    let results: Vec<_> = reader.read_airspaces().collect::<Result<Vec<_>, _>>()?;

    println!("Airspaces: {}", results.len());

    println!("\n=== First 10 Airspaces ===");
    for (i, airspace) in results.iter().take(10).enumerate() {
        println!("{}. {:?} {:?}", i + 1, airspace.style, airspace.class);
        println!(
            "   Altitude: {} - {} meters ({:?} - {:?})",
            airspace.min_alt, airspace.max_alt, airspace.min_alt_style, airspace.max_alt_style
        );

        println!("   Points: {}", airspace.points.len());

        if let Some(name) = &airspace.name {
            println!("   Name: {}", name);
        }
        if let Some(freq) = airspace.frequency {
            println!("   Frequency: {} Hz", freq);
        }
    }

    Ok(())
}
