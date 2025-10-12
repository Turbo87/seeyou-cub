#![expect(unused_imports)]

use seeyou_cub::{Airspace, CubReader, CubStyle};
use serde_json::json;
use std::env;
use std::fs;
use std::io;

/// Maps airspace style to hex color following aviation conventions
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

/// Converts an Airspace to a GeoJSON Feature
#[cfg_attr(not(test), allow(dead_code))]
fn airspace_to_geojson_feature(airspace: &Airspace) -> serde_json::Value {
    // Convert points: radians -> degrees, lat/lon -> [lon, lat]
    let mut coordinates: Vec<[f32; 2]> = airspace
        .points
        .iter()
        .map(|pt| [pt.lon.to_degrees(), pt.lat.to_degrees()])
        .collect();

    // Ensure polygon is closed (first point = last point)
    if let Some(first) = coordinates.first().copied()
        && coordinates.last() != Some(&first)
    {
        coordinates.push(first);
    }

    json!({
        "type": "Feature",
        "geometry": {
            "type": "Polygon",
            "coordinates": [coordinates]
        },
        "properties": {
            "name": airspace.name,
            "class": format!("{:?}", airspace.class),
            "style": format!("{:?}", airspace.style),
            "min_alt": airspace.min_alt,
            "max_alt": airspace.max_alt,
            "min_alt_style": format!("{:?}", airspace.min_alt_style),
            "max_alt_style": format!("{:?}", airspace.max_alt_style),
            "color": style_to_color(&airspace.style)
        }
    })
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
    use insta::assert_debug_snapshot;
    use seeyou_cub::{AltStyle, CubClass, DaysActive, Point};

    #[test]
    fn test_style_to_color() {
        assert_eq!(style_to_color(&CubStyle::ProhibitedArea), "#8B0000");
        assert_eq!(style_to_color(&CubStyle::RestrictedArea), "#DC143C");
        assert_eq!(style_to_color(&CubStyle::DangerArea), "#FF4500");
        assert_eq!(style_to_color(&CubStyle::ControlZone), "#4169E1");
        assert_eq!(style_to_color(&CubStyle::GliderSector), "#FFD700");
    }

    #[test]
    fn test_airspace_to_geojson_feature() {
        let airspace = Airspace {
            name: "Test Zone".to_string(),
            points: vec![
                Point { lat: 0.5, lon: 0.4 }, // ~28.6° lat, ~22.9° lon
                Point { lat: 0.6, lon: 0.5 },
                Point { lat: 0.5, lon: 0.6 },
            ],
            style: CubStyle::ControlZone,
            class: CubClass::ClassD,
            min_alt: 0,
            max_alt: 5000,
            min_alt_style: AltStyle::MeanSeaLevel,
            max_alt_style: AltStyle::MeanSeaLevel,
            days_active: DaysActive::all(),
            ..Default::default()
        };

        let feature = airspace_to_geojson_feature(&airspace);
        assert_debug_snapshot!(feature);
    }
}
