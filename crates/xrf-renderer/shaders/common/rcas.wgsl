// FidelityFX Super Resolution 1's robust contrast-adaptive sharpening (`ffx_fsr1.h`, MIT, `FsrRcasF`), over texels
// as RCAS is made for: display-encoded, or the reversible tonemap `c / (1 + max(c))` of linear light.

// `FSR_RCAS_LIMIT`: the most negative lobe, so the sharpened pixel never rings past its neighbours.
const RCAS_LIMIT: f32 = 0.25 - 1.0 / 16.0;

// `FsrRcasF`: the centre `e` sharpened by a lobe over its four neighbours `b`, `d`, `f`, `h`, as strong as the five
// leave room for within `0..1`, times `sharpness`, so an edge is crisped and a flat area left alone.
fn rcas_sharpen(b: vec3<f32>, d: vec3<f32>, e: vec3<f32>, f: vec3<f32>, h: vec3<f32>, sharpness: f32) -> vec3<f32> {
  let least: vec3<f32> = min(min(b, d), min(f, h));
  let most: vec3<f32> = max(max(b, d), max(f, h));
  // How far each channel may go down before it passes nought, and up before it passes one, the centre counted with
  // its neighbours: a centre outside their span is what sharpening pushes furthest.
  let hit_min: vec3<f32> = min(least, e) / max(most * 4.0, vec3<f32>(1e-5));
  let hit_max: vec3<f32> = (1.0 - max(most, e)) / min(least * 4.0 - 4.0, vec3<f32>(-1e-5));
  let lobes: vec3<f32> = max(-hit_min, hit_max);
  let lobe: f32 = max(-RCAS_LIMIT, min(max(lobes.x, max(lobes.y, lobes.z)), 0.0)) * sharpness;

  return ((b + d + f + h) * lobe + e) / (lobe * 4.0 + 1.0);
}

// `rcas_sharpen` over the reversible tonemap of linear light, held at most halfway from the brightest of its five taps
// to the tonemap's white, so it undoes to at most about twice that tap's light: the limiter lets the most sharpening
// reach white itself, which undoes to no bound.
fn rcas_sharpen_held(
  b: vec3<f32>,
  d: vec3<f32>,
  e: vec3<f32>,
  f: vec3<f32>,
  h: vec3<f32>,
  sharpness: f32
) -> vec3<f32> {
  let brightest: vec3<f32> = max(max(max(b, d), max(f, h)), e);
  let room: f32 = (1.0 + max(brightest.r, max(brightest.g, brightest.b))) * 0.5;

  return clamp(rcas_sharpen(b, d, e, f, h, sharpness), vec3<f32>(0.0), vec3<f32>(room));
}
