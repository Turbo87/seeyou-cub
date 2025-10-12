#![expect(unused_imports)]

use seeyou_cub::{Airspace, CubReader, CubStyle};
use std::env;
use std::fs;
use std::io;

/// Maps airspace style to hex color following aviation conventions
#[expect(dead_code)]
fn style_to_color(style: &CubStyle) -> &'static str {
    match style {
        CubStyle::ProhibitedArea => "#8B0000",           // Dark red
        CubStyle::RestrictedArea => "#DC143C",           // Red
        CubStyle::DangerArea => "#FF4500",               // Orange red
        CubStyle::ControlZone => "#4169E1",              // Blue
        CubStyle::TerminalControlArea => "#87CEEB",      // Light blue
        CubStyle::ControlArea => "#9370DB",              // Purple
        CubStyle::GliderSector => "#FFD700",             // Yellow
        CubStyle::TransponderMandatoryZone => "#FF8C00", // Dark orange
        CubStyle::Warning => "#FFA500",                  // Orange
        CubStyle::Alert => "#FF6347",                    // Tomato
        _ => "#808080",                                  // Gray (unknown/other)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: create-map <path-to-cub-file>");
        std::process::exit(1);
    }

    let _cub_path = &args[1];

    // TODO: Implementation goes here

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_to_color() {
        assert_eq!(style_to_color(&CubStyle::ProhibitedArea), "#8B0000");
        assert_eq!(style_to_color(&CubStyle::RestrictedArea), "#DC143C");
        assert_eq!(style_to_color(&CubStyle::DangerArea), "#FF4500");
        assert_eq!(style_to_color(&CubStyle::ControlZone), "#4169E1");
        assert_eq!(style_to_color(&CubStyle::GliderSector), "#FFD700");
    }
}
