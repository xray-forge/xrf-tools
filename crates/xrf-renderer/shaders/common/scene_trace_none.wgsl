// `common/scene_trace` for a device without ray queries: the same functions, binding nothing, meeting nothing.

struct SceneHit {
  distance: f32,
  record: u32,
  primitive: u32,
  barycentrics: vec2<f32>,
}

struct SceneSurface {
  position: vec3<f32>,
  normal: vec3<f32>,
  albedo: vec3<f32>,
  hemi: f32,
  sun: f32,
  slice: f32,
}

const IS_SCENE_TRACED: bool = false;

fn trace_scene_ray(origin: vec3<f32>, direction: vec3<f32>, near: f32, far: f32) -> SceneHit {
  return SceneHit(-1.0, 0u, 0u, vec2<f32>(0.0));
}

fn trace_scene_blocked(origin: vec3<f32>, direction: vec3<f32>, far: f32) -> bool {
  return false;
}

fn scene_surface(hit: SceneHit, spread: f32) -> SceneSurface {
  return SceneSurface(vec3<f32>(0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0), 0.0, 0.0, 0.0);
}
