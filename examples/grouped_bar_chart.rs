//! Grouped bar chart example
//!
//! Demonstrates multiple data series grouped by category using the color encoding.
//!
//! Run with: cargo run --example grouped_bar_chart
//! Save output: cargo run --example grouped_bar_chart > grouped_bar_chart.svg

use std::fs;
use vis_core::chart;
use vis_render::render_svg;

fn main() {
    let spec =
        fs::read_to_string("examples/grouped_bar.json").expect("Failed to read examples/grouped_bar.json");

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
