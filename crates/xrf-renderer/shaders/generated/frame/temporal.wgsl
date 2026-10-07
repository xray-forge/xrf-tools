// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var frame: texture_2d<f32>;
@group(1) @binding(1) var depth_target: texture_depth_2d;
@group(1) @binding(2) var history: texture_2d<f32>;
@group(1) @binding(3) var history_sampler: sampler;
@group(1) @binding(4) var<uniform> temporal: Temporal;
@group(1) @binding(5) var motion_target: texture_2d<f32>;
