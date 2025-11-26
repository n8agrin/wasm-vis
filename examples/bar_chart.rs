use vis_core::chart;
use vis_render::render_svg;

fn main() {
    let spec = r#"
    {
        "width": 600,
        "height": 400,
        "padding": { "top": 20, "right": 20, "bottom": 50, "left": 60 },
        "data": {
            "values": [
                { "category": "A", "value": 28 },
                { "category": "B", "value": 55 },
                { "category": "C", "value": 43 },
                { "category": "D", "value": 91 },
                { "category": "E", "value": 81 },
                { "category": "F", "value": 53 }
            ]
        },
        "mark": "bar",
        "encoding": {
            "x": { "field": "category", "type": "nominal" },
            "y": { "field": "value", "type": "quantitative" }
        }
    }
    "#;

    match chart(spec) {
        Ok(scene) => {
            let svg = render_svg(&scene);
            println!("{}", svg);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
