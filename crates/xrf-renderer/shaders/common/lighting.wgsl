// The frame's lighting, as `pass/lighting_uniform.rs` writes it, and the engine's own lighting functions.

struct Lighting {
  // xyz: towards the sun, in view space.
  to_sun: vec4<f32>,
  // rgb: the sun's colour; w: its specular weight.
  sun: vec4<f32>,
  ambient: vec4<f32>,
  environment: vec4<f32>,
  // rgb: what the irradiance cubes return while they are not up; w: one once both are.
  sky_irradiance: vec4<f32>,
  // rgb: the fog's colour; w: one where it fogs.
  fog_color: vec4<f32>,
  // x, y: `fog_params.x` and `.w`; z: one where far geometry fades into the sky's haze above the fold; w: one where the
  // sky is drawn.
  fog: vec4<f32>,
  // rgb: `sky_color`; w: how far from the first keyframe's sky to the second's.
  sky: vec4<f32>,
  // x: the sky's rotation; y: the clouds'; z: the clouds' clock, in seconds.
  sky_params: vec4<f32>,
  // The clouds' colour, then their cover; nothing where they are hidden.
  clouds: vec4<f32>,
  // x: one for Anomaly's shading; y: the rain's density.
  engine: vec4<f32>,
  // y: one where the scene is lit; z: one where the exposure adapts; w: one where the ambient occlusion darkens the
  // hemisphere.
  params: vec4<f32>,
  // `L_ambient` and `L_hemi_color` as a forward pass binds them: the weather's own, neither doubled nor scaled.
  forward_ambient: vec4<f32>,
  forward_hemi: vec4<f32>,
  // rgb: the sun's sprite colour times how far it has faded in; w: half its side as a share of the distance it stands
  // at, zero where none is drawn.
  sun_sprite: vec4<f32>,
  // x: the sun shafts' density; y: the steps along a ray; nothing where they are not drawn.
  shafts: vec4<f32>,
};

// The exposure's state as `frame/exposure.wgsl` adapts it, read from its head.
struct Exposure {
  // The scale the tonemap multiplies by, adapted on the GPU frame by frame.
  adapted: f32,
};

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

// The scaled colour `tonemap` took to a tonemapped one: the curve solved for `x`.
fn untonemap(color: vec3<f32>) -> vec3<f32> {
  let rest: vec3<f32> = 1.0 - color;

  return (sqrt(rest * rest + color * (4.0 / WHITE_INTENSITY_SQUARED)) - rest) * (WHITE_INTENSITY_SQUARED / 2.0);
}

// How much fog lies between the camera and a view space point: none without fog, one where it is total.
fn fog_amount(state: Lighting, position: vec3<f32>) -> f32 {
  return saturate(length(position) * state.fog.y + state.fog.x) * state.fog_color.w;
}
