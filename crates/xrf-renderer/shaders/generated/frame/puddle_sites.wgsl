// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var surface: texture_depth_2d;
@group(0) @binding(1) var lowest: texture_storage_2d<r32float, write>;
@group(0) @binding(2) var<uniform> shape: PuddleSites;
@group(0) @binding(3) var heights: texture_2d<f32>;
@group(0) @binding(4) var water: texture_depth_2d;
@group(0) @binding(5) var sites: texture_storage_2d<rgba32float, write>;
@group(0) @binding(6) var kept_sites: texture_2d<f32>;
@group(0) @binding(7) var region_noise: texture_2d<f32>;
@group(0) @binding(8) var noise_sampler: sampler;
@group(0) @binding(9) var<uniform> wet: Wet;
@group(0) @binding(10) var puddles: texture_storage_2d<rgba32float, write>;
