// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var scene: texture_2d<f32>;
@group(1) @binding(1) var albedo_target: texture_2d<f32>;
@group(1) @binding(2) var light_target: texture_2d<f32>;
@group(1) @binding(3) var depth_target: texture_depth_2d;
@group(1) @binding(4) var<uniform> lighting: Lighting;
@group(1) @binding(5) var<storage, read> exposure: Exposure;
@group(1) @binding(6) var<storage, read_write> sample: MeterSample;
