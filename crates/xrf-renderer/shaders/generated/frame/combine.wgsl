// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var albedo_target: texture_2d<f32>;
@group(1) @binding(1) var normal_target: texture_2d<f32>;
@group(1) @binding(2) var material_target: texture_2d<f32>;
@group(1) @binding(3) var depth_target: texture_depth_2d;
@group(1) @binding(4) var light_target: texture_2d<f32>;
@group(1) @binding(5) var material_lut: texture_3d<f32>;
@group(1) @binding(6) var lut_sampler: sampler;
@group(1) @binding(7) var<uniform> lighting: Lighting;
@group(1) @binding(8) var<storage, read> exposure: Exposure;
@group(1) @binding(9) var occlusion_target: texture_2d<f32>;
@group(1) @binding(10) var haze_map: texture_2d<f32>;
