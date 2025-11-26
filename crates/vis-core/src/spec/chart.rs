use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::encoding::Encoding;
use crate::ir::Padding;

/// Top-level chart specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSpec {
    /// Chart width in pixels
    #[serde(default = "default_width")]
    pub width: f64,
    /// Chart height in pixels
    #[serde(default = "default_height")]
    pub height: f64,
    /// Padding around the plot area
    #[serde(default = "default_padding")]
    pub padding: Padding,
    /// Background color (CSS color string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// Data source
    pub data: DataSpec,
    /// Mark type for single-layer charts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark: Option<MarkSpec>,
    /// Encoding channels for single-layer charts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    /// Layers for multi-layer charts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<Vec<LayerSpec>>,
    /// Stacking configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<StackConfig>,
    /// Title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

fn default_width() -> f64 {
    600.0
}

fn default_height() -> f64 {
    400.0
}

fn default_padding() -> Padding {
    Padding::new(20.0, 20.0, 40.0, 50.0)
}

/// Data specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataSpec {
    /// Inline data values
    Inline { values: Vec<Value> },
    /// Named dataset reference (for composition)
    Named { name: String },
}

impl DataSpec {
    pub fn values(&self) -> Option<&[Value]> {
        match self {
            DataSpec::Inline { values } => Some(values),
            DataSpec::Named { .. } => None,
        }
    }
}

/// Mark type specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MarkSpec {
    /// Simple mark type
    Simple(MarkType),
    /// Mark with configuration
    WithConfig {
        #[serde(rename = "type")]
        mark_type: MarkType,
        #[serde(flatten)]
        config: MarkConfig,
    },
}

impl MarkSpec {
    pub fn mark_type(&self) -> MarkType {
        match self {
            MarkSpec::Simple(t) => *t,
            MarkSpec::WithConfig { mark_type, .. } => *mark_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkType {
    Bar,
    Line,
    Point,
    Area,
    Rule,
    Text,
    Rect,
    // Composite marks (expand during compilation)
    Boxplot,
    Bullet,
    Funnel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkConfig {
    /// Default fill color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    /// Default stroke color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    /// Default stroke width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    /// Default opacity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// Corner radius for rect/bar marks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f64>,
}

/// Layer specification for multi-layer charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerSpec {
    /// Mark type for this layer
    pub mark: MarkSpec,
    /// Encoding channels
    pub encoding: Encoding,
    /// Optional layer-specific data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<DataSpec>,
}

/// Stacking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StackConfig {
    /// Boolean to enable/disable
    Enabled(bool),
    /// Stacking mode
    Mode(StackMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StackMode {
    /// Stack values
    Zero,
    /// Normalize to 100%
    Normalize,
    /// Center around zero
    Center,
}
