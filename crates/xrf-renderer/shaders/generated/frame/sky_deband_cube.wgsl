// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var cube: texture_cube<f32>;
@group(0) @binding(1) var cube_sampler: sampler;
@group(0) @binding(2) var faces: texture_storage_2d_array<rgba16float, write>;
