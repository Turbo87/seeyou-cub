#![expect(unused_imports)]

use seeyou_cub::{Airspace, CubReader, CubStyle};
use std::env;
use std::fs;
use std::io;

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
