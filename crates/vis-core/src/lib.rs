pub mod compile;
pub mod ir;
pub mod scale;
pub mod spec;

pub use compile::compile;
pub use ir::Scene;
pub use spec::ChartSpec;

/// Parse a JSON chart specification
pub fn parse_spec(json: &str) -> Result<ChartSpec, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse and compile a chart specification to a scene graph
pub fn chart(json: &str) -> Result<Scene, Box<dyn std::error::Error>> {
    let spec = parse_spec(json)?;
    let scene = compile(&spec)?;
    Ok(scene)
}
