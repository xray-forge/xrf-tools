// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(3) @binding(0) var<uniform> lighting: Lighting;
@group(3) @binding(1) var<uniform> water: Water;
@group(3) @binding(2) var depth_target: texture_depth_2d;
@group(3) @binding(3) var sky_cube_0: texture_cube<f32>;
@group(3) @binding(4) var sky_cube_1: texture_cube<f32>;
@group(3) @binding(5) var sky_clamp: sampler;
@group(3) @binding(6) var nearest_water: texture_depth_2d;
