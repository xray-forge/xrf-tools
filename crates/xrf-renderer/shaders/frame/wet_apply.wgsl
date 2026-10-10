#import "common/camera"
#import "common/octahedral"
#import "common/fullscreen"
#import "common/wet"

// `rain_apply_normal` and `rain_apply_gloss`: the normal the patch bent written back, and the albedo darkened and the
// gloss raised by how wet it is, wherever anything was drawn.

// The depth, the material marks, the patch (the light target, borrowed before any light is drawn: the normal packed
// into `xy`, what the wetness and, enhanced, a puddle's water add to the gloss into `z` and how wet the engine reads
// the surface into `w`), the enhanced wetting's
// wet surface, how much of a puddle in `g` and the water's depth over the ground in `b`, and the settings.
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

  return textureLoad(patched, texel, 0).xy;
}

// What the albedo is multiplied by for a wetness, darker the wetter, as the engine darkens it.
fn wet_darkening(gloss: f32) -> f32 {
  let darker: f32 = 1.0 - sqrt(gloss);

  return select(max(darker, 0.5), darker + wet.density * 0.5, wet.is_extended > 0.5);
}

// What the albedo is multiplied by, darker the wetter, and the gloss the wetness adds, which the blend adds, as the
// patch worked it out. Anomaly brightens by half the rain what it wets least.
@fragment
fn fs_wet_gloss(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  let wetted: vec4<f32> = textureLoad(patched, texel, 0);

  return vec4<f32>(vec3<f32>(wet_darkening(wetted.w)), wetted.z);
}

// How a puddle darkens the ground it covers, soaked through and under water, darker than the wet ground about it as a
// puddle's floor reads; what its muddy water takes away a metre deep, each way, blue first; and what the silt in it
// gives back in the floor's place, as a share of the ground's own colour, since a puddle's water scatters as much as
// it absorbs: a shallow puddle reads dark neutral grey over its floor, never brown and never black.
const PUDDLE_TINT: vec3<f32> = vec3<f32>(0.32, 0.31, 0.3);
const PUDDLE_ABSORPTION: vec3<f32> = vec3<f32>(1.5, 2.2, 3.5);
const PUDDLE_SILT: vec3<f32> = vec3<f32>(0.45, 0.42, 0.38);

@fragment
fn fs_wet_normal_enhanced(in: FullscreenVarying) -> @location(0) vec2<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  return textureLoad(patched, texel, 0).xy;
}

// The enhanced wet look: the albedo darkened as the engine darkens a wet surface, or under a puddle tinted as its
// soaked floor and taken from by its water down to the ground and back, and the gloss raised by the engine's wetness
// or to the water's, which the reflection's specular weighs.
@fragment
fn fs_wet_gloss_enhanced(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  let wetted: vec4<f32> = textureLoad(patched, texel, 0);
  let surface: vec4<f32> = textureLoad(wet_surface, texel, 0);
  // The light down to the floor and back, and the silt's share in its place.
  let through: vec3<f32> = exp(-PUDDLE_ABSORPTION * 2.0 * surface.b);
  let water: vec3<f32> = mix(PUDDLE_SILT, PUDDLE_TINT * through, through);

  return vec4<f32>(mix(vec3<f32>(wet_darkening(wetted.w)), water, surface.g), wetted.z);
}
