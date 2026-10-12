// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var<storage, read> early: array<u32>;
@group(0) @binding(1) var<storage, read> late: array<u32>;
@group(0) @binding(2) var scene: texture_2d<f32>;
@group(0) @binding(3) var<storage, read_write> dispatch: array<u32>;
