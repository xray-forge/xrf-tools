// What the scene target holds: tonemapped colour, as the engine's passes draw it, or linear light at the frame's
// exposure, which the window pass tonemaps once (`camera.output.x`). Each pass works in the colour it always has and
// goes through these where it writes the scene or reads it back. Under the physical light model (`camera.output.w`)
// the light itself is linear: every colour authored as a display value is decoded before it is lit
// (`authored_colour`), but for the lights, the weather's and the lamps', which keep their strength with their hue
// decoded (`srgb_light`, on the CPU and in the hemisphere), and the window pass encodes the tonemapped frame for the
// display.

#import "common/camera"
#import "common/lighting"
#import "common/srgb"

fn is_scene_linear() -> bool {
  return camera.output.x > 0.5;
}

fn is_light_physical() -> bool {
  return camera.output.w > 0.5;
}

// Whether the physical light stands at the real world's levels (`RenderLightLevel::Physical`) rather than the game's.
fn are_levels_physical() -> bool {
  return camera.output.w > 1.5;
}

// A colour authored as a display value (a texture's, a lamp's, the fog's) as the frame lights with it: decoded by the
// sRGB transfer under the physical light model, as it is stored under the engine's.
fn authored_colour(colour: vec3<f32>) -> vec3<f32> {
  if (is_light_physical()) {
    return srgb_decoded(colour);
  }

  return colour;
}

// An amount of light the engine tunes as a display value (a lamp's falloff, a scattering density) as the share of light
// it sets: decoded under the physical light model, so an amount lit with a decoded light gives the light the engine's
// display value showed.
fn authored_amount(amount: f32) -> f32 {
  if (is_light_physical()) {
    return srgb_decoded(vec3<f32>(amount)).x;
  }

  return amount;
}

// Light at `scale` as the scene holds it: tonemapped, or linear at its exposure.
fn scene_from_light(light: vec3<f32>, scale: f32) -> vec3<f32> {
  return select(tonemap(light, scale), light * scale, is_scene_linear());
}

// Light at `scale` as the colour it is shown as, which `scene_from_shown` takes back: tonemapped, and encoded for the
// display where the light is physical.
fn shown_from_light(light: vec3<f32>, scale: f32) -> vec3<f32> {
  let tonemapped: vec3<f32> = tonemap(light, scale);

  return select(tonemapped, srgb_encoded(tonemapped), is_light_physical());
}

// A colour the engine draws as it is shown (the sky, a sprite, the water) as the scene holds it: as it is, or the light
// at an exposure of one which the window's tonemap shows as that colour, decoded first where the window encodes what
// it shows.
fn scene_from_shown(shown: vec3<f32>) -> vec3<f32> {
  return select(shown, untonemapped(authored_colour(shown), 1.0), is_scene_linear());
}

// What the scene holds, as the colour it is shown.
fn scene_shown(held: vec3<f32>) -> vec3<f32> {
  let tonemapped: vec3<f32> = tonemap(held, 1.0);

  return select(held, select(tonemapped, srgb_encoded(tonemapped), is_light_physical()), is_scene_linear());
}

// What the scene holds, as the light it was at `scale`.
fn scene_light(held: vec3<f32>, scale: f32) -> vec3<f32> {
  return select(untonemapped(held, scale), held / max(scale, 1e-4), is_scene_linear());
}
