#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/wet"

// `rain_apply_normal` and `rain_apply_gloss`: the normal the patch bent written back, and the albedo darkened and the
// gloss raised by how wet it is, wherever anything was drawn.

// The depth, the material marks, the patches (the light target, borrowed before any light is drawn: the engine's the
// patched normal in colour and the wetness in alpha, the enhanced the normal packed into `xy`, the gloss the rain adds
// into `z` and how much of a puddle the point is into `w`), and the settings.
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

// How a puddle tints what it covers, and how much more terrain glosses at full wetness.
const PUDDLE_TINT: vec3<f32> = vec3<f32>(0.66, 0.63, 0.6);
const TERRAIN_WETNESS: f32 = 0.15;

@fragment
fn fs_wet_normal_enhanced(in: FullscreenVarying) -> @location(0) vec2<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  return textureLoad(patched, texel, 0).xy;
}

// The enhanced wet look: the albedo darkened by the rain's gloss and tinted under a puddle, and the gloss raised by
// the rain's, and on terrain by a puddle's, held to the reflectivity, and by the level's wetness.
@fragment
fn fs_wet_gloss_enhanced(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  let wetted: vec4<f32> = textureLoad(patched, texel, 0);
  let is_terrain: bool = has_mark(textureLoad(material_target, texel, 0).a, MARK_TERRAIN);
  let puddle: f32 = min(wetted.w, wet.puddles.z) + saturate(wet.puddles.x * 2.0) * TERRAIN_WETNESS;
  let tint: vec3<f32> = mix(vec3<f32>(1.0), PUDDLE_TINT, wetted.w);
  let rain: f32 = wetted.z * select(0.8, 1.0, wet.is_extended > 0.5);

  return vec4<f32>(tint * wet_darkening(wetted.z), rain + select(0.0, puddle, is_terrain));
}
