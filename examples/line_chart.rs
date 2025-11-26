//! Simple line chart example
//!
//! Run with: cargo run --example line_chart
//! Save output: cargo run --example line_chart 2>/dev/null > line_chart.svg

use std::fs;
use vis_core::chart;
use vis_render::render_svg;

fn main() {
    let spec = fs::read_to_string("examples/line.json").expect("Failed to read examples/line.json");

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
