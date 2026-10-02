#import "static/records"

// A batch's visible clusters as its static draws pull them: every cluster one instance of `CLUSTER_VERTICES` vertices,
// read from its layout's arena through the shared index arena.

@group(1) @binding(0) var<storage, read> clusters: array<Cluster>;
@group(1) @binding(1) var<storage, read> slots: array<Slot>;
@group(1) @binding(2) var<storage, read> places: array<Place>;
@group(1) @binding(3) var<storage, read> surfaces: array<Surface>;
@group(1) @binding(4) var<storage, read> indices: array<u32>;
@group(1) @binding(5) var<storage, read> lists: array<vec2<u32>>;
@group(1) @binding(6) var<storage, read> words: array<u32>;
@group(1) @binding(7) var<uniform> wind: Wind;

// How the trees sway this frame, as `pass/wind_uniform.rs` writes it.
struct Wind {
  // The engine's `wind`: which way the trees lean, and how far, across the ground.
  wind: vec4<f32>,
  // The engine's `wave`: its direction through the level, and its phase in `w`, both over a turn.
  wave: vec4<f32>,
};

// `calc_cyclic`: a wave from minus one to one over each whole turn, a parabola rather than a sine.
fn cyclic(phase: f32) -> f32 {
  let f: f32 = fract(phase) * 2.8284271 - 1.4142136;

  return f * f - 1.0;
}

// `deffer_tree_*.vs`: a tree's vertex in the world moved across the ground by the wind, as far as its height over the
// tree's foot times the wave at its place, and as much of that as its rigidity lets it.
fn swayed(world: vec3<f32>, foot: f32, rigidity: f32) -> vec3<f32> {
  let phase: f32 = cyclic(wind.wave.w + dot(world, wind.wave.xyz));
  let lean: vec2<f32> = wind.wind.xz * (world.y - foot) * phase * rigidity;

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
