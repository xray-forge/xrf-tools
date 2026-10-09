// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var albedo_target: texture_2d<f32>;
@group(1) @binding(1) var normal_target: texture_2d<f32>;
@group(1) @binding(2) var material_target: texture_2d<f32>;
@group(1) @binding(3) var depth_target: texture_depth_2d;
@group(1) @binding(4) var light_target: texture_2d<f32>;
@group(1) @binding(5) var occlusion_target: texture_2d<f32>;
@group(1) @binding(6) var wet_surface: texture_2d<f32>;
@group(1) @binding(7) var material_lut: texture_3d<f32>;
@group(1) @binding(8) var lut_sampler: sampler;
@group(1) @binding(9) var<uniform> lighting: Lighting;
@group(1) @binding(10) var<uniform> reflection: Reflections;
@group(1) @binding(11) var nearest_depth: texture_2d<f32>;
@group(1) @binding(12) var blue_noise: texture_2d<f32>;
@group(1) @binding(13) var traced: texture_2d<f32>;
@group(1) @binding(14) var reprojected: texture_2d<f32>;
@group(1) @binding(15) var variance: texture_2d<f32>;
@group(1) @binding(16) var average: texture_2d<f32>;
@group(1) @binding(17) var prefiltered: texture_2d<f32>;
@group(1) @binding(18) var history_radiance: texture_2d<f32>;
@group(1) @binding(19) var history_surface: texture_2d<f32>;
@group(1) @binding(20) var history_held: texture_2d<f32>;
@group(1) @binding(21) var history_sampler: sampler;
