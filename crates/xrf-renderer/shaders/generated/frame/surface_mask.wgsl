// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var surface: texture_depth_2d;
@group(0) @binding(1) var lowest: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var<uniform> shape: SurfaceMask;
@group(0) @binding(3) var heights: texture_2d<f32>;
@group(0) @binding(4) var water: texture_depth_2d;
@group(0) @binding(5) var sites: texture_storage_2d<rgba32float, write>;
