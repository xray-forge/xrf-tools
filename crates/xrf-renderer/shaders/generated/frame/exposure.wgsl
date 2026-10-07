// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(0) @binding(0) var high: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> state: ExposureState;
@group(0) @binding(2) var<uniform> params: ExposureParams;
