use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Encoding channels that map data to visual properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Encoding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x2: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y2: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<ChannelDef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<ChannelDef>,
}

/// Definition of how a channel maps data to visual property
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChannelDef {
    /// Full channel definition
    Full(ChannelDefFull),
    /// Shorthand: just the field name (type will be inferred)
    Field(String),
}

impl ChannelDef {
    pub fn field(&self) -> Option<&str> {
        match self {
            ChannelDef::Full(def) => def.field.as_deref(),
            ChannelDef::Field(f) => Some(f.as_str()),
        }
    }

    pub fn value(&self) -> Option<&Value> {
        match self {
            ChannelDef::Full(def) => def.value.as_ref(),
            ChannelDef::Field(_) => None,
        }
    }

    pub fn data_type(&self) -> Option<DataType> {
        match self {
            ChannelDef::Full(def) => def.data_type,
            ChannelDef::Field(_) => None,
        }
    }

    pub fn aggregate(&self) -> Option<Aggregate> {
        match self {
            ChannelDef::Full(def) => def.aggregate,
            ChannelDef::Field(_) => None,
        }
    }

    pub fn scale_name(&self) -> Option<&str> {
        match self {
            ChannelDef::Full(def) => def.scale.as_deref(),
            ChannelDef::Field(_) => None,
        }
    }

    pub fn axis(&self) -> Option<&AxisConfig> {
        match self {
            ChannelDef::Full(def) => def.axis.as_ref(),
            ChannelDef::Field(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDefFull {
    /// Data field name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// Constant value (alternative to field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    /// Data type: nominal, ordinal, quantitative, temporal
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,
    /// Named scale reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<String>,
    /// Aggregation function (must be explicit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate: Option<Aggregate>,
    /// Axis configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis: Option<AxisConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// Categorical data without order (e.g., countries, categories)
    Nominal,
    /// Categorical data with order (e.g., ratings, sizes)
    Ordinal,
    /// Continuous numeric data
    Quantitative,
    /// Date/time data
    Temporal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Aggregate {
    Count,
    Sum,
    Mean,
    Median,
    Min,
    Max,
    Distinct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orient: Option<AxisOrient>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_count: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AxisOrient {
    Top,
    Bottom,
    Left,
    Right,
}
