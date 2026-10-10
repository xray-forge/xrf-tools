// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var sky_cube_0: texture_cube<f32>;
@group(0) @binding(1) var sky_cube_1: texture_cube<f32>;
@group(0) @binding(2) var sky_clouds_0: texture_2d<f32>;
@group(0) @binding(3) var sky_clouds_1: texture_2d<f32>;
@group(0) @binding(4) var sky_clamp: sampler;
@group(0) @binding(5) var sky_repeat: sampler;
@group(0) @binding(6) var source: texture_cube<f32>;
@group(0) @binding(7) var<uniform> lighting: Lighting;
@group(0) @binding(8) var<storage, read> exposure: Exposure;
@group(0) @binding(9) var<uniform> settings: SkyPrefilter;
@group(0) @binding(10) var faces: texture_storage_2d_array<rgba16float, write>;
