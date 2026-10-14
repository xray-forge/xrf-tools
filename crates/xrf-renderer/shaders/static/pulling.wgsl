#import "static/sway"
#import "static/records"
#import "static/vertex"
#import "generated/static/draw"

// A batch's visible clusters as its static draws pull them: every cluster one instance of `CLUSTER_VERTICES` vertices,
// read from its layout's arena through the shared index arena.

// A tree's vertex as the draw's wind sways it (`static/sway`).
fn swayed(world: vec3<f32>, foot: f32, rigidity: f32, foliage: vec2<f32>) -> vec3<f32> {
  return tree_swayed(wind, world, foot, rigidity, foliage);
}

// The same vertex as the frame before's wind swayed it.
fn swayed_before(world: vec3<f32>, foot: f32, rigidity: f32, foliage: vec2<f32>) -> vec3<f32> {
  return tree_swayed_before(wind, world, foot, rigidity, foliage);
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

  return PulledVertex(vertex * VERTEX_WORDS, places[entry.y], slots[cluster.slot].surface, entry);
}
