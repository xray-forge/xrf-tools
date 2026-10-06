// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var normal_target: texture_2d<f32>;
@group(1) @binding(1) var depth_target: texture_depth_2d;
@group(1) @binding(2) var motion_target: texture_2d<f32>;
@group(1) @binding(3) var<uniform> occlusion: Vbao;
@group(1) @binding(4) var source: texture_2d<f32>;
@group(1) @binding(5) var history: texture_2d<f32>;
@group(1) @binding(6) var albedo_target: texture_2d<f32>;
@group(1) @binding(7) var material_target: texture_2d<f32>;
@group(1) @binding(8) var light_target: texture_2d<f32>;
@group(1) @binding(9) var light_source: texture_2d<f32>;
@group(1) @binding(10) var gathered: texture_2d<f32>;
@group(1) @binding(11) var light_history: texture_2d<f32>;
