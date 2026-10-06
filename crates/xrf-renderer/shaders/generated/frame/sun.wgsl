// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var normal_target: texture_2d<f32>;
@group(1) @binding(1) var material_target: texture_2d<f32>;
@group(1) @binding(2) var depth_target: texture_depth_2d;
@group(1) @binding(3) var material_lut: texture_3d<f32>;
@group(1) @binding(4) var lut_sampler: sampler;
@group(1) @binding(5) var<uniform> lighting: Lighting;
@group(1) @binding(6) var shadow_maps: texture_depth_2d_array;
@group(1) @binding(7) var<uniform> shadows: Shadows;
@group(1) @binding(8) var contact_shadows: texture_2d<f32>;
