// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var depth_target: texture_depth_2d;
@group(1) @binding(1) var shadow_maps: texture_depth_2d_array;
@group(1) @binding(2) var<uniform> shadows: Shadows;
@group(1) @binding(3) var<uniform> lighting: Lighting;
@group(1) @binding(4) var<storage, read> exposure: Exposure;
