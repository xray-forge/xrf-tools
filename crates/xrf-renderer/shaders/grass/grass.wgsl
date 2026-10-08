enable wgpu_binding_array;

#import "common/camera"
#import "common/cut_out"
#import "common/foliage_wind"
#import "common/octahedral"
#import "grass/records"

// `deffer_detail_w_flat.vs` and `deffer_detail_s_flat.vs` with `deffer_base_aref_flat.ps`: a model's tufts as the
// planting sorted them, each stood at its place, turned and scaled, a waving one leant by its wave, lit by its slot's sky
// and sun, cut out at its reference, and drawn into the G-buffer from both sides.

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
};

@group(1) @binding(0) var<storage, read> sorted: array<vec4<f32>>;
@group(1) @binding(1) var<storage, read> models: array<GrassModel>;
@group(1) @binding(2) var<uniform> wind: GrassWind;

@group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(2) @binding(1) var texture_sampler: sampler;

// The model drawn: a struct, as the HLSL backend takes immediates only as one.
struct GrassDraw {
  model: u32,
};

var<immediate> draw: GrassDraw;

// What `def_gloss` writes for a surface without a bump.
const DEFAULT_GLOSS: f32 = 2.0 / 255.0;

// How far below its foot a tuft's normals point from: up, and never zero.
const NORMAL_DROP: f32 = 0.75;

// The lighting model's slice of the material table for the default, Blinn: `(1 + 0.5) / 4`.
const MATERIAL_SLICE: f32 = 0.375;

struct GrassVarying {
  @builtin(position) clip: vec4<f32>,
  @location(0) normal: vec3<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) hemi: f32,
  @location(3) sun: f32,
  // Where the vertex stands in the world, and how far it stood from there the frame before.
  @location(4) world: vec3<f32>,
  @location(5) moved: vec3<f32>,
  // Whether the tuft is lit as foliage.
  @location(6) @interpolate(flat) flora: f32,
};

struct GrassOutput {
  @location(0) albedo: vec4<f32>,
  @location(1) normal: vec2<f32>,
  @location(2) material: vec4<f32>,
  @location(3) motion: vec2<f32>,
};

// `calc_cyclic`: a wave from minus one to one over each whole turn, a parabola rather than a sine.
fn cyclic(phase: f32) -> f32 {
  let f: f32 = fract(phase) * 2.8284271 - 1.4142136;

  return f * f - 1.0;
}

// A waving tuft's vertex leant across the ground by its wave's wind, as far as its height over the foot times the wave
// at its place, and as much of that as its own height in the model lets it. The wave runs through the engine's space,
// so the place is read with `z` negated, and the lean carried back the same way.
fn swayed(standing: vec3<f32>, foot: f32, wave: f32, share: f32, is_previous: bool) -> vec3<f32> {
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

@vertex
fn vs_grass(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>,
  @builtin(instance_index) instance_index: u32) -> GrassVarying {
  let place: vec4<f32> = sorted[instance_index * 2u];
  let look: vec4<f32> = sorted[instance_index * 2u + 1u];
  let model: GrassModel = models[draw.model];
  let local: vec3<f32> = position * look.x;
  // Turned about the tuft's up by its yaw, `Fmatrix::rotateY` carried into renderer space.
  let c: f32 = cos(place.w);
  let s: f32 = sin(place.w);
  let standing: vec3<f32> = place.xyz + vec3<f32>(c * local.x - s * local.z, local.y, s * local.x + c * local.z);
  let share: f32 = position.y / max(model.shape.w, 0.0001);
  let current: vec3<f32> = swayed(standing, place.y, look.w, share, false);
  let view: mat3x3<f32> = mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz);
  var out: GrassVarying;

  out.clip = camera.view_projection * vec4<f32>(current, 1.0);
  // From a point below the foot to the vertex: a tuft reads lit from above and a little outward, as the engine lights
  // it.
  out.normal = view * normalize(current - (place.xyz - vec3<f32>(0.0, NORMAL_DROP, 0.0)));
  out.uv = uv;
  out.hemi = look.y;
  out.sun = look.z;
  out.world = current;
  out.moved = swayed(standing, place.y, look.w, share, true) - current;
  out.flora = select(0.0, 1.0, wind.foliage_flora.w > 0.5);

  return out;
}

@fragment
fn fs_grass(in: GrassVarying) -> GrassOutput {
  let model: GrassModel = models[draw.model];
  let dx: vec2<f32> = dpdx(in.uv);
  let dy: vec2<f32> = dpdy(in.uv);
  let base: vec4<f32> = textureSampleGrad(textures[model.texture], texture_sampler, in.uv, dx, dy);

  if (is_alpha_cut(base.a, vec2<f32>(textureDimensions(textures[model.texture])), dx, dy, model.alpha_reference)) {
    discard;
  }

  var out: GrassOutput;

  // White without textures, as grass states no flat colour.
  let is_flora: bool = in.flora > 0.5;
  // Foliage: the tuft's normal leans up, so the sun lights it from above and through it from behind.
  let up: vec3<f32> = (camera.view * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz;
  let normal: vec3<f32> = select(normalize(in.normal), normalize(normalize(in.normal) + up), is_flora);

  out.albedo = vec4<f32>(mix(untextured_color(vec3<f32>(1.0)), base.rgb, camera.switches.x), DEFAULT_GLOSS);
  out.normal = octahedral_encode(normal);
  out.material = vec4<f32>(in.hemi, in.sun, MATERIAL_SLICE, encode_marks(false, false, is_flora, false));
  out.motion = camera_motion(in.world, in.world + in.moved);

  return out;
}
