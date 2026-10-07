// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var depth_target: texture_depth_2d;
@group(1) @binding(1) var albedo_target: texture_2d<f32>;
@group(1) @binding(2) var normal_target: texture_2d<f32>;
@group(1) @binding(3) var cover: texture_depth_2d;
@group(1) @binding(4) var splash: texture_2d_array<f32>;
@group(1) @binding(5) var flow: texture_2d<f32>;
@group(1) @binding(6) var wet_sampler: sampler;
@group(1) @binding(7) var<uniform> wet: Wet;
