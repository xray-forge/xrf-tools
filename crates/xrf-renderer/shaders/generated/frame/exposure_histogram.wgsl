// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var depth_target: texture_depth_2d;
@group(0) @binding(2) var<storage, read_write> state: ExposureState;
@group(0) @binding(3) var<storage, read_write> bins: array<atomic<u32>>;
@group(0) @binding(4) var<uniform> params: ExposureHistogram;
@group(0) @binding(5) var<uniform> lighting: Lighting;
