// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(5) @binding(0) var<uniform> lighting: Lighting;
@group(5) @binding(1) var<uniform> water: Water;
@group(5) @binding(2) var<uniform> enhanced: EnhancedWater;
@group(5) @binding(3) var depth_target: texture_depth_2d;
@group(5) @binding(4) var sky_cube_0: texture_cube<f32>;
@group(5) @binding(5) var sky_cube_1: texture_cube<f32>;
@group(5) @binding(6) var sky_clamp: sampler;
@group(5) @binding(7) var nearest_water: texture_depth_2d;
@group(5) @binding(8) var water_scene: texture_2d<f32>;
@group(5) @binding(9) var reflection_history: texture_2d<f32>;
@group(5) @binding(10) var blue_noise: texture_2d<f32>;
@group(5) @binding(11) var material_lut: texture_3d<f32>;
@group(5) @binding(12) var lut_sampler: sampler;
@group(5) @binding(13) var sky_environment_0: texture_cube<f32>;
@group(5) @binding(14) var sky_environment_1: texture_cube<f32>;
@group(5) @binding(15) var<storage, read> exposure: Exposure;
