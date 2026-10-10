#import "grass/records"

// What the planting passes bind: the level's grass at group 0, and the view's ring, lists and settings at group 1
// (`GrassPass`'s level and build layouts), which the trace's cards read as the cull does.

@group(0) @binding(0) var<storage, read> grid: array<u32>;
@group(0) @binding(1) var<storage, read> slot_records: array<u32>;
@group(0) @binding(2) var<storage, read> bins: array<u32>;
@group(0) @binding(3) var<storage, read> triangles: array<f32>;
@group(0) @binding(4) var<storage, read> dither: array<u32, 256>;
@group(0) @binding(5) var<storage, read> models: array<GrassModel>;
// Items planted a model, then the whole frame's in the last.
@group(0) @binding(6) var<storage, read_write> counts: array<atomic<u32>>;
// Where each model's range goes on filling, as the items are sorted into it.
@group(0) @binding(7) var<storage, read_write> cursors: array<atomic<u32>>;
// One indexed draw a model.
@group(0) @binding(8) var<storage, read_write> args: array<u32>;

// The ring: a key a cell, a vector a cell (its ground's middle height and half its height, its hemisphere and sun), and
// two vectors a cached tuft (its place and turn, then its size, model and wave).
@group(1) @binding(0) var<storage, read_write> keys: array<u32>;
@group(1) @binding(1) var<storage, read_write> shapes: array<vec4<f32>>;
@group(1) @binding(2) var<storage, read_write> cached: array<vec4<f32>>;
// Stale cells a band counts, then the schedule: the band the frame's planting stops in, what it plants there, tickets.
@group(1) @binding(3) var<storage, read_write> band_counts: array<atomic<u32>>;
@group(1) @binding(4) var<storage, read_write> schedule: array<atomic<u32>, 3>;
// The frame's items as planted, each one's model, and the same sorted by model, which the draws read.
@group(1) @binding(5) var<storage, read_write> planted: array<vec4<f32>>;
@group(1) @binding(6) var<storage, read_write> planted_models: array<u32>;
@group(1) @binding(7) var<storage, read_write> sorted: array<vec4<f32>>;
// What the view plants with: its camera, its ring's reach and steps, and the settings.
@group(1) @binding(8) var<uniform> params: GrassParams;
// This frame's puddle sites (`LevelSurface::puddles`: a site's spot in `xy`, its height, its radius or none) and
// where they stand (`pass/grass_puddles_uniform.rs`), which no tuft is planted in: grass does not grow under water.
@group(1) @binding(9) var puddles: texture_2d<f32>;
struct GrassPuddles {
  surface: vec4<f32>,
  shape: vec4<f32>,
  state: vec4<f32>,
};
@group(1) @binding(10) var<uniform> puddle_window: GrassPuddles;
