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
  flora: vec4<f32>,
  indirect: vec4<f32>,
  reflections: vec4<f32>,
  height_fog: vec4<f32>,
  sky_fade: vec4<f32>,
}

struct TraceSwaySlot {
  vertex_start: u32,
  vertex_count: u32,
  place: u32,
  surface: u32,
  first_out: u32,
}

struct Glass {
  refraction: f32,
}

struct Smoothing {
  linear: f32,
}

struct Froxels {
  size: vec4<f32>,
  params: vec4<f32>,
}

struct GrassModel {
  shape: vec4<f32>,
  is_waving: f32,
  texture: u32,
  alpha_reference: f32,
  index_count: u32,
  first_index: u32,
  spread: f32,
  pad: array<u32, 2>,
}

struct TraceGrass {
  grid: vec4<i32>,
  steps: i32,
  fade: f32,
  height: f32,
  model_count: u32,
  slope_limit: f32,
}

struct TraceRequest {
  origin: vec4<f32>,
  cone: vec2<f32>,
  toward: u32,
  pixel: u32,
  screen: vec2<u32>,
  screen_length: f32,
  hit_end: u32,
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
}

struct EnhancedWater {
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
  trace_distance: f32,
  trace_turns: u32,
  trace_frame: u32,
  vibrance: f32,
}

struct WaterBlur {
  direction: vec4<f32>,
}

struct Present {
  view: u32,
  is_occluded: u32,
  is_upscaled: u32,
  distortion: f32,
  origin: vec2<f32>,
  size: vec2<f32>,
  corrections: vec4<f32>,
  grading: vec4<f32>,
  selection: vec4<f32>,
  is_bloomed: u32,
  is_indirect: u32,
  is_reflected: u32,
  reflection_ratio: f32,
  deband: vec4<f32>,
  tonemap: vec4<f32>,
  inspect: vec4<f32>,
}

struct Upscale {
  output_size: vec2<f32>,
  sharpness: f32,
  linear: f32,
}

struct Bloom {
  params: vec4<f32>,
  weights: array<vec4<f32>, 2>,
}

struct EnhancedBloom {
  size: vec4<f32>,
  spread: vec4<f32>,
  strengths: vec4<f32>,
}

struct FogScattering {
  size: vec2<f32>,
  intensity: f32,
  time: f32,
}

struct PuddleSites {
  window: vec4<f32>,
  shape: vec4<f32>,
  water_window: vec4<f32>,
  fires: array<vec4<f32>, 32>,
}

struct SkyPrefilter {
  level: vec4<f32>,
  shape: vec4<f32>,
}

struct OverheadShift {
  offset: vec4<f32>,
}

struct Surroundings {
  to_sun: vec4<f32>,
  face: vec4<f32>,
}

struct ParticleVertex {
  position: vec3<f32>,
  color: u32,
  uv: vec2<f32>,
  surface: u32,
}

struct ParticleSurface {
  texture: u32,
  flags: u32,
  alpha_reference: f32,
  distortion: u32,
}

struct Shadows {
  matrices: array<mat4x4<f32>, 4>,
  texels: vec4<f32>,
  forward: vec4<f32>,
  count: u32,
  filter_reach: u32,
  resolution: f32,
  bias: f32,
  blends: vec4<f32>,
}

struct ContactShadows {
  to_sun: vec4<f32>,
  length: f32,
  intensity: f32,
  thickness: f32,
  steps: u32,
  reach: f32,
  noise: f32,
  lights: u32,
  stride: u32,
}

struct SunTrace {
  to_sun: vec4<f32>,
  spread: f32,
  noise_x: f32,
  noise_y: f32,
}

struct LightTrace {
  noise: vec4<f32>,
  radius: f32,
}

struct IndirectTrace {
  noise: vec4<f32>,
}

struct Lights {
  count: u32,
  near: f32,
  far: f32,
  shadow_filter: u32,
  projection: vec4<f32>,
}

struct LightRecord {
  position: vec4<f32>,
  color: vec4<f32>,
  axis: vec4<f32>,
  right: vec4<f32>,
  up: vec4<f32>,
  sphere: vec4<f32>,
  shadow: vec4<f32>,
  faces: array<vec4<f32>, 6>,
}

struct LightVolume {
  near: vec4<f32>,
  far: vec4<f32>,
  color: vec4<f32>,
  frustum: vec4<f32>,
}

struct LightVolumes {
  noise: u32,
  scroll: f32,
}

struct AmbientOcclusion {
  radius: f32,
  power: f32,
  spread: f32,
  reach: f32,
}

struct Vbao {
  radius: f32,
  thickness: f32,
  power: f32,
  spread: f32,
  reach: f32,
  frames: f32,
  has_history: f32,
  slice_noise: f32,
  step_noise: f32,
  intensity: f32,
  has_light_history: f32,
  light_radius: f32,
}

struct Reflections {
  crossings: u32,
  frame: u32,
  intensity: f32,
  distance: f32,
  has_history: f32,
  ratio: f32,
  has_source: f32,
  is_traced: f32,
  trace_turns: u32,
}

struct ExposureParams {
  target_gray: f32,
  weight: f32,
  floor_luminance: f32,
  blend: f32,
}

struct ExposureState {
  adapted: f32,
  used: f32,
  pad1: f32,
  pad2: f32,
  cells: array<f32, 4096>,
}

struct Exposure {
  adapted: f32,
  used: f32,
}

struct Flares {
  to_sun: vec4<f32>,
  sun: vec4<f32>,
  color: vec4<f32>,
  gradient: vec4<f32>,
  flares: array<vec4<f32>, 16>,
}

struct Rain {
  color: vec4<f32>,
  axis: vec4<f32>,
  window: vec4<f32>,
  cover_shape: vec4<f32>,
  count: u32,
  time: f32,
  splash_indices: u32,
}

struct Wet {
  density: f32,
  time: f32,
  is_extended: f32,
  is_enhanced: f32,
  cover: vec4<f32>,
  cover_state: vec4<f32>,
  surface: vec4<f32>,
  surface_shape: vec4<f32>,
  puddles: vec4<f32>,
  puddle_state: vec4<f32>,
  reach: vec4<f32>,
  gust: vec4<f32>,
  water: vec4<f32>,
  relief: vec4<f32>,
  drying: vec4<f32>,
}

struct Thunder {
  axes: array<vec4<f32>, 3>,
  position: vec4<f32>,
  shift: vec4<f32>,
  top: vec4<f32>,
  top_extent: vec4<f32>,
  center: vec4<f32>,
  center_extent: vec4<f32>,
}

struct Temporal {
  current: mat4x4<f32>,
  previous: mat4x4<f32>,
  previous_view: mat4x4<f32>,
  params: vec4<f32>,
}

struct Fsr {
  render_size: vec2<f32>,
  display_size: vec2<f32>,
  jitter: vec2<f32>,
  previous_jitter: vec2<f32>,
  downscale: vec2<f32>,
  shading_size: vec2<f32>,
  device_to_view: vec4<f32>,
  jitter_phase_count: f32,
  frame_index: f32,
  velocity_factor: f32,
  hdr: f32,
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
  passes: u32,
  pad: u32,
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
  heights: vec4<u32>,
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
  distortion: u32,
}

struct Wind {
  wind: vec4<f32>,
  wave: vec4<f32>,
  previous_wind: vec4<f32>,
  previous_wave: vec4<f32>,
  foliage_wind: vec4<f32>,
  foliage_grass: vec4<f32>,
  foliage_trees: vec4<f32>,
  foliage_anim: vec4<f32>,
  foliage_previous_anim: vec4<f32>,
  foliage_flora: vec4<f32>,
}

struct TraceInstance {
  first_record: u32,
  place: u32,
  pad: array<u32, 2>,
}

struct TraceRecord {
  first_index: u32,
  vertex_start: u32,
  surface: u32,
  flags: u32,
}
