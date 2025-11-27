//! Normalized (100%) stacked bar chart example
//!
//! Demonstrates normalized stacking where bars show proportions summing to 100%.
//! Uses `"stack": "normalize"` to enable normalized stacking.
//!
//! Stack options:
//! - false: No stacking (grouped bars)
//! - true or "zero": Standard stacking from zero baseline
//! - "normalize": Normalize to 100% (0-1 range)
//! - "center": Center around zero (diverging)
//!
//! Run with: cargo run --example normalized_stacked_bar_chart
//! Save output: cargo run --example normalized_stacked_bar_chart > normalized_stacked_bar_chart.svg

use std::fs;
use vis_core::chart;
use vis_render::render_svg;

fn main() {
    let spec = fs::read_to_string("examples/normalized_stacked_bar.json")
        .expect("Failed to read examples/normalized_stacked_bar.json");

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
