use serde_json::Value;
use std::collections::HashMap;

use super::stack::{compute_stack, max_stacked_value, min_stacked_value};
use super::{extract_categories, extract_numbers, generate_axis, infer_data_type, CompileError, PlotArea};
use crate::ir::{Color, Geometry, Group, Mark, MarkItem, MarkType, Point, Stroke, Transform};
use crate::scale::{BandScale, LinearScale};
use crate::spec::{AxisOrient, DataType, Encoding, StackConfig, StackMode};

use super::bar::COLORS;

/// Compile line chart encoding to scene graph
pub fn compile_line(
    encoding: &Encoding,
    data: &[Value],
    plot_area: &PlotArea,
    stack_config: Option<&StackConfig>,
) -> Result<Group, CompileError> {
    // Get x and y channels
    let x_channel = encoding
        .x
        .as_ref()
        .ok_or_else(|| CompileError::MissingField("encoding.x".to_string()))?;
    let y_channel = encoding
        .y
        .as_ref()
        .ok_or_else(|| CompileError::MissingField("encoding.y".to_string()))?;

    let x_field = x_channel
        .field()
        .ok_or_else(|| CompileError::InvalidEncoding("x must have a field".to_string()))?;
    let y_field = y_channel
        .field()
        .ok_or_else(|| CompileError::InvalidEncoding("y must have a field".to_string()))?;

    // Infer data types
    let x_type = x_channel.data_type().unwrap_or_else(|| infer_data_type(data, x_field));

    // Extract unique x categories for band scale
    let categories = extract_categories(data, x_field);
    let unique_categories: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        categories
            .iter()
            .filter(|c| seen.insert((*c).clone()))
            .cloned()
            .collect()
    };

    // Check for color encoding (multiple series)
    let color_field = encoding
        .color
        .as_ref()
        .and_then(|c| c.field())
        .map(|s| s.to_string());

    // Determine if we should stack
    let should_stack = color_field.is_some()
        && stack_config.map_or(false, |sc| !matches!(sc, StackConfig::Enabled(false)));

    // Create x scale (band for categorical, linear for quantitative)
    let _x_is_categorical = matches!(x_type, DataType::Nominal | DataType::Ordinal);

    let mut line_items = Vec::new();
    let mut area_items = Vec::new();

    if let Some(ref color_f) = color_field {
        if should_stack {
            // Stacked lines/areas
            let stack_cfg = stack_config.cloned().unwrap_or(StackConfig::Enabled(true));
            let stacked = compute_stack(data, x_field, y_field, color_f, &stack_cfg);

            let max_val = max_stacked_value(&stacked);
            let min_val = min_stacked_value(&stacked);

            let (domain_min, domain_max) = match &stack_cfg {
                StackConfig::Mode(StackMode::Normalize) => (0.0, 1.0),
                StackConfig::Mode(StackMode::Center) => (min_val, max_val),
                _ => (0.0, max_val),
            };

            let cat_scale = BandScale::new(unique_categories.clone(), (0.0, plot_area.width)).padding(0.0);
            let val_scale = LinearScale::new((domain_min, domain_max), (plot_area.height, 0.0)).nice();

            // Get unique series
            let color_values: Vec<String> = extract_categories(data, color_f);
            let unique_colors: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                color_values
                    .iter()
                    .filter(|c| seen.insert((*c).clone()))
                    .cloned()
                    .collect()
            };

            // Group stacked values by series, preserving category order
            let mut by_series: HashMap<String, Vec<(String, f64, f64)>> = HashMap::new();
            for sv in &stacked {
                by_series
                    .entry(sv.series.clone())
                    .or_default()
                    .push((sv.category.clone(), sv.y0, sv.y1));
            }

            // Create area fills for stacked lines
            for (series, values) in &by_series {
                let color_idx = unique_colors.iter().position(|c| c == series).unwrap_or(0);
                let color = Color::from_hex(COLORS[color_idx % COLORS.len()]).unwrap();

                // Sort by category order
                let mut sorted_values: Vec<_> = values.clone();
                sorted_values.sort_by(|a, b| {
                    let idx_a = unique_categories.iter().position(|c| c == &a.0).unwrap_or(0);
                    let idx_b = unique_categories.iter().position(|c| c == &b.0).unwrap_or(0);
                    idx_a.cmp(&idx_b)
                });

                // Build points for top line and baseline
                let mut top_points = Vec::new();
                let mut baseline_points = Vec::new();

                for (cat, y0, y1) in &sorted_values {
                    let x = cat_scale.scale(cat).unwrap_or(0.0) + cat_scale.bandwidth() / 2.0;
                    top_points.push(Point::new(x, val_scale.scale(*y1)));
                    baseline_points.push(Point::new(x, val_scale.scale(*y0)));
                }

                // Create area
                area_items.push(
                    MarkItem::new(Geometry::Area {
                        points: top_points.clone(),
                        baseline: baseline_points,
                    })
                    .with_fill(color)
                    .with_opacity(0.7),
                );

                // Create line on top
                line_items.push(
                    MarkItem::new(Geometry::Line { points: top_points })
                        .with_stroke(Stroke::solid(color, 2.0)),
                );
            }

            return build_line_group(line_items, area_items, &cat_scale, &val_scale, encoding, plot_area);
        } else {
            // Multiple lines (no stacking)
            let values = extract_numbers(data, y_field);
            let max_value = values.iter().cloned().fold(0.0_f64, f64::max);

            let cat_scale = BandScale::new(unique_categories.clone(), (0.0, plot_area.width)).padding(0.0);
            let val_scale = LinearScale::new((0.0, max_value), (plot_area.height, 0.0)).nice().zero();

            // Get unique series
            let color_values: Vec<String> = extract_categories(data, color_f);
            let unique_colors: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                color_values
                    .iter()
                    .filter(|c| seen.insert((*c).clone()))
                    .cloned()
                    .collect()
            };

            // Group data by series
            let mut by_series: HashMap<String, Vec<(String, f64)>> = HashMap::new();
            for row in data {
                let cat = extract_string(row, x_field);
                let series = extract_string(row, color_f);
                let val = row.get(y_field).and_then(|v| v.as_f64());

                if let (Some(cat), Some(series), Some(val)) = (cat, series, val) {
                    by_series.entry(series).or_default().push((cat, val));
                }
            }

            for (series, values) in &by_series {
                let color_idx = unique_colors.iter().position(|c| c == series).unwrap_or(0);
                let color = Color::from_hex(COLORS[color_idx % COLORS.len()]).unwrap();

                // Sort by category order
                let mut sorted_values: Vec<_> = values.clone();
                sorted_values.sort_by(|a, b| {
                    let idx_a = unique_categories.iter().position(|c| c == &a.0).unwrap_or(0);
                    let idx_b = unique_categories.iter().position(|c| c == &b.0).unwrap_or(0);
                    idx_a.cmp(&idx_b)
                });

                let points: Vec<Point> = sorted_values
                    .iter()
                    .map(|(cat, val)| {
                        let x = cat_scale.scale(cat).unwrap_or(0.0) + cat_scale.bandwidth() / 2.0;
                        let y = val_scale.scale(*val);
                        Point::new(x, y)
                    })
                    .collect();

                line_items.push(
                    MarkItem::new(Geometry::Line { points })
                        .with_stroke(Stroke::solid(color, 2.0)),
                );
            }

            return build_line_group(line_items, area_items, &cat_scale, &val_scale, encoding, plot_area);
        }
    }

    // Simple line (single series)
    let values = extract_numbers(data, y_field);
    let max_value = values.iter().cloned().fold(0.0_f64, f64::max);

    let cat_scale = BandScale::new(unique_categories.clone(), (0.0, plot_area.width)).padding(0.0);
    let val_scale = LinearScale::new((0.0, max_value), (plot_area.height, 0.0)).nice().zero();

    let default_color = Color::from_hex(COLORS[0]).unwrap();

    // Build points in category order
    let mut points_map: HashMap<String, f64> = HashMap::new();
    for row in data {
        let cat = extract_string(row, x_field);
        let val = row.get(y_field).and_then(|v| v.as_f64());

        if let (Some(cat), Some(val)) = (cat, val) {
            points_map.insert(cat, val);
        }
    }

    let points: Vec<Point> = unique_categories
        .iter()
        .filter_map(|cat| {
            points_map.get(cat).map(|val| {
                let x = cat_scale.scale(cat).unwrap_or(0.0) + cat_scale.bandwidth() / 2.0;
                let y = val_scale.scale(*val);
                Point::new(x, y)
            })
        })
        .collect();

    line_items.push(
        MarkItem::new(Geometry::Line { points })
            .with_stroke(Stroke::solid(default_color, 2.0)),
    );

    build_line_group(line_items, area_items, &cat_scale, &val_scale, encoding, plot_area)
}

fn build_line_group(
    line_items: Vec<MarkItem>,
    area_items: Vec<MarkItem>,
    cat_scale: &BandScale,
    val_scale: &LinearScale,
    encoding: &Encoding,
    plot_area: &PlotArea,
) -> Result<Group, CompileError> {
    let mut root = Group::new().with_transform(Transform::translate(plot_area.x, plot_area.y));

    // Add area marks first (behind lines)
    if !area_items.is_empty() {
        root.add_mark(Mark {
            mark_type: MarkType::Area,
            items: area_items,
        });
    }

    // Add line marks
    root.add_mark(Mark {
        mark_type: MarkType::Line,
        items: line_items,
    });

    // Generate axes
    let x_axis_ticks = cat_scale.ticks();
    let y_axis_ticks: Vec<crate::scale::Tick> = val_scale
        .ticks(5)
        .into_iter()
        .map(|t| crate::scale::Tick {
            value: val_scale.scale(t.value),
            label: t.label,
        })
        .collect();

    // Get axis titles
    let x_title = encoding
        .x
        .as_ref()
        .and_then(|c| c.axis())
        .and_then(|a| a.title.as_deref());
    let y_title = encoding
        .y
        .as_ref()
        .and_then(|c| c.axis())
        .and_then(|a| a.title.as_deref());

    // Add x-axis
    for mark in generate_axis(AxisOrient::Bottom, &x_axis_ticks, plot_area, x_title) {
        root.add_mark(mark);
    }

    // Add y-axis
    for mark in generate_axis(AxisOrient::Left, &y_axis_ticks, plot_area, y_title) {
        root.add_mark(mark);
    }

    Ok(root)
}

fn extract_string(row: &Value, field: &str) -> Option<String> {
    row.get(field).map(|v| match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => v.to_string(),
    })
}
