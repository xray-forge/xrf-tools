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
  // A spawned object's hemisphere cube: its six faces, `+x +y +z -x -y -z`, as bytes in `x` and `y`; `w` one where it
  // has one.
  cube: vec4<u32>,
  // A skinned model's: its bone matrices' first row, its links' first vertex, its own first vertex in the model arena,
  // and how many bones it has, zero where it is rigid. The last frame's matrices follow this frame's.
  skin: vec4<u32>,
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
  // An environment-mapped model's cube, by environment slot; none at zero.
  environment: u32,
  // A terrain's details the mask's red, green, blue and alpha weigh, their bumps, and the mask.
  terrain_details: vec4<u32>,
  terrain_bumps: vec4<u32>,
  terrain_mask: u32,
  pad0: u32,
  pad1: u32,
  pad2: u32,
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

// A row's band word, as `StaticRow::pack_band` and `pack_group` lay it: its progressive band and the bands its mesh has
// in four bits each, the slide windows they share in sixteen, and the visibility group showing it in the top byte.
fn row_band(word: u32) -> u32 {
  return word & 15u;
}

fn row_bands(word: u32) -> u32 {
  return (word >> 4u) & 15u;
}

fn row_windows(word: u32) -> u32 {
  return (word >> 8u) & 0xffffu;
}

fn row_group(word: u32) -> u32 {
  return word >> 24u;
}

// The shading classes a layout's batches run through, as `StaticClass` orders them: the two the G-buffer draws and
// shadows cast first, then water, composited surfaces and wall marks.
const CLASS_COUNT: u32 = 5u;
const WATER_CLASS: u32 = 2u;

const SLOT_SINGLE: u32 = 1u;
const SLOT_LISTED: u32 = 2u;

const SURFACE_HAS_BASE: u32 = 1u;
const SURFACE_HAS_DETAIL: u32 = 2u;
const SURFACE_HAS_BUMP: u32 = 4u;
const SURFACE_HAS_DETAIL_BUMP: u32 = 8u;
const SURFACE_HAS_HEMI: u32 = 16u;
const SURFACE_IS_CUT_OUT: u32 = 32u;
const SURFACE_IS_SOFT_WATER: u32 = 64u;
const SURFACE_IS_ANOMALY_WATER: u32 = 128u;
const SURFACE_IS_REFLECTING: u32 = 256u;
const SURFACE_IS_SPECULAR: u32 = 512u;
const SURFACE_IS_TRANSPARENT: u32 = 1024u;
const SURFACE_IS_FOAMED: u32 = 2048u;
const SURFACE_HAS_WATER_NORMAL: u32 = 4096u;
const SURFACE_HAS_FOAM: u32 = 8192u;
const SURFACE_HAS_DISTORTION: u32 = 16384u;
const SURFACE_IS_ADDED: u32 = 32768u;
const SURFACE_IS_WEIGHTED: u32 = 65536u;
const SURFACE_IS_MULTIPLIED: u32 = 131072u;
const SURFACE_IS_DOUBLED: u32 = 262144u;
const SURFACE_IS_MODEL: u32 = 524288u;
const SURFACE_IS_ENVIRONMENT_MAPPED: u32 = 1048576u;
// An object the level stores as a tree, which the wind leaves standing.
const SURFACE_IS_STILL: u32 = 2097152u;
// A terrain's, which lays four details and their bumps over its base by its mask, lit by its base's alpha.
const SURFACE_IS_TERRAIN: u32 = 4194304u;

// Vertices one cluster's draw spans: 128 triangles, those past its own collapsed.
const CLUSTER_VERTICES: u32 = 384u;

fn place_matrix(place: Place) -> mat4x4<f32> {
  return mat4x4<f32>(place.m0, place.m1, place.m2, place.m3);
}
