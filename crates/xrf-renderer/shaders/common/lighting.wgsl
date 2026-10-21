// The frame's lighting, as `pass/lighting_uniform.rs` writes it, and the engine's own lighting functions.

#import "generated/structs"
#import "common/luminance"
#import "common/srgb"

// The scale the frame's light is written at per cd/m², so a half float holds it however bright the scene
// (`PreExposure`); one under the engine's light.
fn pre_exposure(state: Lighting) -> f32 {
  return state.pre_exposure.x;
}

// The cd/m² a light of the weather's in the engine's units stands for under the physical light (`LightUnits::weather`);
// one under the engine's, whose light has no units.
fn light_unit(state: Lighting) -> f32 {
  return state.pre_exposure.z;
}

// The scale a light in the engine's units is written at: its luminance at the pre-exposure. The sun, the ambient and
// the lamps arrive at it, and a pass making light of the engine's values (the hemisphere's cubes, the fog's colour, a
// self-lit surface) gives it the same.
fn light_scale(state: Lighting) -> f32 {
  return light_unit(state) * pre_exposure(state);
}

// The scale a local light in the engine's units (a lamp, a self-lit surface) is written at: the local lights' own unit
// (`LightUnits::local`) at the pre-exposure.
fn local_light_scale(state: Lighting) -> f32 {
  return state.pre_exposure.w * pre_exposure(state);
}

// The sun foliage passes through from behind (`SSS`, which the engine's light model returns for the sun's colour to
// multiply): the sun by how directly the eye looks past the leaf into it, times the flora's strength, tinted again by
// the sun's colour as far as the flora's colour share says. Under the physical light that second tint is the sun's hue
// alone, so a leaf passes the light that falls on it and no more: a dim moon's as little as the moon gives. Directions
// in one space, `to_eye` and `to_light` from the leaf.
fn flora_through(state: Lighting, to_eye: vec3<f32>, to_light: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
  let amount: f32 = saturate(dot(to_eye, -(to_light + normal))) * state.flora.y;
  let sun: vec3<f32> = state.sun.rgb;
  let hue: vec3<f32> = sun / max(dot(sun, vec3<f32>(0.2126, 0.7152, 0.0722)), 1e-12);
  let tint: vec3<f32> = select(sun, hue, is_lighting_physical(state));

  return amount * sun * mix(vec3<f32>(1.0), tint, state.flora.z);
}

// The exposure the frame is shown at: the adapted scale where it adapts, one otherwise.
fn frame_exposure(state: Lighting, exposure: Exposure) -> f32 {
  return select(1.0, exposure.adapted, state.params.z > 0.5);
}

// The exposure a picture the engine exposes as it is (the sky's), in the engine's units, is shown at.
fn picture_exposure(state: Lighting, exposure: Exposure) -> f32 {
  return light_unit(state) * frame_exposure(state, exposure);
}

// The scale the frame's light, written at the pre-exposure, is brought to the frame's exposure by.
fn frame_scale(state: Lighting, exposure: Exposure) -> f32 {
  return frame_exposure(state, exposure) / pre_exposure(state);
}

// The scale the last frame's scene, held at the exposure it was shown at, is read back by as light at this frame's
// pre-exposure (`scene_light`).
fn previous_scale(state: Lighting, exposure: Exposure) -> f32 {
  return select(1.0, exposure.used, state.params.z > 0.5) / pre_exposure(state);
}

// The scale a history of the last frame's light is brought to this frame's pre-exposure by.
fn history_scale(state: Lighting) -> f32 {
  return state.pre_exposure.x / state.pre_exposure.y;
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

// The tonemap undone: from the colour a frame showed back to the light it was given at the scale it used, each
// channel `y = x (1 + x / W²) / (1 + x)` solved for `x`.
fn untonemapped(colour: vec3<f32>, scale: f32) -> vec3<f32> {
  let white: f32 = WHITE_INTENSITY_SQUARED;
  let shown: vec3<f32> = max(colour, vec3<f32>(0.0));
  let lit: vec3<f32> = 0.5 * white * ((shown - 1.0) + sqrt((shown - 1.0) * (shown - 1.0) + 4.0 * shown / white));

  return lit / max(scale, 1e-4);
}

// The sky as light for the reflections: its picture taken back through the tonemap, as `untonemapped` takes it, its
// highlights scaled by `gain` at its white, expanded above one (an inverse tone mapping operator, Banterle 2006), since
// X-Ray's sky is a low-dynamic-range picture capped at the tonemap's white where a real overcast sky is far brighter
// than the ground, and dimmed below it. A picture shown encoded for the display (`is_encoded`, the physical light) is
// decoded first.
fn sky_as_light(shown: vec3<f32>, scale: f32, gain: f32, is_encoded: bool) -> vec3<f32> {
  let bright: f32 = smoothstep(0.5, 1.0, luminance(max(shown, vec3<f32>(0.0))));

  return untonemapped(select(shown, srgb_decoded(shown), is_encoded), scale) * mix(1.0, gain, bright);
}

// How much fog lies between the camera and a view space point: none without fog, one where it is total.
fn fog_amount(state: Lighting, position: vec3<f32>) -> f32 {
  return saturate(length(position) * state.fog.y + state.fog.x) * state.fog_color.w;
}

// How much of the enhanced fog's height fog lies at a world height: none from its height up, rising smoothly to whole
// as far below the world's zero; none while it is not drawn.
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

// Whether the light is physical (`RenderLightModel::Physical`): `engine.w`, written from the view's one rule as the
// camera's `output.w` is, for the shared functions that take the lighting rather than read the camera.
fn is_lighting_physical(state: Lighting) -> bool {
  return state.engine.w > 0.5;
}

// The sun's colour as the engine binds it, for a pass mixing it into a shown colour: as it stands, or, where the light
// is physical, its units and pre-exposure taken off and encoded back.
fn sun_shown(state: Lighting) -> vec3<f32> {
  let sun: vec3<f32> = state.sun.rgb / light_scale(state);

  return select(sun, srgb_encoded(sun), is_lighting_physical(state));
}

// The fog's colour as light at the light's scale: the weather's display value, decoded where the light is physical.
// The passes mixing it into a shown colour take `fog_color` as it stands.
fn fog_light(state: Lighting) -> vec3<f32> {
  return select(state.fog_color.rgb, srgb_decoded(state.fog_color.rgb), is_lighting_physical(state)) *
    light_scale(state);
}

// Light fogged towards the fog's light by `amount`: blended as the engine blends it, in display values, so its tuned
// amounts veil as they did; under the physical light both are encoded for the blend, the light's scale taken off first
// and given back after, and the result decoded back.
fn fog_mixed(state: Lighting, light: vec3<f32>, fog: vec3<f32>, amount: f32) -> vec3<f32> {
  let scale: f32 = light_scale(state);
  let encoded: vec3<f32> = srgb_decoded(mix(srgb_encoded(light / scale), srgb_encoded(fog / scale), amount)) * scale;

  return select(mix(light, fog, amount), encoded, is_lighting_physical(state));
}

// The colour the fog takes towards a view space point at a world height: the weather's, taking the sun's colour where
// the enhanced fog's height fog lies and the view faces the sun. Mixed as the engine mixes it, in display values: under
// the physical light both are encoded for the mix and the result decoded back, as `fog_mixed` blends, else a sun lit at
// its full linear strength would outweigh the fog's colour several times over and veil the frame in it.
fn fog_color_at(state: Lighting, position: vec3<f32>, height: f32) -> vec3<f32> {
  let facing: f32 = saturate(dot(state.to_sun.xyz, normalize(position)));
  let share: f32 = facing * height_fog_share(state, height) * state.height_fog.z;
  let fog: vec3<f32> = fog_light(state);

  return fog_mixed(state, fog, state.sun.rgb, share);
}
