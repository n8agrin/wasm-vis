# Cartesian Visualization Framework - Implementation Plan

## Overview

A Rust/WASM data visualization library using a grammar of graphics approach, targeting Cartesian charts with a simple declarative config format.

## Core Design Principles

1. **Grammar of Graphics foundation** - Marks, encodings, scales, facets as composable primitives
2. **Simple config, rich IR** - Users write minimal YAML/JSON; the system compiles to a full scenegraph
3. **Explicit over implicit** - No magic auto-aggregation; transformations must be declared
4. **Rust core, multi-target** - Single codebase compiles to WASM (browser) and native (server-side PNG/SVG)

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Config (YAML)  │────▶│   Scenegraph    │────▶│    Renderer     │
│  User-facing    │     │   (IR)          │     │  SVG/Canvas/PNG │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Layer 1: Configuration Format

Inspired by ggplot2's elegance but in declarative YAML/JSON:

**Simple case (bar chart):**
```yaml
data:
  values: [{ category: "A", value: 28 }, { category: "B", value: 55 }]

mark: bar
encoding:
  x: { field: category, type: nominal }
  y: { field: value, type: quantitative }
```

**Layered composition (ggplot2-style):**
```yaml
data:
  values: [...]

layer:
  - mark: line
    encoding:
      x: { field: date, type: temporal }
      y: { field: value, type: quantitative }
  - mark: point
    encoding:
      x: { field: date, type: temporal }
      y: { field: value, type: quantitative }
```

**Faceting:**
```yaml
data:
  values: [...]

facet:
  column: { field: region }

spec:
  mark: point
  encoding:
    x: { field: sales, type: quantitative }
    y: { field: profit, type: quantitative }
```

### Layer 2: Scenegraph (IR)

The compiled intermediate representation that renderers consume:

```rust
pub struct Scene {
    pub width: f64,
    pub height: f64,
    pub root: Group,
}

pub struct Group {
    pub transform: Transform,
    pub clip: Option<Rect>,
    pub children: Vec<SceneNode>,
}

pub enum SceneNode {
    Group(Box<Group>),
    Mark(Mark),
}

pub struct Mark {
    pub mark_type: MarkType,
    pub items: Vec<MarkItem>,
}

pub enum MarkType { Rect, Symbol, Line, Area, Rule, Text, Arc, Path }

pub struct MarkItem {
    pub geometry: Geometry,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke>,
    pub opacity: f64,
}
```

### Layer 3: Renderers

Simple trait that backends implement:

```rust
pub trait Renderer {
    type Output;
    fn render(&self, scene: &Scene) -> Self::Output;
}
```

Implementations:
- **SVG** - String output, works everywhere
- **Canvas 2D** - WASM browser rendering
- **tiny-skia** - Native PNG rasterization (server-side)

## Module Structure

```
wasm-vis/
├── Cargo.toml                 # Workspace
├── crates/
│   ├── vis-core/              # Core types (no platform deps)
│   │   └── src/
│   │       ├── spec/          # Config spec types (serde)
│   │       │   ├── mod.rs
│   │       │   ├── mark.rs    # Mark, Encoding definitions
│   │       │   ├── scale.rs   # Scale spec types
│   │       │   └── facet.rs   # Facet spec types
│   │       ├── ir/            # Scenegraph types
│   │       │   ├── mod.rs
│   │       │   ├── scene.rs   # Scene, Group, SceneNode
│   │       │   ├── mark.rs    # Mark, MarkItem, Geometry
│   │       │   └── style.rs   # Color, Stroke, Font
│   │       ├── scale/         # Scale implementations
│   │       │   ├── mod.rs
│   │       │   ├── linear.rs
│   │       │   ├── band.rs
│   │       │   ├── ordinal.rs
│   │       │   └── time.rs
│   │       ├── compile/       # Spec → IR compilation
│   │       │   ├── mod.rs
│   │       │   ├── resolve.rs # Scale inference, domain training
│   │       │   ├── layout.rs  # Axis, legend layout
│   │       │   └── facet.rs   # Facet partitioning
│   │       └── axis.rs        # Axis mark generation
│   │
│   ├── vis-render/            # Renderer implementations
│   │   └── src/
│   │       ├── svg.rs
│   │       └── raster.rs      # tiny-skia PNG (feature-gated)
│   │
│   └── vis-wasm/              # WASM bindings
│       └── src/
│           ├── lib.rs         # wasm-bindgen exports
│           └── canvas.rs      # Canvas 2D renderer
│
└── examples/
    ├── bar.yaml
    ├── scatter.yaml
    └── faceted.yaml
```

## Core Abstractions

### Marks (Visual Primitives)

```rust
pub enum MarkType {
    // Primitives
    Rect,      // bars, heatmap cells
    Symbol,    // scatter points (circle, square, etc.)
    Line,      // line charts
    Area,      // area charts
    Rule,      // reference lines, axis ticks
    Text,      // labels
    Arc,       // pie/donut slices
    Path,      // custom shapes
}

// Composite marks expand to primitives during compilation:
pub enum CompositeMarkType {
    BoxPlot,   // → Rect + Rule + Symbol
    Bullet,    // → Rect + Rule
    Funnel,    // → Rect
}
```

### Encodings (Data → Visual Mapping)

```rust
pub struct Encoding {
    pub x: Option<ChannelDef>,
    pub y: Option<ChannelDef>,
    pub x2: Option<ChannelDef>,
    pub y2: Option<ChannelDef>,
    pub color: Option<ChannelDef>,
    pub fill: Option<ChannelDef>,
    pub stroke: Option<ChannelDef>,
    pub size: Option<ChannelDef>,
    pub opacity: Option<ChannelDef>,
    pub shape: Option<ChannelDef>,
    pub text: Option<ChannelDef>,
}

pub struct ChannelDef {
    pub field: Option<String>,
    pub value: Option<Value>,        // constant value
    pub data_type: DataType,         // nominal, ordinal, quantitative, temporal
    pub scale: Option<String>,       // named scale reference
    pub aggregate: Option<Aggregate>, // explicit aggregation
    pub axis: Option<AxisConfig>,
}
```

### Scales

```rust
pub trait Scale {
    fn domain(&self) -> &Domain;
    fn range(&self) -> &Range;
    fn scale(&self, value: &Value) -> f64;
    fn ticks(&self, count: usize) -> Vec<Tick>;
}

// Implementations
pub struct LinearScale { ... }
pub struct BandScale { ... }    // categorical with width
pub struct OrdinalScale { ... } // categorical colors
pub struct TimeScale { ... }
```

### Faceting

Follows ggplot2's model:
1. Partition data by facet field(s)
2. Replicate inner spec for each partition
3. Train scales across ALL partitions (shared by default)
4. Lay out facet cells in grid

```rust
pub struct FacetSpec {
    pub row: Option<FieldDef>,
    pub column: Option<FieldDef>,
    pub wrap: Option<u32>,        // for single-dimension wrapping
    pub spacing: f64,
    pub resolve: ResolveSpec,     // shared vs independent scales
}
```

### Multiple Axes

Achieved through explicit scale naming (simpler than Vega-Lite's resolve):

```yaml
layer:
  - encoding:
      y:
        field: temperature
        scale: temp_scale
        axis: { orient: left }
  - encoding:
      y:
        field: rainfall
        scale: rain_scale  # different name = different axis
        axis: { orient: right }
```

## Chart Types (V1 Scope)

| Chart Type | Mark(s) | Notes |
|------------|---------|-------|
| Bar (vertical/horizontal) | Rect | Grouped, stacked via encoding |
| Line | Line | Multi-series via color encoding |
| Scatter | Symbol | Shape encoding for categories |
| Area | Area | Stacked via stack transform |
| Box Plot | Rect + Rule + Symbol | Composite, expands in compiler |
| Bullet | Rect + Rule | Composite |
| Funnel | Rect | Trapezoidal styling |

## Implementation Phases

### Phase 1: Foundation
- Spec types with serde parsing (YAML + JSON)
- Core IR types (Scene, Group, Mark, MarkItem)
- LinearScale, BandScale
- SVG renderer
- Bar chart end-to-end

### Phase 2: Core Charts
- Line, Scatter, Area marks
- Symbol shapes
- OrdinalScale (for colors)
- Axis generation with ticks/labels

### Phase 3: Composition
- Layer support
- Basic faceting (column/row)
- Multiple Y-axes via named scales

### Phase 4: WASM + Extended
- wasm-pack build
- Canvas 2D renderer
- Composite marks (BoxPlot, Bullet, Funnel)
- TimeScale

### Phase 5: Polish
- Error messages with suggestions
- PNG export (tiny-skia)
- Performance optimization

## Key Design Decisions

### Why Grammar of Graphics?
- Composability: layers, facets emerge naturally
- Extensibility: new mark types slot in cleanly
- Proven: ggplot2 has validated this approach

### Why Explicit Scale Naming for Multiple Axes?
- Avoids Vega-Lite's confusing `resolve` hierarchy
- Simple mental model: same name = shared, different name = independent

### Why Composite Marks Expand to Primitives?
- Keeps IR simple (only 8 primitive types)
- Renderers stay simple
- BoxPlot config is convenient; BoxPlot IR would be complexity

### Why No Auto-Aggregation?
- Explicit is clearer (user specifies `aggregate: sum`)
- Avoids Vega-Lite's confusion about when aggregation happens
- Data transformation can happen before the viz layer

## Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"

# WASM (vis-wasm only)
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["CanvasRenderingContext2d", "HtmlCanvasElement"] }

# Raster (vis-render, feature-gated)
tiny-skia = { version = "0.11", optional = true }
```

## Design Decisions (Confirmed)

1. **Config format**: JSON-only initially. YAML support can be added trivially later.

2. **Data in IR**: Each `MarkItem` retains its source datum for future interactivity (tooltips, brushing, selections).

3. **Stacking**: Sensible defaults with user override. When a mark has both a positional channel and a color/group channel, stacking is enabled by default for area/bar marks. User can explicitly set `stack: false` or `stack: "normalize"` to override.

```yaml
# Stacking is automatic when color creates groups
mark: bar
encoding:
  x: { field: month }
  y: { field: revenue }
  color: { field: product }  # → stacked by default

# Override with explicit config
mark: bar
encoding:
  x: { field: month }
  y: { field: revenue }
  color: { field: product }
stack: false  # → grouped bars instead
```
