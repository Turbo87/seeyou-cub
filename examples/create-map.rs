#![expect(unused_imports)]

use seeyou_cub::{Airspace, CubReader, CubStyle};
use serde_json::json;
use std::env;
use std::fs;
use std::io;

const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>CUB Airspace Map</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <script src='https://unpkg.com/maplibre-gl@4/dist/maplibre-gl.js'></script>
    <link href='https://unpkg.com/maplibre-gl@4/dist/maplibre-gl.css' rel='stylesheet' />
    <style>
        body { margin: 0; padding: 0; }
        #map { position: absolute; top: 0; bottom: 0; width: 100%; }
        .maplibregl-popup-content {
            padding: 10px;
            max-width: 300px;
        }
        .maplibregl-popup-content h3 {
            margin: 0 0 5px 0;
            font-size: 14px;
        }
        .maplibregl-popup-content p {
            margin: 3px 0;
            font-size: 12px;
        }
    </style>
</head>
<body>
    <div id="map"></div>
    <script>
        const geojsonData = {GEOJSON};
        const bounds = {BOUNDS};

        const map = new maplibregl.Map({
            container: 'map',
            style: {
                version: 8,
                sources: {
                    'osm': {
                        type: 'raster',
                        tiles: ['https://tile.openstreetmap.org/{z}/{x}/{y}.png'],
                        tileSize: 256,
                        attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
                    }
                },
                layers: [{
                    id: 'osm',
                    type: 'raster',
                    source: 'osm',
                    minzoom: 0,
                    maxzoom: 19
                }]
            },
            bounds: bounds,
            fitBoundsOptions: { padding: 50 }
        });

        map.on('load', () => {
            // Add airspace data source
            map.addSource('airspaces', {
                type: 'geojson',
                data: geojsonData
            });

            // Add fill layer for airspace polygons
            map.addLayer({
                id: 'airspaces-fill',
                type: 'fill',
                source: 'airspaces',
                paint: {
                    'fill-color': ['get', 'color'],
                    'fill-opacity': 0.35
                }
            });

            // Add line layer for airspace borders
            map.addLayer({
                id: 'airspaces-line',
                type: 'line',
                source: 'airspaces',
                paint: {
                    'line-color': ['get', 'color'],
                    'line-width': 2,
                    'line-opacity': 0.8
                }
            });

            // Change cursor on hover
            map.on('mouseenter', 'airspaces-fill', () => {
                map.getCanvas().style.cursor = 'pointer';
            });

            map.on('mouseleave', 'airspaces-fill', () => {
                map.getCanvas().style.cursor = '';
            });

            // Show popup on click
            map.on('click', 'airspaces-fill', (e) => {
                const props = e.features[0].properties;

                const popupContent = `
                    <h3>${props.name}</h3>
                    <p><strong>Type:</strong> ${props.style}</p>
                    <p><strong>Class:</strong> ${props.class}</p>
                    <p><strong>Altitude:</strong> ${props.min_alt} ${props.min_alt_style} - ${props.max_alt} ${props.max_alt_style}</p>
                `;

                new maplibregl.Popup()
                    .setLngLat(e.lngLat)
                    .setHTML(popupContent)
                    .addTo(map);
            });
        });
    </script>
</body>
</html>
"#;

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

/// Generates HTML with embedded GeoJSON data and bounding box
#[cfg_attr(not(test), allow(dead_code))]
fn generate_html(geojson: &str, bounds: [[f32; 2]; 2]) -> String {
    let bounds_json = format!(
        "[[{}, {}], [{}, {}]]",
        bounds[0][0], bounds[0][1], bounds[1][0], bounds[1][1]
    );

    HTML_TEMPLATE
        .replace("{GEOJSON}", geojson)
        .replace("{BOUNDS}", &bounds_json)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: create-map <path-to-cub-file>");
        std::process::exit(1);
    }

    let cub_path = &args[1];

    // Open CUB file
    let mut reader = CubReader::from_path(cub_path)
        .map_err(|e| format!("Error: Cannot open CUB file: {}", e))?;

    // Extract bounding box from header (for initial map view)
    let bbox = reader.bounding_box();
    let bounds = [
        [bbox.left.to_degrees(), bbox.bottom.to_degrees()],
        [bbox.right.to_degrees(), bbox.top.to_degrees()],
    ];

    // Read all airspaces, collecting valid ones
    let features: Vec<_> = reader
        .read_airspaces()
        .filter_map(|result| {
            result
                .inspect_err(|err| eprintln!("Warning: Failed to parse airspace: {err}"))
                .ok()
        })
        // Skip invalid polygons (need at least 3 points)
        .filter_map(|airspace| {
            if airspace.points.len() < 3 {
                eprintln!(
                    "Warning: Skipping airspace '{}' with only {} points",
                    airspace.name,
                    airspace.points.len()
                );
                None
            } else {
                Some(airspace)
            }
        })
        .map(|airspace| airspace_to_geojson_feature(&airspace))
        .collect();

    // Build GeoJSON FeatureCollection
    let geojson = serde_json::json!({
        "type": "FeatureCollection",
        "features": features
    });

    let geojson_string = serde_json::to_string(&geojson)?;

    // Generate HTML
    let html = generate_html(&geojson_string, bounds);

    // Write to map.html
    fs::write("map.html", html).map_err(|e| format!("Error: Cannot write to map.html: {}", e))?;

    // Print success message with absolute path
    let absolute_path = fs::canonicalize("map.html")?;
    println!("Map generated: {}", absolute_path.display());

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

    #[test]
    fn test_generate_html() {
        let geojson = r#"{"type": "FeatureCollection", "features": []}"#;
        let bounds = [[-122.5, 37.5], [-122.0, 38.0]];

        let html = generate_html(geojson, bounds);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("maplibre-gl"));
        assert!(html.contains(geojson));
        assert!(html.contains("[[-122.5, 37.5], [-122, 38]]"));
    }
}
