// Generated from Rust declarations; do not edit. Regenerate with `cargo test`.

struct Lighting {
  to_sun: vec4<f32>,
  sun: vec4<f32>,
  ambient: vec4<f32>,
  environment: vec4<f32>,
  sky_irradiance: vec4<f32>,
  fog_color: vec4<f32>,
  fog: vec4<f32>,
  sky: vec4<f32>,
  sky_params: vec4<f32>,
  clouds: vec4<f32>,
  engine: vec4<f32>,
  params: vec4<f32>,
  forward_ambient: vec4<f32>,
  forward_hemi: vec4<f32>,
  forward_sun: vec4<f32>,
  sun_sprite: vec4<f32>,
  shafts: vec4<f32>,
}

struct Water {
  time: f32,
  wave_height: f32,
  wave_speed: f32,
  ripple: f32,
  reflection: f32,
  intensity: f32,
  soft: f32,
  distorted: f32,
  refraction: f32,
  turbidity: f32,
  soft_border: f32,
  reflectivity: f32,
  reflection_blur: f32,
  blur_noise: f32,
  reflected: f32,
  history: f32,
  wind_velocity: f32,
  specular: f32,
  caustics: f32,
  parallax_height: f32,
  ripples: f32,
  rain: f32,
  flowed: f32,
  waves: f32,
  heights: f32,
  gusts_x: f32,
  gusts_y: f32,
  variation: f32,
}

struct WaterBlur {
  direction: vec4<f32>,
}

struct Cluster {
  first_index: u32,
  triangles: u32,
  vertex_start: u32,
  slot: u32,
}

struct Slot {
  first_cluster: u32,
  cluster_count: u32,
  place: u32,
  kind: u32,
  batch: u32,
  surface: u32,
  pad: array<u32, 2>,
}

struct Place {
  transform: mat4x4<f32>,
  info: vec4<f32>,
  cube: vec4<u32>,
  skin: vec4<u32>,
}

struct Row {
  sphere: vec4<f32>,
  place: u32,
  slot: u32,
  lod: u32,
  band: u32,
}

struct Region {
  base: u32,
  capacity: u32,
}

struct CullParams {
  cluster_count: u32,
  row_count: u32,
  batch_count: u32,
  impostor_count: u32,
  glod_start: f32,
  glod_end: f32,
  discard_below: f32,
  candidate_capacity: u32,
  is_occluding: u32,
  lod_a: f32,
  lod_b: f32,
  is_impostors: u32,
  pad: vec4<u32>,
  lod_origin: vec4<f32>,
}

struct Occlusion {
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
  size: vec2<f32>,
  levels: u32,
  has_history: u32,
}

struct Impostor {
  sphere: vec4<f32>,
  normals: array<vec4<f32>, 8>,
  factor: f32,
  surface: u32,
}

struct TerrainSlots {
  details: vec4<u32>,
  bumps: vec4<u32>,
  mask: u32,
}

struct Surface {
  tiling: f32,
  detail_scale: f32,
  alpha_reference: f32,
  slice: f32,
  color: vec3<f32>,
  flags: u32,
  textures: array<u32, 8>,
  terrain: TerrainSlots,
}

struct Wind {
  wind: vec4<f32>,
  wave: vec4<f32>,
  previous_wind: vec4<f32>,
  previous_wave: vec4<f32>,
}
