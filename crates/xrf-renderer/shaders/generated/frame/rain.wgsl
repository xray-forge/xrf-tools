// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<uniform> rain: Rain;
@group(1) @binding(1) var cover: texture_depth_2d;
@group(1) @binding(2) var streak_texture: texture_2d<f32>;
@group(1) @binding(3) var splash_texture: texture_2d<f32>;
@group(1) @binding(4) var rain_sampler: sampler;
@group(1) @binding(5) var<storage, read> splash_vertices: array<vec4<f32>>;
@group(1) @binding(6) var<storage, read> splash_indices: array<u32>;
