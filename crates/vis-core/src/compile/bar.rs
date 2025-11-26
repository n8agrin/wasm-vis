use serde_json::Value;

use super::stack::{compute_stack, max_stacked_value, min_stacked_value};
use super::{extract_categories, extract_numbers, generate_axis, infer_data_type, CompileError, PlotArea};
use crate::ir::{Color, Geometry, Group, Mark, MarkItem, MarkType, Transform};
use crate::scale::{BandScale, LinearScale};
use crate::spec::{AxisOrient, DataType, Encoding, StackConfig, StackMode};

/// Default color palette (hotpink is the default/first color)
pub const COLORS: &[&str] = &[
    "#ff69b4", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc949", "#af7aa1", "#ff9da7",
    "#9c755f", "#bab0ab",
];

/// Compile bar chart encoding to scene graph
pub fn compile_bar(
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

    // Infer data types if not specified
    let x_type = x_channel.data_type().unwrap_or_else(|| infer_data_type(data, x_field));
    let y_type = y_channel.data_type().unwrap_or_else(|| infer_data_type(data, y_field));

    // Determine orientation: if x is quantitative and y is nominal, horizontal bars
    let is_horizontal = matches!(x_type, DataType::Quantitative)
        && matches!(y_type, DataType::Nominal | DataType::Ordinal);

    // Create scales based on orientation
    let (cat_field, val_field) = if is_horizontal {
        (y_field, x_field)
    } else {
        (x_field, y_field)
    };

    let categories = extract_categories(data, cat_field);
    let unique_categories: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        categories
            .iter()
            .filter(|c| seen.insert((*c).clone()))
            .cloned()
            .collect()
    };

    // Check for color encoding (grouped/stacked bars)
    let color_field = encoding
        .color
        .as_ref()
        .and_then(|c| c.field())
        .map(|s| s.to_string());

    // Determine if we should stack
    let should_stack = color_field.is_some()
        && stack_config.map_or(false, |sc| !matches!(sc, StackConfig::Enabled(false)));

    // Build bar marks
    let mut bar_items = Vec::new();

    if let Some(ref color_f) = color_field {
        if should_stack {
            // Stacked bars
            let stack_cfg = stack_config.cloned().unwrap_or(StackConfig::Enabled(true));
            let stacked = compute_stack(data, cat_field, val_field, color_f, &stack_cfg);

            // Determine scale domain from stacked values
            let max_val = max_stacked_value(&stacked);
            let min_val = min_stacked_value(&stacked);

            // For normalize mode, domain is 0-1
            let (domain_min, domain_max) = match &stack_cfg {
                StackConfig::Mode(StackMode::Normalize) => (0.0, 1.0),
                StackConfig::Mode(StackMode::Center) => (min_val, max_val),
                _ => (0.0, max_val),
            };

            // Create scales
            let (cat_scale, val_scale) = if is_horizontal {
                let cat_scale = BandScale::new(unique_categories.clone(), (0.0, plot_area.height)).padding(0.2);
                let val_scale = LinearScale::new((domain_min, domain_max), (0.0, plot_area.width)).nice();
                (cat_scale, val_scale)
            } else {
                let cat_scale = BandScale::new(unique_categories.clone(), (0.0, plot_area.width)).padding(0.2);
                let val_scale = LinearScale::new((domain_min, domain_max), (plot_area.height, 0.0)).nice();
                (cat_scale, val_scale)
            };

            // Get unique color values for color assignment
            let color_values: Vec<String> = extract_categories(data, color_f);
            let unique_colors: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                color_values
                    .iter()
                    .filter(|c| seen.insert((*c).clone()))
                    .cloned()
                    .collect()
            };

            let bandwidth = cat_scale.bandwidth();

            for sv in &stacked {
                let color_idx = unique_colors.iter().position(|c| c == &sv.series).unwrap_or(0);
                let color = Color::from_hex(COLORS[color_idx % COLORS.len()]).unwrap();

                if is_horizontal {
                    let y = cat_scale.scale(&sv.category).unwrap_or(0.0);
                    let x0 = val_scale.scale(sv.y0);
                    let x1 = val_scale.scale(sv.y1);
                    bar_items.push(
                        MarkItem::new(Geometry::Rect {
                            x: x0,
                            y,
                            width: x1 - x0,
                            height: bandwidth,
                            corner_radius: 0.0,
                        })
                        .with_fill(color)
                        .with_datum(sv.row.clone()),
                    );
                } else {
                    let x = cat_scale.scale(&sv.category).unwrap_or(0.0);
                    let y0 = val_scale.scale(sv.y0);
                    let y1 = val_scale.scale(sv.y1);
                    bar_items.push(
                        MarkItem::new(Geometry::Rect {
                            x,
                            y: y1, // y1 is smaller (higher on screen) for vertical
                            width: bandwidth,
                            height: y0 - y1,
                            corner_radius: 0.0,
                        })
                        .with_fill(color)
                        .with_datum(sv.row.clone()),
                    );
                }
            }

            return build_bar_group(bar_items, &cat_scale, &val_scale, encoding, plot_area, is_horizontal);
        } else {
            // Grouped bars (no stacking)
            let values = extract_numbers(data, val_field);
            let max_value = values.iter().cloned().fold(0.0_f64, f64::max);

            let (cat_scale, val_scale) = if is_horizontal {
                let cat_scale = BandScale::new(unique_categories.clone(), (0.0, plot_area.height)).padding(0.2);
                let val_scale = LinearScale::new((0.0, max_value), (0.0, plot_area.width)).nice().zero();
                (cat_scale, val_scale)
            } else {
                let cat_scale = BandScale::new(unique_categories.clone(), (0.0, plot_area.width)).padding(0.2);
                let val_scale = LinearScale::new((0.0, max_value), (plot_area.height, 0.0)).nice().zero();
                (cat_scale, val_scale)
            };

            let color_values: Vec<String> = extract_categories(data, color_f);
            let unique_colors: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                color_values
                    .iter()
                    .filter(|c| seen.insert((*c).clone()))
                    .cloned()
                    .collect()
            };

            let group_bandwidth = cat_scale.bandwidth();
            let bar_width = group_bandwidth / unique_colors.len() as f64;

            for row in data.iter() {
                let cat = row.get(cat_field).and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    _ => None,
                });
                let val = row.get(val_field).and_then(|v| v.as_f64());
                let color_val = row.get(color_f).and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    _ => None,
                });

                if let (Some(cat), Some(val), Some(cv)) = (cat, val, color_val) {
                    let color_idx = unique_colors.iter().position(|c| c == &cv).unwrap_or(0);
                    let color = Color::from_hex(COLORS[color_idx % COLORS.len()]).unwrap();

                    if is_horizontal {
                        let y = cat_scale.scale(&cat).unwrap_or(0.0) + color_idx as f64 * bar_width;
                        let width = val_scale.scale(val);
                        bar_items.push(
                            MarkItem::new(Geometry::Rect {
                                x: 0.0,
                                y,
                                width,
                                height: bar_width * 0.9,
                                corner_radius: 0.0,
                            })
                            .with_fill(color)
                            .with_datum(row.clone()),
                        );
                    } else {
                        let x = cat_scale.scale(&cat).unwrap_or(0.0) + color_idx as f64 * bar_width;
                        let bar_height = plot_area.height - val_scale.scale(val);
                        bar_items.push(
                            MarkItem::new(Geometry::Rect {
                                x,
                                y: val_scale.scale(val),
                                width: bar_width * 0.9,
                                height: bar_height,
                                corner_radius: 0.0,
                            })
                            .with_fill(color)
                            .with_datum(row.clone()),
                        );
                    }
                }
            }

            return build_bar_group(bar_items, &cat_scale, &val_scale, encoding, plot_area, is_horizontal);
        }
    }

    // Simple bars (no color encoding)
    let values = extract_numbers(data, val_field);
    let max_value = values.iter().cloned().fold(0.0_f64, f64::max);

    let (cat_scale, val_scale) = if is_horizontal {
        let cat_scale = BandScale::new(unique_categories.clone(), (0.0, plot_area.height)).padding(0.2);
        let val_scale = LinearScale::new((0.0, max_value), (0.0, plot_area.width)).nice().zero();
        (cat_scale, val_scale)
    } else {
        let cat_scale = BandScale::new(unique_categories.clone(), (0.0, plot_area.width)).padding(0.2);
        let val_scale = LinearScale::new((0.0, max_value), (plot_area.height, 0.0)).nice().zero();
        (cat_scale, val_scale)
    };

    let default_color = Color::from_hex(COLORS[0]).unwrap();
    let bandwidth = cat_scale.bandwidth();

    for row in data.iter() {
        let cat = row.get(cat_field).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        });
        let val = row.get(val_field).and_then(|v| v.as_f64());

        if let (Some(cat), Some(val)) = (cat, val) {
            if is_horizontal {
                let y = cat_scale.scale(&cat).unwrap_or(0.0);
                let width = val_scale.scale(val);
                bar_items.push(
                    MarkItem::new(Geometry::Rect {
                        x: 0.0,
                        y,
                        width,
                        height: bandwidth,
                        corner_radius: 0.0,
                    })
                    .with_fill(default_color)
                    .with_datum(row.clone()),
                );
            } else {
                let x = cat_scale.scale(&cat).unwrap_or(0.0);
                let bar_height = plot_area.height - val_scale.scale(val);
                bar_items.push(
                    MarkItem::new(Geometry::Rect {
                        x,
                        y: val_scale.scale(val),
                        width: bandwidth,
                        height: bar_height,
                        corner_radius: 0.0,
                    })
                    .with_fill(default_color)
                    .with_datum(row.clone()),
                );
            }
        }
    }

    build_bar_group(bar_items, &cat_scale, &val_scale, encoding, plot_area, is_horizontal)
}

fn build_bar_group(
    bar_items: Vec<MarkItem>,
    cat_scale: &BandScale,
    val_scale: &LinearScale,
    encoding: &Encoding,
    plot_area: &PlotArea,
    is_horizontal: bool,
) -> Result<Group, CompileError> {
    let mut root = Group::new().with_transform(Transform::translate(plot_area.x, plot_area.y));

    // Add bar marks
    root.add_mark(Mark {
        mark_type: MarkType::Rect,
        items: bar_items,
    });

    // Generate axes
    let x_axis_ticks = if is_horizontal {
        val_scale.ticks(5)
    } else {
        cat_scale.ticks()
    };
    let y_axis_ticks = if is_horizontal {
        cat_scale.ticks()
    } else {
        val_scale
            .ticks(5)
            .into_iter()
            .map(|t| crate::scale::Tick {
                value: val_scale.scale(t.value),
                label: t.label,
            })
            .collect()
    };

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
