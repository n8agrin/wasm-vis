//! Aggregated bar chart example
//!
//! Demonstrates float data types and aggregation functions.
//! Uses mean aggregation to average temperature values per city.
//!
//! Supported aggregations: count, sum, mean, median, min, max, distinct
//!
//! Run with: cargo run --example aggregated_bar_chart
//! Save output: cargo run --example aggregated_bar_chart > aggregated_bar_chart.svg

use std::fs;
use vis_core::chart;
use vis_render::render_svg;

fn main() {
    let spec = fs::read_to_string("examples/aggregated_bar.json")
        .expect("Failed to read examples/aggregated_bar.json");

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
