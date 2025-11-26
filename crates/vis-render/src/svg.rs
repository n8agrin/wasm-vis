use std::fmt::Write;

use vis_core::ir::{
    Geometry, Group, Mark, MarkItem, MarkType, Scene, SceneNode, SymbolShape, TextAnchor,
    TextBaseline,
};

/// Render a scene to an SVG string
pub fn render_svg(scene: &Scene) -> String {
    let mut svg = String::with_capacity(8192);

    // SVG header
    write!(
        &mut svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        scene.width, scene.height, scene.width, scene.height
    )
    .unwrap();
    svg.push('\n');

    // Background
    if let Some(bg) = &scene.background {
        write!(
            &mut svg,
            r#"  <rect width="100%" height="100%" fill="{}"/>"#,
            bg.to_css()
        )
        .unwrap();
        svg.push('\n');
    }

    // Render root group
    render_group(&mut svg, &scene.root, 1);

    svg.push_str("</svg>\n");
    svg
}

fn render_group(svg: &mut String, group: &Group, indent: usize) {
    let pad = "  ".repeat(indent);

    // Open group
    let has_transform = !group.transform.is_identity();
    let has_clip = group.clip.is_some();

    if has_transform || has_clip {
        write!(svg, "{}<g", pad).unwrap();
        if has_transform {
            write!(svg, r#" transform="{}""#, group.transform.to_svg()).unwrap();
        }
        if let Some(clip) = &group.clip {
            // For simplicity, use inline clip-path
            write!(
                svg,
                r#" clip-path="url(#clip-{}-{})""#,
                clip.x as i32, clip.y as i32
            )
            .unwrap();
        }
        svg.push_str(">\n");
    }

    // Render children
    for child in &group.children {
        match child {
            SceneNode::Group(g) => {
                render_group(svg, g, indent + 1);
            }
            SceneNode::Mark(m) => {
                render_mark(svg, m, indent + 1);
            }
        }
    }

    // Close group
    if has_transform || has_clip {
        write!(svg, "{}</g>\n", pad).unwrap();
    }
}

fn render_mark(svg: &mut String, mark: &Mark, indent: usize) {
    let pad = "  ".repeat(indent);

    // Group for mark (optional, for organization)
    write!(svg, "{}<g class=\"mark-{:?}\">\n", pad, mark.mark_type).unwrap();

    for item in &mark.items {
        render_item(svg, item, &mark.mark_type, indent + 1);
    }

    write!(svg, "{}</g>\n", pad).unwrap();
}

fn render_item(svg: &mut String, item: &MarkItem, _mark_type: &MarkType, indent: usize) {
    let pad = "  ".repeat(indent);

    match &item.geometry {
        Geometry::Rect {
            x,
            y,
            width,
            height,
            corner_radius,
        } => {
            write!(
                svg,
                r#"{}<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}""#,
                pad, x, y, width, height
            )
            .unwrap();
            if *corner_radius > 0.0 {
                write!(svg, r#" rx="{:.2}""#, corner_radius).unwrap();
            }
            write_style(svg, item);
            svg.push_str("/>\n");
        }

        Geometry::Circle { cx, cy, r } => {
            write!(
                svg,
                r#"{}<circle cx="{:.2}" cy="{:.2}" r="{:.2}""#,
                pad, cx, cy, r
            )
            .unwrap();
            write_style(svg, item);
            svg.push_str("/>\n");
        }

        Geometry::Symbol { x, y, size, shape } => {
            if matches!(shape, SymbolShape::Circle) {
                let r = (*size / std::f64::consts::PI).sqrt();
                write!(
                    svg,
                    r#"{}<circle cx="{:.2}" cy="{:.2}" r="{:.2}""#,
                    pad, x, y, r
                )
                .unwrap();
                write_style(svg, item);
                svg.push_str("/>\n");
            } else {
                let path = shape.to_path(*size);
                write!(
                    svg,
                    r#"{}<path d="{}" transform="translate({:.2},{:.2})""#,
                    pad, path, x, y
                )
                .unwrap();
                write_style(svg, item);
                svg.push_str("/>\n");
            }
        }

        Geometry::Line { points } => {
            if points.is_empty() {
                return;
            }
            write!(svg, r#"{}<path d=""#, pad).unwrap();
            for (i, pt) in points.iter().enumerate() {
                if i == 0 {
                    write!(svg, "M{:.2},{:.2}", pt.x, pt.y).unwrap();
                } else {
                    write!(svg, "L{:.2},{:.2}", pt.x, pt.y).unwrap();
                }
            }
            svg.push('"');
            // Lines typically have no fill
            svg.push_str(r#" fill="none""#);
            if let Some(stroke) = &item.stroke {
                write!(
                    svg,
                    r#" stroke="{}" stroke-width="{:.2}""#,
                    stroke.color.to_css(),
                    stroke.width
                )
                .unwrap();
                if let Some(dash) = &stroke.dash {
                    write!(
                        svg,
                        r#" stroke-dasharray="{}""#,
                        dash.iter()
                            .map(|d| format!("{:.2}", d))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                    .unwrap();
                }
            }
            if item.opacity < 1.0 {
                write!(svg, r#" opacity="{:.2}""#, item.opacity).unwrap();
            }
            svg.push_str("/>\n");
        }

        Geometry::Area { points, baseline } => {
            if points.is_empty() {
                return;
            }
            write!(svg, r#"{}<path d=""#, pad).unwrap();
            // Upper line
            for (i, pt) in points.iter().enumerate() {
                if i == 0 {
                    write!(svg, "M{:.2},{:.2}", pt.x, pt.y).unwrap();
                } else {
                    write!(svg, "L{:.2},{:.2}", pt.x, pt.y).unwrap();
                }
            }
            // Lower line (reversed)
            for pt in baseline.iter().rev() {
                write!(svg, "L{:.2},{:.2}", pt.x, pt.y).unwrap();
            }
            svg.push_str("Z\"");
            write_style(svg, item);
            svg.push_str("/>\n");
        }

        Geometry::Rule { x1, y1, x2, y2 } => {
            write!(
                svg,
                r#"{}<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}""#,
                pad, x1, y1, x2, y2
            )
            .unwrap();
            if let Some(stroke) = &item.stroke {
                write!(
                    svg,
                    r#" stroke="{}" stroke-width="{:.2}""#,
                    stroke.color.to_css(),
                    stroke.width
                )
                .unwrap();
            } else if let Some(fill) = &item.fill {
                write!(svg, r#" stroke="{}""#, fill.to_css()).unwrap();
            }
            if item.opacity < 1.0 {
                write!(svg, r#" opacity="{:.2}""#, item.opacity).unwrap();
            }
            svg.push_str("/>\n");
        }

        Geometry::Text {
            x,
            y,
            text,
            font,
            anchor,
            baseline,
            angle,
        } => {
            write!(svg, r#"{}<text x="{:.2}" y="{:.2}""#, pad, x, y).unwrap();

            // Text anchor
            let anchor_str = match anchor {
                TextAnchor::Start => "start",
                TextAnchor::Middle => "middle",
                TextAnchor::End => "end",
            };
            write!(svg, r#" text-anchor="{}""#, anchor_str).unwrap();

            // Dominant baseline
            let baseline_str = match baseline {
                TextBaseline::Top => "hanging",
                TextBaseline::Middle => "middle",
                TextBaseline::Bottom => "ideographic",
                TextBaseline::Alphabetic => "alphabetic",
            };
            write!(svg, r#" dominant-baseline="{}""#, baseline_str).unwrap();

            // Font
            write!(
                svg,
                r#" font-family="{}" font-size="{:.1}""#,
                font.family, font.size
            )
            .unwrap();

            // Rotation
            if *angle != 0.0 {
                write!(svg, r#" transform="rotate({:.1} {:.2} {:.2})""#, angle, x, y).unwrap();
            }

            // Fill (text color)
            if let Some(fill) = &item.fill {
                write!(svg, r#" fill="{}""#, fill.to_css()).unwrap();
            }

            if item.opacity < 1.0 {
                write!(svg, r#" opacity="{:.2}""#, item.opacity).unwrap();
            }

            // Escape text content
            let escaped = escape_xml(text);
            write!(svg, ">{}</text>\n", escaped).unwrap();
        }

        Geometry::Arc {
            cx,
            cy,
            inner_radius,
            outer_radius,
            start_angle,
            end_angle,
        } => {
            // Generate arc path
            let path = arc_path(*cx, *cy, *inner_radius, *outer_radius, *start_angle, *end_angle);
            write!(svg, r#"{}<path d="{}""#, pad, path).unwrap();
            write_style(svg, item);
            svg.push_str("/>\n");
        }

        Geometry::Path { d } => {
            write!(svg, r#"{}<path d="{}""#, pad, d).unwrap();
            write_style(svg, item);
            svg.push_str("/>\n");
        }
    }
}

fn write_style(svg: &mut String, item: &MarkItem) {
    if let Some(fill) = &item.fill {
        write!(svg, r#" fill="{}""#, fill.to_css()).unwrap();
    } else {
        svg.push_str(r#" fill="none""#);
    }
    if let Some(stroke) = &item.stroke {
        write!(
            svg,
            r#" stroke="{}" stroke-width="{:.2}""#,
            stroke.color.to_css(),
            stroke.width
        )
        .unwrap();
        if let Some(dash) = &stroke.dash {
            write!(
                svg,
                r#" stroke-dasharray="{}""#,
                dash.iter()
                    .map(|d| format!("{:.2}", d))
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .unwrap();
        }
    }
    if item.opacity < 1.0 {
        write!(svg, r#" opacity="{:.2}""#, item.opacity).unwrap();
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn arc_path(
    cx: f64,
    cy: f64,
    inner_radius: f64,
    outer_radius: f64,
    start_angle: f64,
    end_angle: f64,
) -> String {
    let cos_start = start_angle.cos();
    let sin_start = start_angle.sin();
    let cos_end = end_angle.cos();
    let sin_end = end_angle.sin();

    let outer_start_x = cx + outer_radius * cos_start;
    let outer_start_y = cy + outer_radius * sin_start;
    let outer_end_x = cx + outer_radius * cos_end;
    let outer_end_y = cy + outer_radius * sin_end;

    let inner_start_x = cx + inner_radius * cos_start;
    let inner_start_y = cy + inner_radius * sin_start;
    let inner_end_x = cx + inner_radius * cos_end;
    let inner_end_y = cy + inner_radius * sin_end;

    let large_arc = if (end_angle - start_angle).abs() > std::f64::consts::PI {
        1
    } else {
        0
    };

    if inner_radius > 0.0 {
        format!(
            "M{:.2},{:.2}A{:.2},{:.2} 0 {} 1 {:.2},{:.2}L{:.2},{:.2}A{:.2},{:.2} 0 {} 0 {:.2},{:.2}Z",
            outer_start_x,
            outer_start_y,
            outer_radius,
            outer_radius,
            large_arc,
            outer_end_x,
            outer_end_y,
            inner_end_x,
            inner_end_y,
            inner_radius,
            inner_radius,
            large_arc,
            inner_start_x,
            inner_start_y
        )
    } else {
        format!(
            "M{:.2},{:.2}A{:.2},{:.2} 0 {} 1 {:.2},{:.2}L{:.2},{:.2}Z",
            outer_start_x,
            outer_start_y,
            outer_radius,
            outer_radius,
            large_arc,
            outer_end_x,
            outer_end_y,
            cx,
            cy
        )
    }
}
