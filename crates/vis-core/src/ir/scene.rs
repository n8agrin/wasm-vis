use serde::{Deserialize, Serialize};

use super::mark::Mark;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub width: f64,
    pub height: f64,
    pub background: Option<super::style::Color>,
    pub root: Group,
}

impl Scene {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            background: None,
            root: Group::default(),
        }
    }

    pub fn with_background(mut self, color: super::style::Color) -> Self {
        self.background = Some(color);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Group {
    #[serde(default)]
    pub transform: Transform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip: Option<Rect>,
    #[serde(default)]
    pub children: Vec<SceneNode>,
}

impl Group {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_clip(mut self, clip: Rect) -> Self {
        self.clip = Some(clip);
        self
    }

    pub fn add_child(&mut self, node: SceneNode) {
        self.children.push(node);
    }

    pub fn add_mark(&mut self, mark: Mark) {
        self.children.push(SceneNode::Mark(mark));
    }

    pub fn add_group(&mut self, group: Group) {
        self.children.push(SceneNode::Group(Box::new(group)));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SceneNode {
    Group(Box<Group>),
    Mark(Mark),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    #[serde(default)]
    pub translate_x: f64,
    #[serde(default)]
    pub translate_y: f64,
    #[serde(default = "one")]
    pub scale_x: f64,
    #[serde(default = "one")]
    pub scale_y: f64,
    #[serde(default)]
    pub rotate: f64,
}

fn one() -> f64 {
    1.0
}

impl Transform {
    pub fn translate(x: f64, y: f64) -> Self {
        Self {
            translate_x: x,
            translate_y: y,
            scale_x: 1.0,
            scale_y: 1.0,
            rotate: 0.0,
        }
    }

    pub fn to_svg(&self) -> String {
        let mut parts = Vec::new();
        if self.translate_x != 0.0 || self.translate_y != 0.0 {
            parts.push(format!("translate({},{})", self.translate_x, self.translate_y));
        }
        if self.scale_x != 1.0 || self.scale_y != 1.0 {
            parts.push(format!("scale({},{})", self.scale_x, self.scale_y));
        }
        if self.rotate != 0.0 {
            parts.push(format!("rotate({})", self.rotate));
        }
        parts.join(" ")
    }

    pub fn is_identity(&self) -> bool {
        self.translate_x == 0.0
            && self.translate_y == 0.0
            && self.scale_x == 1.0
            && self.scale_y == 1.0
            && self.rotate == 0.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Padding {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Padding {
    pub const fn uniform(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self { top, right, bottom, left }
    }
}
