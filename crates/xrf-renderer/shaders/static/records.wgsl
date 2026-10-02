// The static scene's records, as `scene/static_scene` writes them.

struct Cluster {
  first_index: u32,
  triangles: u32,
  vertex_start: u32,
  slot: u32,
};

struct Slot {
  first_cluster: u32,
  cluster_count: u32,
  place: u32,
  kind: u32,
  batch: u32,
  surface: u32,
  pad0: u32,
  pad1: u32,
};

struct Place {
  m0: vec4<f32>,
  m1: vec4<f32>,
  m2: vec4<f32>,
  m3: vec4<f32>,
  // The hemisphere's scale and bias, the impostor or -1, the largest axis scale.
  info: vec4<f32>,
  cube: vec4<f32>,
};

struct Row {
  sphere: vec4<f32>,
  place: u32,
  slot: u32,
  lod: u32,
  band: u32,
};

struct Surface {
  tiling: f32,
  detail_scale: f32,
  alpha_reference: f32,
  slice: f32,
  color: vec3<f32>,
  flags: u32,
  base: u32,
  detail: u32,
  bump: u32,
  bump_companion: u32,
  detail_bump: u32,
  detail_bump_companion: u32,
  hemi: u32,
  pad: u32,
};

struct Region {
  base: u32,
  capacity: u32,
};

struct Impostor {
  sphere: vec4<f32>,
  normals: array<vec4<f32>, 8>,
  factor: f32,
  surface: u32,
  pad0: u32,
  pad1: u32,
};

const IMPOSTOR_FACETS: u32 = 8u;

// What an impostor's level of detail draws of its clump: its trees, itself, or both while one fades into the other.
const LOD_TREES: u32 = 1u;
const LOD_IMPOSTOR: u32 = 2u;

// A row no impostor decides.
const NO_LOD: u32 = 0xffffffffu;

const SLOT_SINGLE: u32 = 1u;
const SLOT_LISTED: u32 = 2u;

const SURFACE_HAS_BASE: u32 = 1u;
const SURFACE_HAS_DETAIL: u32 = 2u;
const SURFACE_HAS_BUMP: u32 = 4u;
const SURFACE_HAS_DETAIL_BUMP: u32 = 8u;
const SURFACE_HAS_HEMI: u32 = 16u;

// Vertices one cluster's draw spans: 128 triangles, those past its own collapsed.
const CLUSTER_VERTICES: u32 = 384u;

fn place_matrix(place: Place) -> mat4x4<f32> {
  return mat4x4<f32>(place.m0, place.m1, place.m2, place.m3);
}
