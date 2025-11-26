mod svg;

pub use svg::render_svg;

use vis_core::Scene;

/// Trait for rendering a scene to output
pub trait Renderer {
    type Output;
    fn render(&self, scene: &Scene) -> Self::Output;
}
