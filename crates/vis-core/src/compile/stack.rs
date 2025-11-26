use serde_json::Value;
use std::collections::HashMap;

use crate::spec::{StackConfig, StackMode};

/// Result of stacking computation for a single data point
#[derive(Debug, Clone)]
pub struct StackedValue {
    /// Original row data
    pub row: Value,
    /// Stacked y0 (bottom of bar/area)
    pub y0: f64,
    /// Stacked y1 (top of bar/area)
    pub y1: f64,
    /// Category value (x-axis grouping)
    pub category: String,
    /// Color/series value
    pub series: String,
}

/// Compute stacked values from data
///
/// Groups data by category_field, then stacks values within each group
/// according to the stack configuration.
pub fn compute_stack(
    data: &[Value],
    category_field: &str,
    value_field: &str,
    series_field: &str,
    stack_config: &StackConfig,
) -> Vec<StackedValue> {
    let mode = match stack_config {
        StackConfig::Enabled(false) => return vec![], // No stacking
        StackConfig::Enabled(true) => StackMode::Zero,
        StackConfig::Mode(mode) => *mode,
    };

    // Group data by category
    let mut by_category: HashMap<String, Vec<(String, f64, Value)>> = HashMap::new();

    for row in data {
        let category = extract_string(row, category_field).unwrap_or_default();
        let series = extract_string(row, series_field).unwrap_or_default();
        let value = row.get(value_field).and_then(|v| v.as_f64()).unwrap_or(0.0);

        by_category
            .entry(category)
            .or_default()
            .push((series, value, row.clone()));
    }

    // Compute stacked values
    let mut results = Vec::new();

    for (category, items) in by_category {
        // Compute total for normalization
        let total: f64 = items.iter().map(|(_, v, _)| *v).sum();

        let mut cumulative = 0.0;

        for (series, value, row) in items {
            let (y0, y1) = match mode {
                StackMode::Zero => {
                    let y0 = cumulative;
                    let y1 = cumulative + value;
                    cumulative = y1;
                    (y0, y1)
                }
                StackMode::Normalize => {
                    if total == 0.0 {
                        (0.0, 0.0)
                    } else {
                        let y0 = cumulative;
                        let y1 = cumulative + value / total;
                        cumulative = y1;
                        (y0, y1)
                    }
                }
                StackMode::Center => {
                    // Center mode: stack around zero
                    let y0 = cumulative - total / 2.0;
                    let y1 = y0 + value;
                    cumulative += value;
                    (y0, y1)
                }
            };

            results.push(StackedValue {
                row,
                y0,
                y1,
                category: category.clone(),
                series,
            });
        }
    }

    results
}

/// Compute the maximum stacked value (for scale domain)
pub fn max_stacked_value(stacked: &[StackedValue]) -> f64 {
    stacked
        .iter()
        .map(|s| s.y1)
        .fold(0.0_f64, f64::max)
}

/// Compute the minimum stacked value (for scale domain, needed for center mode)
pub fn min_stacked_value(stacked: &[StackedValue]) -> f64 {
    stacked
        .iter()
        .map(|s| s.y0)
        .fold(f64::INFINITY, f64::min)
}

fn extract_string(row: &Value, field: &str) -> Option<String> {
    row.get(field).map(|v| match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => v.to_string(),
    })
}
