#import "common/lighting"
#import "common/sky_harmonics"

// The colour the fog takes along a view: the weather's `fog_color`, or at the anchored levels the sky as drawn along the
// view, as an engine under an authored sky colours its fog (HDRP's fog in Sky Color mode, Godot's
// `fog_aerial_perspective`), so the distance goes the sky's own blue away from the sun and its glow towards it where the
// weather's colour lay flat; how much of each the view's fog setting picks (`RenderFogColour`). Every pass that fogs
// reads it here by the view's world direction, so the land, the water, the glass and the reflections veil alike.

// The fog's colour along a world direction as a display value, as the weather's `fog_color` stands: turned by the fog
// colour's share (`sky_reflected.w`, `RenderFogColour`) towards the sky as drawn along the view, blurred by its
// harmonics (the cosine lobe `sky_irradiance_at` gives, the anchored level's scale on them taken off), encoded as the
// weather's is. The share is nought but at the anchored levels while the harmonics hold the sky's shape
// (`LightingUniform::sky_reflected`).
fn fog_shown_toward(state: Lighting, direction: vec3<f32>) -> vec3<f32> {
  let share: f32 = state.sky_reflected.w;

  if (share <= 0.0) {
    return state.fog_color.rgb;
  }

  let sky: vec3<f32> = sky_irradiance_at(state, direction) /
    (max(light_scale(state), 1e-12) * max(state.sky_reflected.rgb, vec3<f32>(1e-6)));

  return mix(state.fog_color.rgb, srgb_encoded(sky), share);
}

// The fog's colour along a world direction as light at the light's scale: its display value, decoded where the light is
// physical.
fn fog_colour_light(state: Lighting, direction: vec3<f32>) -> vec3<f32> {
  let shown: vec3<f32> = fog_shown_toward(state, direction);

  if (is_lighting_physical(state)) {
    return srgb_decoded(shown) * light_scale(state);
  }

  return shown * light_scale(state);
}

// The fog's colour along a world direction as the frame shows it: a picture, at the pictures' share of the exposure.
// The passes mixing it into a shown colour take `fog_shown_toward`.
fn fog_light(state: Lighting, direction: vec3<f32>) -> vec3<f32> {
  return fog_colour_light(state, direction) * picture_share(state);
}

// The colour the fog takes towards a view space point at a world height, seen along a world `direction`: its colour
// along the view, taking the sun's where the enhanced fog's height fog lies and the view faces the sun. Mixed as the
// engine mixes it, in display values: under the physical light both are encoded for the mix and the result decoded
// back, as `fog_mixed` blends, else a sun lit at its full linear strength would outweigh the fog's colour several times
// over and veil the frame in it.
fn fog_color_at(state: Lighting, position: vec3<f32>, direction: vec3<f32>, height: f32) -> vec3<f32> {
  let facing: f32 = saturate(dot(state.to_sun.xyz, normalize(position)));
  let share: f32 = facing * height_fog_share(state, height) * state.height_fog.z;
  let fog: vec3<f32> = fog_colour_light(state, direction);

  // The turn is the fog's picture too, shown at the pictures' share as the rest of it.
  return fog_mixed(state, fog, state.sun.rgb, share) * picture_share(state);
}
