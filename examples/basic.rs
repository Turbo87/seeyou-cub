use seeyou_cub::parse;
use std::env;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <cub-file>", args[0]);
        std::process::exit(1);
    }

    let file = File::open(&args[1])?;
    let (mut cub, warnings) = parse(file)?;

    println!("=== CUB File Info ===");
    println!("Header: {}", cub.header().title);
    println!("Airspaces: {}", cub.items().len());

    let (w, s, e, n) = cub.header().bounding_box();
    println!("Bounds: W={:.4} S={:.4} E={:.4} N={:.4}", w, s, e, n);

    if !warnings.is_empty() {
        println!("\n=== Warnings ({}) ===", warnings.len());
        for warning in warnings {
            println!("  {:?}", warning);
        }
    }

    println!("\n=== First 10 Airspaces ===");
    for i in 0..cub.items().len().min(10) {
        let item = cub.items()[i].clone();
        println!("{}. {:?} {:?}", i + 1, item.style(), item.class());
        println!("   Altitude: {} - {} meters ({:?} - {:?})",
            item.min_alt, item.max_alt,
            item.min_alt_style(), item.max_alt_style()
        );

        // Parse points
        let points: Vec<_> = cub.read_points(&item)?
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

    Ok(())
}