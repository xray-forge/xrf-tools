#import "common/compute_grid"
#import "static/records"
#import "static/sway"
#import "static/vertex"

// The trace's swaying trees: each vertex of the instances a view sways, placed in the world and swayed by the view's
// wind as the G-buffer's tree draws sway it (`static/sway`), written for their acceleration structures to be built
// over, so the trace's trees stand where the frame draws them.

// The tree arena's words, the places and surfaces, the wind, the swaying instances, and what is written.
#import "generated/static/trace_sway"

const WORKGROUP: u32 = 64u;

// The swaying instance an output vertex belongs to: the last whose first output vertex is at or before it.
fn slot_of(vertex: u32) -> u32 {
  var low: u32 = 0u;
  var high: u32 = arrayLength(&sway_slots);

  while (high - low > 1u) {
    let middle: u32 = (low + high) / 2u;

    if (sway_slots[middle].first_out <= vertex) {
      low = middle;
    } else {
      high = middle;
    }
  }

  return low;
}

@compute @workgroup_size(64)
fn cs_trace_sway(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let vertex: u32 = compute_index(id, groups, WORKGROUP);

  if (vertex * 3u >= arrayLength(&swayed_vertices)) {
    return;
  }

  let slot: TraceSwaySlot = sway_slots[slot_of(vertex)];
  let local: u32 = vertex - slot.first_out;

  if (local >= slot.vertex_count) {
    return;
  }

  let at: u32 = (slot.vertex_start + local) * VERTEX_WORDS;
  let place: Place = places[slot.place];
  let flags: u32 = surfaces[slot.surface].flags;
  let is_still: bool = (flags & SURFACE_IS_STILL) != 0u;
  let rigidity: f32 = select(unpack_shorts(words[at + VERTEX_EXTRA]).x / 2048.0, 0.0, is_still);
  let v: f32 = unpack_shorts(words[at + VERTEX_UV]).y / 2048.0;
  let position: vec3<f32> = vertex_position(words[at + VERTEX_POSITION], words[at + VERTEX_POSITION + 1u],
    words[at + VERTEX_POSITION + 2u]);
  let placed: vec3<f32> = (place.transform * vec4<f32>(position, 1.0)).xyz;
  let foliage: vec2<f32> = vec2<f32>(tree_foliage_kind(is_still, (flags & SURFACE_IS_CUT_OUT) != 0u), v);
  let swayed: vec3<f32> = tree_swayed(wind, placed, place.transform[3].y, rigidity, foliage);

  swayed_vertices[vertex * 3u] = swayed.x;
  swayed_vertices[vertex * 3u + 1u] = swayed.y;
  swayed_vertices[vertex * 3u + 2u] = swayed.z;
}
