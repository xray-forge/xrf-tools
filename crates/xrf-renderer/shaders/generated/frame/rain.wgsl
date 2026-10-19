// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<uniform> rain: Rain;
@group(1) @binding(1) var cover: texture_depth_2d;
@group(1) @binding(2) var surface_water: texture_depth_2d;
@group(1) @binding(3) var streak_texture: texture_2d<f32>;
@group(1) @binding(4) var splash_texture: texture_2d<f32>;
@group(1) @binding(5) var rain_sampler: sampler;
@group(1) @binding(6) var<storage, read> splash_vertices: array<vec4<f32>>;
@group(1) @binding(7) var<storage, read> splash_indices: array<u32>;
@group(2) @binding(0) var refracted: texture_2d<f32>;
@group(2) @binding(1) var refracted_sampler: sampler;
@group(2) @binding(2) var drop_map: texture_2d<f32>;
@group(2) @binding(3) var splash_map: texture_2d<f32>;
@group(2) @binding(4) var map_sampler: sampler;
@group(2) @binding(5) var depth_target: texture_depth_2d;
@group(2) @binding(6) var light_target: texture_2d<f32>;
@group(2) @binding(7) var<uniform> lighting: Lighting;
@group(2) @binding(8) var<storage, read> exposure: Exposure;
