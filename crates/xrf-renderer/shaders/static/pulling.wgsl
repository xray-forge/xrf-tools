#import "common/foliage_wind"
#import "static/records"
#import "generated/static/draw"

// A batch's visible clusters as its static draws pull them: every cluster one instance of `CLUSTER_VERTICES` vertices,
// read from its layout's arena through the shared index arena.

// `calc_cyclic`: a wave from minus one to one over each whole turn, a parabola rather than a sine.
fn cyclic(phase: f32) -> f32 {
  let f: f32 = fract(phase) * 2.8284271 - 1.4142136;

  return f * f - 1.0;
}

// What a vertex is to the foliage motion, in a `foliage` pair's `x`: not foliage, a trunk, or a branch or leaf.
const FOLIAGE_NONE: f32 = 0.0;
const FOLIAGE_TRUNK: f32 = 1.0;
const FOLIAGE_BRANCH: f32 = 2.0;

// `deffer_tree_*.vs`: a tree's vertex in the world moved across the ground by the wind, as far as its height over the
// tree's foot times the wave at its place, and as much of that as its rigidity lets it; under the enhanced motion,
// by its trunk's swing or its branch's toss, `foliage` its kind and its texture's `v`.
fn swayed(world: vec3<f32>, foot: f32, rigidity: f32, foliage: vec2<f32>) -> vec3<f32> {
  if (is_foliage_enhanced(foliage)) {
    return foliage_swayed(world, foot, foliage, wind.foliage_anim);
  }

  return swayed_by(world, foot, rigidity, wind.wind, wind.wave);
}

// The same vertex as the frame before's wind swayed it.
fn swayed_before(world: vec3<f32>, foot: f32, rigidity: f32, foliage: vec2<f32>) -> vec3<f32> {
  if (is_foliage_enhanced(foliage)) {
    return foliage_swayed(world, foot, foliage, wind.foliage_previous_anim);
  }

  return swayed_by(world, foot, rigidity, wind.previous_wind, wind.previous_wave);
}

// Whether the enhanced motion moves a vertex of this kind.
fn is_foliage_enhanced(foliage: vec2<f32>) -> bool {
  return wind.foliage_trees.w > 0.5 && foliage.x > 0.5;
}

// A tree's vertex moved by the enhanced motion at the fields' drift, read in the engine's space and carried back.
fn foliage_swayed(world: vec3<f32>, foot: f32, foliage: vec2<f32>, anim: vec4<f32>) -> vec3<f32> {
  let setup: FoliageSetup = FoliageSetup(wind.foliage_wind, wind.foliage_grass, wind.foliage_trees, anim);
  let height: f32 = world.y - foot;
  var moved: vec3<f32>;

  if (foliage.x > 1.5) {
    moved = foliage_branches(vec2<f32>(world.x, -world.z), foot, height, foliage.y, setup);
  } else {
    let trunk: vec3<f32> = foliage_trunk(foot, height, setup);

    moved = vec3<f32>(trunk.x, 0.0, trunk.y);
  }

  return world + vec3<f32>(moved.x, moved.y, -moved.z);
}

fn swayed_by(world: vec3<f32>, foot: f32, rigidity: f32, lean_wind: vec4<f32>, wave: vec4<f32>) -> vec3<f32> {
  let phase: f32 = cyclic(wave.w + dot(world, wave.xyz));
  let lean: vec2<f32> = lean_wind.xz * (world.y - foot) * phase * rigidity;

  return world + vec3<f32>(lean.x, 0.0, lean.y);
}

// Two shorts from a word, the low half first, sign extended.
fn unpack_shorts(word: u32) -> vec2<f32> {
  return vec2<f32>(f32(bitcast<i32>(word << 16u) >> 16u), f32(bitcast<i32>(word) >> 16u));
}

// A direction packed as a `D3DCOLOR`: blue, green, red bytes as z, y, x.
fn unpack_direction(packed: vec4<f32>) -> vec3<f32> {
  return packed.zyx * 2.0 - 1.0;
}

struct PulledVertex {
  word: u32,
  place: Place,
  surface: u32,
  entry: vec2<u32>,
};

fn pull(vertex_index: u32, instance_index: u32) -> PulledVertex {
  let entry: vec2<u32> = lists[instance_index];
  let cluster: Cluster = clusters[entry.x];
  // A cluster short of 128 triangles collapses its tail onto its last corner.
  let corner: u32 = min(vertex_index, cluster.triangles * 3u - 1u);
  let vertex: u32 = cluster.vertex_start + indices[cluster.first_index + corner];

  return PulledVertex(vertex * 8u, places[entry.y], slots[cluster.slot].surface, entry);
}
