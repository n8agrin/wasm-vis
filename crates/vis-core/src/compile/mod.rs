mod bar;
mod line;
mod stack;

use serde_json::Value;
use thiserror::Error;

use crate::ir::{Color, Group, Mark, Scene};
use crate::spec::{AxisOrient, ChartSpec, DataType, Encoding, MarkType, StackConfig};

pub use bar::{compile_bar, COLORS};
pub use line::compile_line;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Unsupported mark type: {0:?}")]
    UnsupportedMark(MarkType),
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),
}

/// Compile a chart specification to a scene graph
pub fn compile(spec: &ChartSpec) -> Result<Scene, CompileError> {
    let mut scene = Scene::new(spec.width, spec.height);

    // Set background
    if let Some(bg) = &spec.background {
        if let Some(color) = Color::from_hex(bg) {
            scene.background = Some(color);
        }
    }

    // Calculate plot area
    let plot_area = PlotArea {
        x: spec.padding.left,
        y: spec.padding.top,
        width: spec.width - spec.padding.left - spec.padding.right,
        height: spec.height - spec.padding.top - spec.padding.bottom,
    };

    // Handle single mark vs layers
    if let Some(mark_spec) = &spec.mark {
        let encoding = spec.encoding.as_ref().ok_or_else(|| {
            CompileError::MissingField("encoding".to_string())
        })?;

        let data = spec
            .data
            .values()
            .ok_or_else(|| CompileError::InvalidData("inline data required".to_string()))?;

        let compiled = compile_mark(mark_spec.mark_type(), encoding, data, &plot_area, spec.stack.as_ref())?;
        scene.root = compiled;
    } else if let Some(_layers) = &spec.layer {
        // TODO: Layer support in Phase 3
        return Err(CompileError::UnsupportedMark(MarkType::Line));
    } else {
        return Err(CompileError::MissingField("mark or layer".to_string()));
    }

    Ok(scene)
}

/// Plot area dimensions
#[derive(Debug, Clone, Copy)]
pub struct PlotArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Compile a single mark type
fn compile_mark(
    mark_type: MarkType,
    encoding: &Encoding,
    data: &[Value],
    plot_area: &PlotArea,
    stack_config: Option<&StackConfig>,
) -> Result<Group, CompileError> {
    match mark_type {
        MarkType::Bar => compile_bar(encoding, data, plot_area, stack_config),
        MarkType::Line => compile_line(encoding, data, plot_area, stack_config),
        MarkType::Point | MarkType::Area | MarkType::Rule | MarkType::Text | MarkType::Rect => {
            Err(CompileError::UnsupportedMark(mark_type))
        }
        MarkType::Boxplot | MarkType::Bullet | MarkType::Funnel => {
            Err(CompileError::UnsupportedMark(mark_type))
        }
    }
}

/// Infer data type from values
pub fn infer_data_type(values: &[Value], field: &str) -> DataType {
    for value in values {
        if let Some(v) = value.get(field) {
            match v {
                Value::Number(_) => return DataType::Quantitative,
                Value::String(s) => {
                    // Check if it's a date-like string
                    if s.contains('-') && s.len() >= 8 {
                        return DataType::Temporal;
                    }
                    return DataType::Nominal;
                }
                _ => continue,
            }
        }
    }
    DataType::Nominal
}

/// Extract field values as strings (for categorical)
pub fn extract_categories(data: &[Value], field: &str) -> Vec<String> {
    data.iter()
        .filter_map(|row| {
            row.get(field).map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => v.to_string(),
            })
        })
        .collect()
}

/// Extract field values as numbers
pub fn extract_numbers(data: &[Value], field: &str) -> Vec<f64> {
    data.iter()
        .filter_map(|row| row.get(field).and_then(|v| v.as_f64()))
        .collect()
}

/// Generate axis marks
pub fn generate_axis(
    orient: AxisOrient,
    ticks: &[crate::scale::Tick],
    plot_area: &PlotArea,
    title: Option<&str>,
) -> Vec<Mark> {
    use crate::ir::{
        Font, Geometry, MarkItem, MarkType as IrMarkType, Stroke, TextAnchor, TextBaseline,
    };

    let mut marks = Vec::new();
    let axis_color = Color::rgb(100, 100, 100);
    let tick_length = 6.0;
    let label_offset = 10.0;

    // Axis line
    let line_item = match orient {
        AxisOrient::Bottom => MarkItem::new(Geometry::Rule {
            x1: 0.0,
            y1: plot_area.height,
            x2: plot_area.width,
            y2: plot_area.height,
        }),
        AxisOrient::Left => MarkItem::new(Geometry::Rule {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: plot_area.height,
        }),
        AxisOrient::Top => MarkItem::new(Geometry::Rule {
            x1: 0.0,
            y1: 0.0,
            x2: plot_area.width,
            y2: 0.0,
        }),
        AxisOrient::Right => MarkItem::new(Geometry::Rule {
            x1: plot_area.width,
            y1: 0.0,
            x2: plot_area.width,
            y2: plot_area.height,
        }),
    }
    .with_stroke(Stroke::solid(axis_color, 1.0));

    marks.push(Mark {
        mark_type: IrMarkType::Rule,
        items: vec![line_item],
    });

    // Tick marks and labels
    let mut tick_items = Vec::new();
    let mut label_items = Vec::new();

    for tick in ticks {
        let (tx1, ty1, tx2, ty2, lx, ly, anchor, baseline) = match orient {
            AxisOrient::Bottom => (
                tick.value,
                plot_area.height,
                tick.value,
                plot_area.height + tick_length,
                tick.value,
                plot_area.height + tick_length + label_offset,
                TextAnchor::Middle,
                TextBaseline::Top,
            ),
            AxisOrient::Left => (
                0.0,
                tick.value,
                -tick_length,
                tick.value,
                -tick_length - label_offset,
                tick.value,
                TextAnchor::End,
                TextBaseline::Middle,
            ),
            AxisOrient::Top => (
                tick.value,
                0.0,
                tick.value,
                -tick_length,
                tick.value,
                -tick_length - label_offset,
                TextAnchor::Middle,
                TextBaseline::Bottom,
            ),
            AxisOrient::Right => (
                plot_area.width,
                tick.value,
                plot_area.width + tick_length,
                tick.value,
                plot_area.width + tick_length + label_offset,
                tick.value,
                TextAnchor::Start,
                TextBaseline::Middle,
            ),
        };

        tick_items.push(
            MarkItem::new(Geometry::Rule {
                x1: tx1,
                y1: ty1,
                x2: tx2,
                y2: ty2,
            })
            .with_stroke(Stroke::solid(axis_color, 1.0)),
        );

        label_items.push(MarkItem::new(Geometry::Text {
            x: lx,
            y: ly,
            text: tick.label.clone(),
            font: Font::default(),
            anchor,
            baseline,
            angle: 0.0,
        }).with_fill(axis_color));
    }

    marks.push(Mark {
        mark_type: IrMarkType::Rule,
        items: tick_items,
    });

    marks.push(Mark {
        mark_type: IrMarkType::Text,
        items: label_items,
    });

    // Title
    if let Some(title_text) = title {
        let (tx, ty, anchor, baseline, angle) = match orient {
            AxisOrient::Bottom => (
                plot_area.width / 2.0,
                plot_area.height + 35.0,
                TextAnchor::Middle,
                TextBaseline::Top,
                0.0,
            ),
            AxisOrient::Left => (
                -40.0,
                plot_area.height / 2.0,
                TextAnchor::Middle,
                TextBaseline::Bottom,
                -90.0,
            ),
            AxisOrient::Top => (
                plot_area.width / 2.0,
                -35.0,
                TextAnchor::Middle,
                TextBaseline::Bottom,
                0.0,
            ),
            AxisOrient::Right => (
                plot_area.width + 40.0,
                plot_area.height / 2.0,
                TextAnchor::Middle,
                TextBaseline::Top,
                90.0,
            ),
        };

        let mut font = Font::default();
        font.size = 14.0;

        marks.push(Mark {
            mark_type: IrMarkType::Text,
            items: vec![MarkItem::new(Geometry::Text {
                x: tx,
                y: ty,
                text: title_text.to_string(),
                font,
                anchor,
                baseline,
                angle,
            }).with_fill(Color::rgb(50, 50, 50))],
        });
    }

    marks
}
