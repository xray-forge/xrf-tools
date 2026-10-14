// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var<storage, read> grid: array<u32>;
@group(0) @binding(1) var<storage, read> slot_records: array<u32>;
@group(0) @binding(2) var<storage, read> bins: array<u32>;
@group(0) @binding(3) var<storage, read> triangles: array<f32>;
@group(0) @binding(4) var<storage, read> dither: array<u32, 256>;
@group(0) @binding(5) var<storage, read> models: array<GrassModel>;
@group(0) @binding(6) var<storage, read_write> counts: array<atomic<u32>>;
@group(0) @binding(7) var<storage, read_write> cursors: array<atomic<u32>>;
@group(0) @binding(8) var<storage, read_write> args: array<u32>;
@group(1) @binding(0) var<storage, read_write> keys: array<u32>;
@group(1) @binding(1) var<storage, read_write> shapes: array<vec4<f32>>;
@group(1) @binding(2) var<storage, read_write> cached: array<vec4<f32>>;
@group(1) @binding(3) var<storage, read_write> band_counts: array<atomic<u32>>;
@group(1) @binding(4) var<storage, read_write> schedule: array<atomic<u32>, 3>;
@group(1) @binding(5) var<storage, read_write> planted: array<vec4<f32>>;
@group(1) @binding(6) var<storage, read_write> planted_models: array<u32>;
@group(1) @binding(7) var<storage, read_write> sorted: array<vec4<f32>>;
@group(1) @binding(8) var<uniform> params: GrassParams;
@group(1) @binding(9) var puddles: texture_2d<f32>;
@group(1) @binding(10) var<uniform> puddle_window: GrassPuddles;
