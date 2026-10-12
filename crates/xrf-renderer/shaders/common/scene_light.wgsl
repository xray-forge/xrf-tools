// What the scene target holds: tonemapped colour, as the engine's passes draw it, or linear light at the frame's
// exposure, which the window pass tonemaps once (`camera.output.x`). Each pass works in the colour it always has and
// goes through these where it writes the scene or reads it back.

#import "common/camera"
#import "common/lighting"

fn is_scene_linear() -> bool {
  return camera.output.x > 0.5;
}

// Light at `scale` as the scene holds it: tonemapped, or linear at its exposure.
fn scene_from_light(light: vec3<f32>, scale: f32) -> vec3<f32> {
  return select(tonemap(light, scale), light * scale, is_scene_linear());
}

// A colour the engine draws as it is shown (the sky, a sprite, the water) as the scene holds it: as it is, or the light
// at an exposure of one which the window's tonemap shows as that colour.
fn scene_from_shown(shown: vec3<f32>) -> vec3<f32> {
  return select(shown, untonemapped(shown, 1.0), is_scene_linear());
}

// What the scene holds, as the colour it is shown.
fn scene_shown(held: vec3<f32>) -> vec3<f32> {
  return select(held, tonemap(held, 1.0), is_scene_linear());
}

// What the scene holds, as the light it was at `scale`.
fn scene_light(held: vec3<f32>, scale: f32) -> vec3<f32> {
  return select(untonemapped(held, scale), held / max(scale, 1e-4), is_scene_linear());
}
