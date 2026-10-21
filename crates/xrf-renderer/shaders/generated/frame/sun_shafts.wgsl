// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var depth_target: texture_depth_2d;
@group(1) @binding(1) var shadow_maps: texture_depth_2d_array;
@group(1) @binding(2) var<uniform> shadows: Shadows;
@group(1) @binding(3) var<uniform> lighting: Lighting;
@group(1) @binding(4) var<storage, read> exposure: Exposure;
@group(1) @binding(5) var shafts: texture_2d<f32>;
@group(1) @binding(6) var light_volumes: texture_2d<f32>;
@group(1) @binding(7) var scene_before: texture_2d<f32>;
@group(1) @binding(8) var<uniform> froxels: Froxels;
@group(1) @binding(9) var froxel_integrated: texture_3d<f32>;
@group(1) @binding(10) var froxel_sampler: sampler;
