// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var depth_target: texture_depth_2d;
@group(1) @binding(1) var albedo_target: texture_2d<f32>;
@group(1) @binding(2) var normal_target: texture_2d<f32>;
@group(1) @binding(3) var material_target: texture_2d<f32>;
@group(1) @binding(4) var material_lut: texture_3d<f32>;
@group(1) @binding(5) var lut_sampler: sampler;
@group(1) @binding(6) var sky_cube_0: texture_cube<f32>;
@group(1) @binding(7) var sky_cube_1: texture_cube<f32>;
@group(1) @binding(8) var sky_environment_0: texture_cube<f32>;
@group(1) @binding(9) var sky_environment_1: texture_cube<f32>;
@group(1) @binding(10) var sky_clouds_0: texture_2d<f32>;
@group(1) @binding(11) var sky_clouds_1: texture_2d<f32>;
@group(1) @binding(12) var sky_clamp: sampler;
@group(1) @binding(13) var sky_repeat: sampler;
@group(1) @binding(14) var<uniform> lighting: Lighting;
@group(1) @binding(15) var<storage, read> exposure: Exposure;
@group(1) @binding(16) var<uniform> settings: Surroundings;
@group(1) @binding(17) var faces: texture_storage_2d_array<rgba16float, write>;
