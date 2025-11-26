# wasm-vis Roadmap

## Completed: Phase 1 - Foundation

- [x] Workspace and crate structure (`vis-core`, `vis-render`)
- [x] IR types: `Scene`, `Group`, `Mark`, `MarkItem`, `Geometry`
- [x] Spec types: `ChartSpec`, `Encoding`, `ChannelDef`, `DataType`
- [x] Scales: `LinearScale`, `BandScale`
- [x] Compiler: spec → IR for bar charts
- [x] SVG renderer
- [x] Bar chart working end-to-end

---

## Phase 2: Core Charts

### Mark Types
- [ ] `Line` mark - connect points with lines
- [ ] `Point`/`Symbol` mark - scatter plots with shapes (circle, square, diamond, etc.)
- [ ] `Area` mark - filled area between line and baseline
- [ ] `Rule` mark - reference lines (already in IR, needs spec support)
- [ ] `Text` mark - labels and annotations

### Scales
- [ ] `OrdinalScale` - categorical → color mapping
- [ ] Color palettes (categorical, sequential, diverging)

### Axis Improvements
- [ ] Grid lines (optional)
- [ ] Axis title from encoding
- [ ] Tick count configuration
- [ ] Label formatting (dates, numbers, custom)

### Files to Create/Modify
- `crates/vis-core/src/compile/line.rs`
- `crates/vis-core/src/compile/point.rs`
- `crates/vis-core/src/compile/area.rs`
- `crates/vis-core/src/scale/ordinal.rs`

---

## Phase 3: Composition

### Layer Support
- [ ] Multiple marks in single chart (`layer` array in spec)
- [ ] Shared scales across layers
- [ ] Independent data per layer

### Faceting
- [ ] `facet.column` - horizontal small multiples
- [ ] `facet.row` - vertical small multiples
- [ ] `facet.wrap` - wrapped grid layout
- [ ] Shared vs independent scales (`resolve` config)
- [ ] Facet labels

### Multiple Axes
- [ ] Named scale references in encoding
- [ ] Secondary Y-axis (right side)
- [ ] Secondary X-axis (top)

### Files to Create/Modify
- `crates/vis-core/src/compile/layer.rs`
- `crates/vis-core/src/compile/facet.rs`
- `crates/vis-core/src/spec/facet.rs` (already stubbed)

---

## Phase 4: WASM + Extended Marks

### WASM Build
- [ ] Create `vis-wasm` crate
- [ ] `wasm-bindgen` exports: `render_svg(spec_json) -> String`
- [ ] `wasm-bindgen` exports: `render_canvas(spec_json, canvas_id)`
- [ ] Canvas 2D renderer implementation
- [ ] npm package setup

### Composite Marks
- [ ] `BoxPlot` - box-and-whisker (expands to Rect + Rule + Symbol)
- [ ] `Bullet` - bullet chart (expands to Rect + Rule)
- [ ] `Funnel` - funnel chart (expands to Rect with trapezoidal shape)

### Time Scale
- [ ] `TimeScale` for temporal data
- [ ] Date parsing
- [ ] Time-aware tick generation (days, months, years)

### Files to Create
- `crates/vis-wasm/Cargo.toml`
- `crates/vis-wasm/src/lib.rs`
- `crates/vis-wasm/src/canvas.rs`
- `crates/vis-core/src/compile/boxplot.rs`
- `crates/vis-core/src/compile/bullet.rs`
- `crates/vis-core/src/compile/funnel.rs`
- `crates/vis-core/src/scale/time.rs`

---

## Phase 5: Polish

### Error Handling
- [ ] Helpful error messages with field suggestions
- [ ] Validation warnings (e.g., missing recommended fields)
- [ ] JSON Schema generation for editor autocomplete

### PNG Export (Server-side)
- [ ] `tiny-skia` integration (feature-gated)
- [ ] `render_png(spec_json) -> Vec<u8>`

### Performance
- [ ] Benchmark suite
- [ ] Scene graph optimization (culling, batching)
- [ ] Large dataset handling (10K+ points)

### Documentation
- [ ] API documentation
- [ ] Example gallery
- [ ] Config format reference

---

## Future Considerations (Post-V1)

### Interactivity
- [ ] Event handling in WASM
- [ ] Tooltips (datum already retained on MarkItem)
- [ ] Brush selection
- [ ] Zoom/pan

### Additional Chart Types
- [ ] Heatmap
- [ ] Histogram (with binning transform)
- [ ] Pie/Donut (Arc mark)
- [ ] Treemap
- [ ] Sankey

### Data Transforms
- [ ] Aggregation (sum, mean, count)
- [ ] Binning
- [ ] Filtering
- [ ] Sorting

### Theming
- [ ] Theme presets (light, dark, publication)
- [ ] Custom theme configuration
