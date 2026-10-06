// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<uniform> thunder: Thunder;
@group(1) @binding(1) var thunder_texture: texture_2d<f32>;
@group(1) @binding(2) var thunder_sampler: sampler;
@group(1) @binding(3) var<storage, read> model_vertices: array<vec4<f32>>;
@group(1) @binding(4) var<storage, read> model_indices: array<u32>;
