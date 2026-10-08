// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(1) @binding(0) var source: texture_2d<f32>;
@group(1) @binding(1) var blurred: texture_2d<f32>;
@group(1) @binding(2) var source_sampler: sampler;
@group(1) @binding(3) var depth_target: texture_depth_2d;
@group(1) @binding(4) var<uniform> lighting: Lighting;
@group(1) @binding(5) var<uniform> scattering: FogScattering;
