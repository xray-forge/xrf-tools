// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(3) @binding(0) var shadow_atlas: texture_depth_2d;
@group(3) @binding(1) var shadow_sampler: sampler_comparison;
@group(3) @binding(2) var light_visibility_out: texture_storage_2d_array<rgba8unorm, write>;
