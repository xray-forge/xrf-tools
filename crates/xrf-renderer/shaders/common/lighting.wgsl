// The frame's lighting, as `pass/lighting_uniform.rs` writes it, and the engine's own lighting functions.

#import "generated/structs"

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

// How much of the enhanced fog's height fog lies at a world height: none from its height up, rising smoothly to whole as
// far below the world's zero; none while it is not drawn.
fn height_fog_share(state: Lighting, height: f32) -> f32 {
  let range: f32 = max(state.height_fog.x, 1e-3);
  let rise: f32 = saturate((range - height) / (2.0 * range));

  return rise * rise * (3.0 - 2.0 * rise) * state.height_fog.w;
}

// How much fog lies between the camera and a view space point standing at a world height: the distance fog, thickened
// by the enhanced fog's density where its height fog lies.
fn fog_amount_at(state: Lighting, position: vec3<f32>, height: f32) -> f32 {
  let fog: f32 = fog_amount(state, position);

  return saturate(fog + height_fog_share(state, height) * fog * state.height_fog.y);
}

// The colour the fog takes towards a view space point at a world height: the weather's, taking the sun's colour where
// the enhanced fog's height fog lies and the view faces the sun.
fn fog_color_at(state: Lighting, position: vec3<f32>, height: f32) -> vec3<f32> {
  let facing: f32 = saturate(dot(state.to_sun.xyz, normalize(position)));
  let sunlit: vec3<f32> = mix(state.fog_color.rgb, state.sun.rgb, facing);

  return mix(state.fog_color.rgb, sunlit, height_fog_share(state, height) * state.height_fog.z);
}
