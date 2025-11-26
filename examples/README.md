# wasm-vis Examples

This directory contains example chart configurations and Rust code demonstrating how to use wasm-vis.

## Running Examples

Run any example from the project root:

```bash
# Run and print SVG to stdout
cargo run --example bar_chart

# Save to file
cargo run --example bar_chart > bar_chart.svg

# Run other examples
cargo run --example grouped_bar_chart
cargo run --example stacked_bar_chart
cargo run --example horizontal_bar_chart
cargo run --example aggregated_bar_chart
cargo run --example normalized_stacked_bar_chart
```

## Example Files

Each example consists of two files:

| JSON Config | Rust File | Description |
|-------------|-----------|-------------|
| `bar.json` | `bar_chart.rs` | Simple vertical bar chart |
| `grouped_bar.json` | `grouped_bar_chart.rs` | Multi-series grouped bars |
| `stacked_bar.json` | `stacked_bar_chart.rs` | Stacked bar chart |
| `horizontal_bar.json` | `horizontal_bar_chart.rs` | Horizontal bar chart |
| `aggregated_bar.json` | `aggregated_bar_chart.rs` | Float data with aggregation |
| `normalized_stacked_bar.json` | `normalized_stacked_bar_chart.rs` | 100% stacked (normalized) |

## Adding New Examples

1. Create a JSON configuration file in `examples/`:
   ```json
   {
     "width": 600,
     "height": 400,
     "data": { "values": [...] },
     "mark": "bar",
     "encoding": { ... }
   }
   ```

2. Create a corresponding Rust file in `examples/`:
   ```rust
   use std::fs;
   use vis_core::chart;
   use vis_render::render_svg;

   fn main() {
       let spec = fs::read_to_string("examples/your_chart.json")
           .expect("Failed to read examples/your_chart.json");

       match chart(&spec) {
           Ok(scene) => println!("{}", render_svg(&scene)),
           Err(e) => eprintln!("Error: {}", e),
       }
   }
   ```

3. Register the example in `Cargo.toml`:
   ```toml
   [[example]]
   name = "your_chart"
   path = "examples/your_chart.rs"
   ```

4. Run with: `cargo run --example your_chart`

---

## Supported Configuration Options

### Top-Level Properties

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `width` | number | 600 | Chart width in pixels |
| `height` | number | 400 | Chart height in pixels |
| `padding` | object | `{top: 20, right: 20, bottom: 40, left: 50}` | Chart padding |
| `title` | string | - | Optional chart title |
| `background` | string | - | Background color (CSS color string) |
| `mark` | string | - | Mark type (currently only `"bar"` supported) |
| `data` | object | - | Data specification |
| `encoding` | object | - | Encoding channels |
| `stack` | boolean/string | - | Stacking configuration |

### Padding Object

```json
{
  "padding": {
    "top": 20,
    "right": 20,
    "bottom": 50,
    "left": 60
  }
}
```

### Data Specification

Inline data values:
```json
{
  "data": {
    "values": [
      { "category": "A", "value": 28 },
      { "category": "B", "value": 55 }
    ]
  }
}
```

### Encoding Channels

| Channel | Description |
|---------|-------------|
| `x` | Horizontal position |
| `y` | Vertical position |
| `color` | Color encoding (for grouping) |
| `fill` | Fill color |
| `stroke` | Stroke color |
| `size` | Size encoding |
| `opacity` | Transparency (0.0-1.0) |

#### Channel Definition

Simple form:
```json
{ "x": { "field": "category", "type": "nominal" } }
```

With aggregation:
```json
{ "y": { "field": "value", "type": "quantitative", "aggregate": "mean" } }
```

With axis configuration:
```json
{
  "x": {
    "field": "category",
    "type": "nominal",
    "axis": {
      "title": "Category",
      "grid": true,
      "ticks": true
    }
  }
}
```

### Data Types

| Type | Description | Example |
|------|-------------|---------|
| `nominal` | Unordered categories | Countries, product names |
| `ordinal` | Ordered categories | Size: S, M, L |
| `quantitative` | Continuous numbers | Revenue, temperature |
| `temporal` | Date/time | ISO date strings |

### Aggregation Functions

| Function | Description |
|----------|-------------|
| `count` | Count of records |
| `sum` | Sum of values |
| `mean` | Arithmetic mean |
| `median` | Median value |
| `min` | Minimum value |
| `max` | Maximum value |
| `distinct` | Distinct count |

### Stack Configuration

| Value | Description |
|-------|-------------|
| `false` | No stacking (grouped bars) |
| `true` | Standard stacking from zero |
| `"zero"` | Same as `true` |
| `"normalize"` | Normalize to 100% (0-1 range) |
| `"center"` | Center around zero (diverging) |

### Mark Configuration

Mark can be a string or object:

```json
{ "mark": "bar" }
```

Or with styling:
```json
{
  "mark": {
    "type": "bar",
    "fill": "#ff69b4",
    "stroke": "#333333",
    "strokeWidth": 1,
    "opacity": 0.8,
    "cornerRadius": 4
  }
}
```

### Axis Configuration

```json
{
  "axis": {
    "orient": "bottom",
    "title": "X Axis Title",
    "grid": true,
    "ticks": true,
    "labels": true,
    "tickCount": 10
  }
}
```

| Property | Type | Description |
|----------|------|-------------|
| `orient` | string | `"top"`, `"bottom"`, `"left"`, `"right"` |
| `title` | string | Axis title |
| `grid` | boolean | Show grid lines |
| `ticks` | boolean | Show tick marks |
| `labels` | boolean | Show tick labels |
| `tickCount` | number | Number of ticks |

---

## Currently Supported Mark Types

| Mark | Status |
|------|--------|
| `bar` | Fully supported |
| `line` | Planned |
| `point` | Planned |
| `area` | Planned |
| `rule` | Planned |
| `text` | Planned |

---

## Chart Orientation

Bar charts automatically detect orientation based on encoding:

- **Vertical bars**: `x` = nominal, `y` = quantitative
- **Horizontal bars**: `x` = quantitative, `y` = nominal

---

## Color Palette

Default 10-color palette for grouped/stacked charts:
1. `#ff69b4` (hotpink - default)
2. `#f28e2b`
3. `#e15759`
4. `#76b7b2`
5. `#59a14f`
6. `#edc949`
7. `#af7aa1`
8. `#ff9da7`
9. `#9c755f`
10. `#bab0ab`

Colors cycle if more groups than palette colors.
