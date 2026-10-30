// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var scene: texture_2d<f32>;
@group(1) @binding(1) var depth_target: texture_depth_2d;
@group(1) @binding(2) var albedo_target: texture_2d<f32>;
@group(1) @binding(3) var normal_target: texture_2d<f32>;
@group(1) @binding(4) var material_target: texture_2d<f32>;
@group(1) @binding(5) var light_target: texture_2d<f32>;
@group(1) @binding(6) var occlusion_target: texture_2d<f32>;
@group(1) @binding(7) var<uniform> present: Present;
@group(1) @binding(8) var upscaled: texture_2d<f32>;
@group(1) @binding(9) var motion_target: texture_2d<f32>;
@group(1) @binding(10) var bloom_target: texture_2d<f32>;
@group(1) @binding(11) var bloom_sampler: sampler;
@group(1) @binding(12) var indirect_light: texture_2d<f32>;
@group(1) @binding(13) var reflections: texture_2d<f32>;
@group(1) @binding(14) var reflection_ends: texture_2d<f32>;
@group(1) @binding(15) var mark_target: texture_2d<f32>;
