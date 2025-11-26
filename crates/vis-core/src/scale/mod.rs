mod band;
mod linear;

pub use band::BandScale;
pub use linear::LinearScale;

use serde_json::Value;

/// Tick mark for axis rendering
#[derive(Debug, Clone)]
pub struct Tick {
    pub value: f64,
    pub label: String,
}

/// Extract numeric value from JSON
pub fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// Extract string value from JSON
pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

/// Compute nice tick values for a numeric range
pub fn nice_ticks(min: f64, max: f64, count: usize) -> Vec<f64> {
    if count == 0 || min >= max {
        return vec![];
    }

    let range = max - min;
    let rough_step = range / count as f64;

    // Find a nice step size (1, 2, 5, 10, 20, 50, etc.)
    let magnitude = 10_f64.powf(rough_step.log10().floor());
    let residual = rough_step / magnitude;

    let nice_step = if residual <= 1.5 {
        magnitude
    } else if residual <= 3.0 {
        2.0 * magnitude
    } else if residual <= 7.0 {
        5.0 * magnitude
    } else {
        10.0 * magnitude
    };

    // Generate ticks
    let start = (min / nice_step).ceil() * nice_step;
    let mut ticks = Vec::new();
    let mut tick = start;
    while tick <= max + nice_step * 0.001 {
        ticks.push(tick);
        tick += nice_step;
    }

    ticks
}

/// Format a numeric value for display
pub fn format_number(value: f64) -> String {
    if value.abs() >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value.abs() >= 1_000.0 {
        format!("{:.1}K", value / 1_000.0)
    } else if value.fract() == 0.0 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
    }
}
