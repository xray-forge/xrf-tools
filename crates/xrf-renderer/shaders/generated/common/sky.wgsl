// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

#import "generated/structs"

enable wgpu_binding_array;
@group(2) @binding(0) var sky_cube_0: texture_cube<f32>;
@group(2) @binding(1) var sky_cube_1: texture_cube<f32>;
@group(2) @binding(2) var sky_environment_0: texture_cube<f32>;
@group(2) @binding(3) var sky_environment_1: texture_cube<f32>;
@group(2) @binding(4) var sky_clouds_0: texture_2d<f32>;
@group(2) @binding(5) var sky_clouds_1: texture_2d<f32>;
@group(2) @binding(6) var sky_clamp: sampler;
@group(2) @binding(7) var sky_repeat: sampler;
@group(2) @binding(8) var environments: binding_array<texture_cube<f32>, 16>;
@group(2) @binding(9) var sky_sun: texture_2d<f32>;
