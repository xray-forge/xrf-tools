#import "generated/structs"

// `common/scene_trace` for a device without ray queries: the same functions, binding nothing, meeting nothing.

struct SceneHit {
  distance: f32,
  record: u32,
  primitive: u32,
  barycentrics: vec2<f32>,
  place: u32,
  position: vec3<f32>,
  to_world: mat3x3<f32>,
}

struct SceneSurface {
  position: vec3<f32>,
  normal: vec3<f32>,
  albedo: vec3<f32>,
  hemi: f32,
  sun: f32,
  slice: f32,
  is_leaf: bool,
  crown: vec3<f32>,
}

struct SceneCone {
  width: f32,
  spread: f32,
}

const IS_SCENE_TRACED: bool = false;
const SCENE_NO_PLACE: u32 = 0xffffffffu;
const SCENE_SUN_REACH: f32 = 1000.0;

fn trace_scene_ray(origin: vec3<f32>, direction: vec3<f32>, near: f32, far: f32, cone: SceneCone) -> SceneHit {
  return SceneHit(-1.0, 0u, 0u, vec2<f32>(0.0), 0u, vec3<f32>(0.0), mat3x3<f32>());
}

fn trace_scene_blocked(origin: vec3<f32>, direction: vec3<f32>, far: f32, cone: SceneCone, passed: u32) -> bool {
  return false;
}

fn shade_scene_hit(
  hit: SceneHit,
  toward: vec3<f32>,
  cone: SceneCone,
  state: Lighting,
  lut: texture_3d<f32>,
  lut_sampler: sampler,
  environment_0: texture_cube<f32>,
  environment_1: texture_cube<f32>,
  cube_sampler: sampler,
) -> vec3<f32> {
  return vec3<f32>(0.0);
}

fn scene_lamps(
  position: vec3<f32>,
  normal: vec3<f32>,
  toward: vec3<f32>,
  slice: f32,
  lut: texture_3d<f32>,
  lut_sampler: sampler,
) -> vec4<f32> {
  return vec4<f32>(0.0);
}

fn scene_surface(hit: SceneHit, spread: f32) -> SceneSurface {
  return SceneSurface(vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0), 0.0, 0.0, 0.0, false,
    vec3<f32>(0.0, 1.0, 0.0));
}

struct SceneGrassPassed {
  opacity: f32,
  albedo: vec3<f32>,
  hemi: f32,
  entry: f32,
}

fn scene_grass_passed(origin: vec3<f32>, toward: vec3<f32>, length: f32, is_met: bool) -> SceneGrassPassed {
  return SceneGrassPassed(0.0, vec3<f32>(0.0), 0.0, -1.0);
}

fn shade_scene_grass(
  grass: SceneGrassPassed,
  origin: vec3<f32>,
  toward: vec3<f32>,
  state: Lighting,
  lut: texture_3d<f32>,
  lut_sampler: sampler,
  environment_0: texture_cube<f32>,
  environment_1: texture_cube<f32>,
  cube_sampler: sampler,
) -> vec3<f32> {
  return vec3<f32>(0.0);
}
