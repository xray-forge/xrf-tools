#import "common/lighting"

// The half box `RenderSky` draws a sky cube through, which every pass reading a sky turns a direction by, and the
// direction every other cube is looked up by; the sky lit through it as the engine draws it, in the frame's own
// units for what reflects it; and the sky as the enhanced water reflects it.

// Where `hbox_verts` puts the ring the box's sides fold at, just under the horizon, and how far below the bottom face
// the box's lower half samples, so all of it reads the cube's bottom rim.
const BOX_HORIZON: f32 = -0.01;
const BOX_BOTTOM: f32 = -1.01;

// A renderer-space direction as the engine looks a cube up by it: in engine space, the `z` the packer negated negated
// back, and never turned. `sky_rotation` turns the sky box's transform alone (`dxEnvironmentRender`'s
// `mSky.rotateY`); water's `s_env0`/`s_env1` and a model's `s_env` sample their cubes by the plain world direction in
// both engines (`water.ps`, Anomaly's `calc_envmap`, `model_env_lq`).
fn cube_lookup(direction: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(direction.x, direction.y, -direction.z);
}

// A renderer-space direction in the sky box's own axes: the cube lookup, turned back by the `rotateY(sky_rotation)` the
// box is drawn with.
fn sky_box_direction(direction: vec3<f32>, rotation: f32) -> vec3<f32> {
  let engine: vec3<f32> = cube_lookup(direction);
  let x: f32 = engine.x;
  let z: f32 = engine.z;
  let c: f32 = cos(rotation);
  let s: f32 = sin(rotation);

  return vec3<f32>(x * c - z * s, direction.y, x * s + z * c);
}

// The cube coordinate `hbox_verts` gives a direction: the top face as it is, each side's whole height folded into the
// sky above the horizon, and everything below it reading the bottom rim, which is the haze a cube is painted with.
fn sky_box_lookup(box: vec3<f32>) -> vec3<f32> {
  let side: f32 = max(abs(box.x), abs(box.z));
  let height: f32 = box.y / side;
  // Each side's two bands, straight across in the face's plane as its vertices interpolate.
  let upper: f32 = -1.0 + (height - BOX_HORIZON) * (2.0 / (1.0 - BOX_HORIZON));
  let lower: f32 = BOX_BOTTOM + (height + 1.0) * ((-1.0 - BOX_BOTTOM) / (1.0 + BOX_HORIZON));
  let on_side: vec3<f32> = vec3<f32>(box.x / side, select(lower, upper, height >= BOX_HORIZON), box.z / side);
  let on_bottom: vec3<f32> = vec3<f32>(box.x / -box.y, BOX_BOTTOM, box.z / -box.y);

  return select(select(on_side, on_bottom, -box.y > side), box, box.y > side);
}

// The sky as the enhanced effects reflect it along a renderer-space direction: turned as its box is, raised to fill a
// reflection's perspective, both keyframes' cubes blended and tinted by the sky's colour.
fn enhanced_sky(
  direction: vec3<f32>,
  lighting: Lighting,
  sky_cube_0: texture_cube<f32>,
  sky_cube_1: texture_cube<f32>,
  sky_clamp: sampler,
) -> vec3<f32> {
  var turned: vec3<f32> = sky_box_direction(direction, lighting.sky_params.x);

  turned.y = (turned.y - max(cos(turned.x) * 0.65, cos(turned.z) * 0.65)) * 2.1 + 0.35;

  let cubes: vec3<f32> = mix(
    textureSampleLevel(sky_cube_0, sky_clamp, turned, 0.0).rgb,
    textureSampleLevel(sky_cube_1, sky_clamp, turned, 0.0).rgb,
    lighting.sky.w,
  );

  return saturate(lighting.sky.rgb) * cubes;
}

// `sky2.vs` and `sky2.ps`: the colour doubled on the tonemap's scale, then the blended cubes a third of it.
const SKY_FACTOR: f32 = 2.0 * 0.33;

// Anomaly's `sky2.vs`: the colour 1.7 times the tonemap's scale, which its `sky2.ps` then takes through the curve.
const CURVED_SKY_FACTOR: f32 = 1.7;

// How high above the fold the rim is still averaged, fading back to single taps by then, as a height over the side.
const HAZE_TOP: f32 = 0.04;

// Taps the rim is averaged over, and the turn between two, two texels of a 512 face: a box over some twenty-four.
const RIM_TAPS: i32 = 13;
const RIM_TAP_ANGLE: f32 = 0.35 * 3.14159265 / 180.0;

// Both skies at an engine cube coordinate, blended as `L_ambient.w` blends them, at their top level: the GPU picks a
// cube's faces as Direct3D does, so the engine's coordinate is sampled as it is.
fn sky_cubes_blended(state: Lighting, cube_0: texture_cube<f32>, cube_1: texture_cube<f32>, cube_sampler: sampler,
  lookup: vec3<f32>) -> vec3<f32> {
  return mix(
    textureSampleLevel(cube_0, cube_sampler, lookup, 0.0).rgb,
    textureSampleLevel(cube_1, cube_sampler, lookup, 0.0).rgb,
    state.sky.w,
  );
}

// The cubes around a box direction under the fold, averaged around the compass so the rim's texels do not show.
fn sky_cubes_rim(state: Lighting, cube_0: texture_cube<f32>, cube_1: texture_cube<f32>, cube_sampler: sampler,
  box: vec3<f32>) -> vec3<f32> {
  var sum: vec3<f32> = vec3<f32>(0.0);

  for (var tap: i32 = 0; tap < RIM_TAPS; tap++) {
    let angle: f32 = (f32(tap) - f32(RIM_TAPS - 1) * 0.5) * RIM_TAP_ANGLE;
    let c: f32 = cos(angle);
    let s: f32 = sin(angle);
    let turned: vec3<f32> = vec3<f32>(box.x * c - box.z * s, box.y, box.x * s + box.z * c);

    sum += sky_cubes_blended(state, cube_0, cube_1, cube_sampler, sky_box_lookup(turned));
  }

  return sum / f32(RIM_TAPS);
}

// How much of the sky along a box direction is the cubes above the fold rather than the rim averaged under it.
fn sky_box_above_fold(box: vec3<f32>) -> f32 {
  return smoothstep(BOX_HORIZON, HAZE_TOP, box.y / max(abs(box.x), abs(box.z)));
}

// The sky lit along a renderer-space direction, before the engine's scale and curve: the cubes through the half box,
// the rim averaged under the fold, times the sky's colour.
fn sky_box_lit(direction: vec3<f32>, state: Lighting, cube_0: texture_cube<f32>, cube_1: texture_cube<f32>,
  cube_sampler: sampler) -> vec3<f32> {
  let box: vec3<f32> = sky_box_direction(direction, state.sky_params.x);
  let above: f32 = sky_box_above_fold(box);
  var color: vec3<f32> = sky_cubes_blended(state, cube_0, cube_1, cube_sampler, sky_box_lookup(box));

  // Below the fold every pixel of a column reads one texel of the bottom rim, which draws its block noise as streaks
  // the height of the band; averaged around the compass there, it keeps the haze and loses the texels.
  if (above < 1.0) {
    color = mix(sky_cubes_rim(state, cube_0, cube_1, cube_sampler, box), color, above);
  }

  return color * state.sky.rgb;
}
