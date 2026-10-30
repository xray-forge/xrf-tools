// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var depth_target: texture_depth_2d;
@group(0) @binding(1) var albedo_target: texture_2d<f32>;
@group(0) @binding(2) var normal_target: texture_2d<f32>;
@group(0) @binding(3) var material_target: texture_2d<f32>;
@group(0) @binding(4) var colour_level: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(5) var normal_level: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(6) var material_level: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(7) var colour_source: texture_2d<f32>;
@group(0) @binding(8) var normal_source: texture_2d<f32>;
@group(0) @binding(9) var material_source: texture_2d<f32>;
@group(0) @binding(10) var<storage, read_write> blocks: array<vec4<u32>>;
@group(0) @binding(11) var<uniform> settings: ImpostorEncode;
