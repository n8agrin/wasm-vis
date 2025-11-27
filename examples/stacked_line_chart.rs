//! Stacked line/area chart example
//!
//! Demonstrates stacked areas with lines on top. Uses stack: true to enable stacking.
//!
//! Run with: cargo run --example stacked_line_chart
//! Save output: cargo run --example stacked_line_chart 2>/dev/null > stacked_line_chart.svg

use std::fs;
use vis_core::chart;
use vis_render::render_svg;

fn main() {
    let spec = fs::read_to_string("examples/stacked_line.json")
        .expect("Failed to read examples/stacked_line.json");

    match chart(&spec) {
        Ok(scene) => {
            let svg = render_svg(&scene);
            println!("{}", svg);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
