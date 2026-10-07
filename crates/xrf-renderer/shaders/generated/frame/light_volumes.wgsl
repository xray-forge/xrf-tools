// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<storage, read> light_volumes: array<LightVolume>;
@group(1) @binding(1) var<storage, read> records: array<LightRecord>;
@group(1) @binding(2) var shadow_atlas: texture_depth_2d;
@group(1) @binding(3) var shadow_sampler: sampler_comparison;
@group(1) @binding(4) var<uniform> volumes: LightVolumes;
@group(1) @binding(5) var depth_target: texture_depth_2d;
