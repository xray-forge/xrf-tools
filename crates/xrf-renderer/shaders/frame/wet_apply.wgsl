#import "common/octahedral"
#import "common/fullscreen"
#import "common/wet"

// `rain_apply_normal` and `rain_apply_gloss`: the normal the patch bent written back, and the albedo darkened and the
// gloss raised by how wet it is, wherever anything was drawn.

@group(1) @binding(0) var depth_target: texture_depth_2d;
// The patched normal in colour, the wetness in alpha: the light target, borrowed before any light is drawn.
@group(1) @binding(1) var patched: texture_2d<f32>;
@group(1) @binding(2) var<uniform> wet: Wet;

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

// What the albedo is multiplied by, darker the wetter, and the gloss the wetness adds, which the blend adds. Anomaly
// brightens by half the rain what it wets least.
@fragment
fn fs_wet_gloss(in: FullscreenVarying) -> @location(0) vec4<f32> {
  let texel: vec2<i32> = vec2<i32>(in.clip.xy);

  if (!is_drawn(texel)) {
    discard;
  }

  let is_extended: bool = wet.is_extended > 0.5;
  let gloss: f32 = textureLoad(patched, texel, 0).w;
  let darker: f32 = 1.0 - sqrt(gloss);
  let intensity: f32 = select(max(darker, 0.5), darker + wet.density * 0.5, is_extended);

  return vec4<f32>(vec3<f32>(intensity), gloss * select(0.8, 1.0, is_extended));
}
