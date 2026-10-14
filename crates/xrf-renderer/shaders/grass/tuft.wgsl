#import "common/foliage_wind"
#import "grass/records"

// A tuft as the cull keeps it (`grass/planting`): its place and yaw, then its size, hemisphere, sun, and its wave with
// its ground's normal packed in one float; stood, turned, scaled and swayed as the grass draw stands it, so what the
// trace's cards hold is where the frame draws it.

// How the grass sways this frame, as `pass/grass_wind_uniform.rs` writes it, in the engine's space.
struct GrassWind {
  // `dir1` and `dir2`: each wave's lean across the ground.
  wind_1: vec4<f32>,
  wind_2: vec4<f32>,
  // Each wave's direction, and its phase in `w`, both over a turn.
  wave_1: vec4<f32>,
  wave_2: vec4<f32>,
  // The same four, the frame before, which a tuft's motion is measured from.
  previous_wind_1: vec4<f32>,
  previous_wind_2: vec4<f32>,
  previous_wave_1: vec4<f32>,
  previous_wave_2: vec4<f32>,
  // The enhanced foliage motion's setup and its fields' drift, this frame and the last.
  foliage_wind: vec4<f32>,
  foliage_grass: vec4<f32>,
  foliage_trees: vec4<f32>,
  foliage_anim: vec4<f32>,
  foliage_previous_anim: vec4<f32>,
  foliage_flora: vec4<f32>,
  // x: one where a tuft stands along its ground's normal rather than straight up.
  alignment: vec4<f32>,
};

// `calc_cyclic`: a wave from minus one to one over each whole turn, a parabola rather than a sine.
fn cyclic(phase: f32) -> f32 {
  let f: f32 = fract(phase) * 2.8284271 - 1.4142136;

  return f * f - 1.0;
}

// A waving tuft's vertex leant across the ground by its wave's `wind`, as far as its height over the foot times the
// wave at its place, and as much of that as its own height in the model lets it. The wave runs through the engine's
// space, so the place is read with `z` negated, and the lean carried back the same way.
fn tuft_swayed(wind: GrassWind, standing: vec3<f32>, foot: f32, wave: f32, share: f32, is_previous: bool) -> vec3<f32> {
  if (wave <= 0.5) {
    return standing;
  }

  // The enhanced motion: the flow field's toss, push and lift at the tuft's place, read in the engine's space.
  if (wind.foliage_trees.w > 0.5) {
    let anim: vec4<f32> = select(wind.foliage_anim, wind.foliage_previous_anim, is_previous);
    let setup: FoliageSetup = FoliageSetup(wind.foliage_wind, wind.foliage_grass, wind.foliage_trees, anim);
    let moved: vec3<f32> = foliage_grass(vec2<f32>(standing.x, -standing.z), standing.y - foot, setup);

    return standing + vec3<f32>(moved.x, moved.y, -moved.z);
  }

  let is_second: bool = wave > 1.5;
  let lean_wind: vec4<f32> = select(select(wind.wind_1, wind.wind_2, is_second),
    select(wind.previous_wind_1, wind.previous_wind_2, is_second), is_previous);
  let phase: vec4<f32> = select(select(wind.wave_1, wind.wave_2, is_second),
    select(wind.previous_wave_1, wind.previous_wave_2, is_second), is_previous);
  let engine: vec3<f32> = vec3<f32>(standing.x, standing.y, -standing.z);
  let lean: f32 = (standing.y - foot) * cyclic(dot(engine, phase.xyz) + phase.w) * share;

  return standing + vec3<f32>(lean_wind.x * lean, 0.0, -lean_wind.z * lean);
}

// A ground normal facing up from the whole number its `x` and `z` were kept as, `y` following.
fn unpack_ground_normal(packed: u32) -> vec3<f32> {
  let across: vec2<f32> = vec2<f32>(vec2<u32>(packed / GROUND_NORMAL_STEPS, packed % GROUND_NORMAL_STEPS)) /
    f32(GROUND_NORMAL_STEPS - 1u) * 2.0 - 1.0;

  return vec3<f32>(across.x, sqrt(max(1.0 - dot(across, across), 0.0)), across.y);
}

// A tuft's point turned from standing straight up to standing along `ground`, about the axis across both (Rodrigues).
fn stand_along(point: vec3<f32>, ground: vec3<f32>) -> vec3<f32> {
  let axis: vec3<f32> = vec3<f32>(ground.z, 0.0, -ground.x);
  let sine: f32 = length(axis);

  if (sine < 1e-4) {
    return point;
  }

  let unit: vec3<f32> = axis / sine;

  return point * ground.y + cross(unit, point) * sine + unit * dot(unit, point) * (1.0 - ground.y);
}

// What a tuft's kept vector packs: its wave, nought to two, and the normal of the ground it stands on, straight up
// unless the wind's alignment stands it along its ground.
struct TuftStance {
  waving: f32,
  ground: vec3<f32>,
}

fn tuft_stance(wind: GrassWind, look: vec4<f32>) -> TuftStance {
  let packed: f32 = floor(look.w / 4.0);
  let ground: vec3<f32> = select(vec3<f32>(0.0, 1.0, 0.0), unpack_ground_normal(u32(packed)), wind.alignment.x > 0.5);

  return TuftStance(look.w - packed * 4.0, ground);
}

// A model's vertex at `position` of the tuft at `place` sized `size`, turned about its up by its yaw,
// `Fmatrix::rotateY` carried into renderer space, then stood along its ground; unswayed.
fn tuft_standing(place: vec4<f32>, size: f32, stance: TuftStance, position: vec3<f32>) -> vec3<f32> {
  let local: vec3<f32> = position * size;
  let c: f32 = cos(place.w);
  let s: f32 = sin(place.w);
  let turned: vec3<f32> = vec3<f32>(c * local.x - s * local.z, local.y, s * local.x + c * local.z);

  return place.xyz + stand_along(turned, stance.ground);
}
