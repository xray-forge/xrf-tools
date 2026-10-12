// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<uniform> lighting: Lighting;
@group(1) @binding(1) var shadow_maps: texture_depth_2d_array;
@group(1) @binding(2) var<uniform> shadows: Shadows;
@group(1) @binding(3) var<storage, read> records: array<LightRecord>;
@group(1) @binding(4) var<storage, read> counts: array<u32>;
@group(1) @binding(5) var<storage, read> items: array<u32>;
@group(1) @binding(6) var<uniform> lights: Lights;
@group(1) @binding(7) var shadow_atlas: texture_depth_2d;
@group(1) @binding(8) var<uniform> froxels: Froxels;
@group(1) @binding(9) var froxel_history: texture_3d<f32>;
@group(1) @binding(10) var froxel_sampler: sampler;
@group(1) @binding(11) var froxel_current: texture_storage_3d<rgba16float, write>;
