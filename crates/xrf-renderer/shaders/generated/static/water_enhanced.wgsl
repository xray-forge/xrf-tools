// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(3) @binding(0) var<uniform> lighting: Lighting;
@group(3) @binding(1) var<uniform> water: Water;
@group(3) @binding(2) var<uniform> enhanced: EnhancedWater;
@group(3) @binding(3) var depth_target: texture_depth_2d;
@group(3) @binding(4) var sky_cube_0: texture_cube<f32>;
@group(3) @binding(5) var sky_cube_1: texture_cube<f32>;
@group(3) @binding(6) var sky_clamp: sampler;
@group(3) @binding(7) var nearest_water: texture_depth_2d;
@group(3) @binding(8) var water_scene: texture_2d<f32>;
@group(3) @binding(9) var reflection_blurred: texture_2d<f32>;
@group(3) @binding(10) var reflection_clear: texture_2d<f32>;
@group(3) @binding(11) var perlin_map: texture_2d<f32>;
@group(3) @binding(12) var light_target: texture_2d<f32>;
@group(3) @binding(13) var wave_map: texture_2d<f32>;
@group(3) @binding(14) var wind_map: texture_2d<f32>;
@group(3) @binding(15) var caustics_map: texture_2d<f32>;
@group(3) @binding(16) var height_map: texture_2d<f32>;
@group(3) @binding(17) var ripple_map: texture_2d<f32>;
