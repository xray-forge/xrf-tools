// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<uniform> froxels: Froxels;
@group(1) @binding(1) var froxel_scattered: texture_3d<f32>;
@group(1) @binding(2) var froxel_integrated: texture_storage_3d<rgba16float, write>;
