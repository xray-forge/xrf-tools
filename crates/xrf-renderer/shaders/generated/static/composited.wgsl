// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

@group(3) @binding(0) var<uniform> lighting: Lighting;
@group(3) @binding(1) var<storage, read> exposure: Exposure;
@group(3) @binding(2) var material_lut: texture_3d<f32>;
@group(3) @binding(3) var lut_sampler: sampler;
@group(3) @binding(4) var shadow_maps: texture_depth_2d_array;
@group(3) @binding(5) var<uniform> shadows: Shadows;
