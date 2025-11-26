//! Horizontal bar chart example
//!
//! Demonstrates horizontal bars by swapping x and y encoding.
//! When quantitative is on x and nominal on y, bars are drawn horizontally.
//!
//! Run with: cargo run --example horizontal_bar_chart
//! Save output: cargo run --example horizontal_bar_chart > horizontal_bar_chart.svg

use std::fs;
use vis_core::chart;
use vis_render::render_svg;

fn main() {
    let spec = fs::read_to_string("examples/horizontal_bar.json")
        .expect("Failed to read examples/horizontal_bar.json");

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
