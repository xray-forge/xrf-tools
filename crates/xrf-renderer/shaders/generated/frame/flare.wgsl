// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var<uniform> flares: Flares;
@group(1) @binding(1) var<storage, read> state: array<f32, 4>;
@group(1) @binding(2) var flare_sampler: sampler;
@group(2) @binding(0) var flare_texture: texture_2d<f32>;
