//! Simple bar chart example
//!
//! Run with: cargo run --example bar_chart
//! Save output: cargo run --example bar_chart > bar_chart.svg

use std::fs;
use vis_core::chart;
use vis_render::render_svg;

fn main() {
    let spec = fs::read_to_string("examples/bar.json").expect("Failed to read examples/bar.json");

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
