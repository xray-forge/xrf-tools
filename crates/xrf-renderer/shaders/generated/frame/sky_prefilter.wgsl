// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var sky_cube_0: texture_cube<f32>;
@group(0) @binding(1) var sky_cube_1: texture_cube<f32>;
@group(0) @binding(2) var sky_clamp: sampler;
@group(0) @binding(3) var source: texture_cube<f32>;
@group(0) @binding(4) var<uniform> lighting: Lighting;
@group(0) @binding(5) var<uniform> settings: SkyPrefilter;
@group(0) @binding(6) var faces: texture_storage_2d_array<rgba16float, write>;
