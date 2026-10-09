#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/wet"

// `rain_apply_normal` and `rain_apply_gloss`: the normal the patch bent written back, and the albedo darkened and the
// gloss raised by how wet it is, wherever anything was drawn.

// The depth, the material marks, the patches (the light target, borrowed before any light is drawn: the engine's the
// patched normal in colour and the wetness in alpha, the enhanced the normal packed into `xy`, the gloss the rain adds
// into `z` and how deep into a puddle the point is into `w`), the enhanced wetting's wet surface, how much of a puddle
// in `g`, and the settings.
#import "generated/frame/wet_apply"

// Whether anything was drawn at a pixel, reversed: nought where nothing was.
fn is_drawn(texel: vec2<i32>) -> bool {
  return textureLoad(depth_target, texel, 0) > 0.0;
}

@fragment
fn fs_wet_normal(in: FullscreenVarying) -> @location(0) vec2<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  return octahedral_encode(textureLoad(patched, texel, 0).xyz);
}

// What the albedo is multiplied by for a wetness, darker the wetter, as the engine darkens it.
fn wet_darkening(gloss: f32) -> f32 {
  let darker: f32 = 1.0 - sqrt(gloss);

  return select(max(darker, 0.5), darker + wet.density * 0.5, wet.is_extended > 0.5);
}

// What the albedo is multiplied by, darker the wetter, and the gloss the wetness adds, which the blend adds. Anomaly
// brightens by half the rain what it wets least.
@fragment
fn fs_wet_gloss(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  let gloss: f32 = textureLoad(patched, texel, 0).w;

  return vec4<f32>(vec3<f32>(wet_darkening(gloss)), gloss * select(0.8, 1.0, wet.is_extended > 0.5));
}

// How a puddle darkens what it covers, soaked through, at its rim, and what its muddy water takes away a metre deep,
// each way, and how deep its middle is, metres: its water darkens and browns inward, as blue is taken first.
const PUDDLE_TINT: vec3<f32> = vec3<f32>(0.42, 0.4, 0.38);
const PUDDLE_ABSORPTION: vec3<f32> = vec3<f32>(6.0, 9.0, 14.0);
const PUDDLE_DEEPEST: f32 = 0.012;

@fragment
fn fs_wet_normal_enhanced(in: FullscreenVarying) -> @location(0) vec2<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  return textureLoad(patched, texel, 0).xy;
}

// The enhanced wet look: the engine's, the albedo darkened and the gloss raised by the rain's gloss, which `hmodel`'s
// reflection of the environment then weighs; tinted under a puddle, whose clear coat of water combine lays over it.
@fragment
fn fs_wet_gloss_enhanced(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  let wetted: vec4<f32> = textureLoad(patched, texel, 0);
  let puddle: f32 = textureLoad(wet_surface, texel, 0).g;
  let tint: vec3<f32> = mix(vec3<f32>(1.0), PUDDLE_TINT, puddle) * exp(-PUDDLE_ABSORPTION * 2.0 * PUDDLE_DEEPEST * wetted.w);

  return vec4<f32>(tint * wet_darkening(wetted.z), wetted.z * select(0.8, 1.0, wet.is_extended > 0.5));
}
