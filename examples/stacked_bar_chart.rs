//! Stacked bar chart example
//!
//! Demonstrates stacked bars where values are accumulated on top of each other.
//! Uses `"stack": true` to enable stacking behavior.
//!
//! Run with: cargo run --example stacked_bar_chart
//! Save output: cargo run --example stacked_bar_chart > stacked_bar_chart.svg

use std::fs;
use vis_core::chart;
use vis_render::render_svg;

fn main() {
    let spec =
        fs::read_to_string("examples/stacked_bar.json").expect("Failed to read examples/stacked_bar.json");

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
