// The frame's lighting, as `pass/lighting_uniform.rs` writes it, and the engine's own lighting functions.

#import "generated/structs"

// The exposure's state as `frame/exposure.wgsl` adapts it, read from its head.
struct Exposure {
  // The scale the tonemap multiplies by, adapted on the GPU frame by frame.
  adapted: f32,
};

// The scale this frame's tonemap multiplies by: the adapted exposure where it adapts, one otherwise.
fn frame_scale(state: Lighting, exposure: Exposure) -> f32 {
  return select(1.0, exposure.adapted, state.params.z > 0.5);
}

// `fWhiteIntensity` of `tonemap`, squared.
const WHITE_INTENSITY_SQUARED: f32 = 1.7 * 1.7;

// `tonemap` of `common_functions.h`: the engine's curve, `x (1 + x / W²) / (1 + x)` over the scaled colour.
fn tonemap(color: vec3<f32>, scale: f32) -> vec3<f32> {
  let x: vec3<f32> = color * scale;

  return x * (1.0 + x / WHITE_INTENSITY_SQUARED) / (1.0 + x);
}

// `def_hdr`: the range the high target holds past the tonemap, which the bloom is built from.
const DEF_HDR: f32 = 9.0;

// `tonemap`'s high part: the scaled colour within `def_hdr`, which an eight-bit target clamps to one.
fn tonemap_high(color: vec3<f32>, scale: f32) -> vec3<f32> {
  return color * scale / DEF_HDR;
}

// How much fog lies between the camera and a view space point: none without fog, one where it is total.
fn fog_amount(state: Lighting, position: vec3<f32>) -> f32 {
  return saturate(length(position) * state.fog.y + state.fog.x) * state.fog_color.w;
}
