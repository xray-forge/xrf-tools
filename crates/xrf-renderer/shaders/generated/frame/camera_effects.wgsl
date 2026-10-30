// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var<uniform> camera_effects: CameraEffects;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var beside: texture_2d<f32>;
@group(0) @binding(3) var inputs: texture_2d<f32>;
@group(0) @binding(4) var tiles: texture_2d<f32>;
@group(0) @binding(5) var source_sampler: sampler;
@group(0) @binding(6) var depth_target: texture_depth_2d;
@group(0) @binding(7) var motion_target: texture_2d<f32>;
