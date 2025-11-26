use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::style::{Color, Font, Stroke, TextAnchor, TextBaseline};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkType {
    Rect,
    Symbol,
    Line,
    Area,
    Rule,
    Text,
    Arc,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mark {
    pub mark_type: MarkType,
    pub items: Vec<MarkItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkItem {
    pub geometry: Geometry,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke>,
    pub opacity: f64,
    /// Original datum for interactivity (tooltips, brushing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datum: Option<Value>,
}

impl MarkItem {
    pub fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            fill: None,
            stroke: None,
            opacity: 1.0,
            datum: None,
        }
    }

    pub fn with_fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_datum(mut self, datum: Value) -> Self {
        self.datum = Some(datum);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Geometry {
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        #[serde(default)]
        corner_radius: f64,
    },
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
    },
    Line {
        points: Vec<Point>,
    },
    Area {
        points: Vec<Point>,
        baseline: Vec<Point>,
    },
    Rule {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Text {
        x: f64,
        y: f64,
        text: String,
        font: Font,
        anchor: TextAnchor,
        baseline: TextBaseline,
        #[serde(default)]
        angle: f64,
    },
    Arc {
        cx: f64,
        cy: f64,
        inner_radius: f64,
        outer_radius: f64,
        start_angle: f64,
        end_angle: f64,
    },
    Path {
        d: String,
    },
    Symbol {
        x: f64,
        y: f64,
        size: f64,
        shape: SymbolShape,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SymbolShape {
    #[default]
    Circle,
    Square,
    Cross,
    Diamond,
    Triangle,
    Star,
}

impl SymbolShape {
    /// Generate SVG path for this symbol centered at origin with given size (area)
    pub fn to_path(&self, size: f64) -> String {
        let r = (size / std::f64::consts::PI).sqrt();
        match self {
            SymbolShape::Circle => {
                // Circle is handled specially in renderers
                format!("M{r},0A{r},{r},0,1,1,-{r},0A{r},{r},0,1,1,{r},0")
            }
            SymbolShape::Square => {
                let s = (size).sqrt();
                let h = s / 2.0;
                format!("M-{h},-{h}h{s}v{s}h-{s}Z")
            }
            SymbolShape::Cross => {
                let s = (size / 5.0).sqrt();
                format!(
                    "M-{s3},-{s}h{s2}v-{s2}h{s2}v{s2}h{s2}v{s2}h-{s2}v{s2}h-{s2}v-{s2}h-{s2}Z",
                    s = s,
                    s2 = s * 2.0,
                    s3 = s * 3.0
                )
            }
            SymbolShape::Diamond => {
                let s = (size / 2.0).sqrt();
                format!("M0,-{s}l{s},{s}l-{s},{s}l-{s},-{s}Z")
            }
            SymbolShape::Triangle => {
                let h = (size * 3.0_f64.sqrt()).sqrt();
                let y = h / 3.0;
                format!("M0,-{y2}l{h2},{h}h-{h}Z", y2 = y * 2.0, h = h, h2 = h / 2.0)
            }
            SymbolShape::Star => {
                // 5-pointed star
                let outer = (size / 2.0).sqrt();
                let inner = outer * 0.4;
                let mut d = String::new();
                for i in 0..10 {
                    let angle = std::f64::consts::PI * (i as f64) / 5.0 - std::f64::consts::FRAC_PI_2;
                    let r = if i % 2 == 0 { outer } else { inner };
                    let x = r * angle.cos();
                    let y = r * angle.sin();
                    if i == 0 {
                        d.push_str(&format!("M{x:.3},{y:.3}"));
                    } else {
                        d.push_str(&format!("L{x:.3},{y:.3}"));
                    }
                }
                d.push('Z');
                d
            }
        }
    }
}
