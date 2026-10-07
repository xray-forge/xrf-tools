// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var normal_target: texture_2d<f32>;
@group(1) @binding(1) var depth_target: texture_depth_2d;
@group(1) @binding(2) var<uniform> occlusion: AmbientOcclusion;
@group(1) @binding(3) var source: texture_2d<f32>;
