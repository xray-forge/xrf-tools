// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(6) @binding(0) var<storage, read> water_rays: array<u32>;
@group(6) @binding(1) var<uniform> water_rays_frame: WaterRaysFrame;
@group(6) @binding(2) var water_depth: texture_depth_2d;
