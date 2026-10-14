enable wgpu_binding_array;

#import "common/camera"
#import "common/cut_out"
#import "common/octahedral"
#import "grass/records"
#import "grass/tuft"

// `deffer_detail_w_flat.vs` and `deffer_detail_s_flat.vs` with `deffer_base_aref_flat.ps`: a model's tufts as the
// planting sorted them, each stood at its place, turned and scaled, a waving one leant by its wave, lit by its slot's sky
// and sun, cut out at its reference, and drawn into the G-buffer from both sides.

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

@vertex
fn vs_grass(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>,
  @builtin(instance_index) instance_index: u32) -> GrassVarying {
  let place: vec4<f32> = sorted[instance_index * 2u];
  let look: vec4<f32> = sorted[instance_index * 2u + 1u];
  let model: GrassModel = models[draw.model];
  let stance: TuftStance = tuft_stance(wind, look);
  let standing: vec3<f32> = tuft_standing(place, look.x, stance, position);
  let share: f32 = position.y / max(model.shape.w, 0.0001);
  let current: vec3<f32> = tuft_swayed(wind, standing, place.y, stance.waving, share, false);
  let view: mat3x3<f32> = mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz);
  var out: GrassVarying;

  out.clip = camera.view_projection * vec4<f32>(current, 1.0);
  // From a point below the foot to the vertex: a tuft reads lit from above and a little outward, as the engine lights
  // it.
  out.normal = view * normalize(current - (place.xyz - stance.ground * NORMAL_DROP));
  out.uv = uv;
  out.hemi = look.y;
  out.sun = look.z;
  out.world = current;
  out.moved = tuft_swayed(wind, standing, place.y, stance.waving, share, true) - current;
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
  out.material =
    vec4<f32>(in.hemi, in.sun, GRASS_MATERIAL_SLICE, encode_marks(false, false, is_flora, false, false, true));
  out.motion = camera_motion(in.world, in.world + in.moved);

  return out;
}
