// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(2) @binding(0) var<uniform> light_trace: LightTrace;
@group(2) @binding(1) var light_visibility_out: texture_storage_2d_array<rgba8unorm, write>;
