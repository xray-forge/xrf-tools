#import "common/compute_grid"
#import "grass/puddles"
#import "grass/ring"
#import "grass/tuft"

// The grass's cards, which cast its shadows: every tuft the ring holds within `card_reach` metres of the eye, kept as
// the cull keeps it but whether the view sees it or not, its model's triangles stood and swayed where the grass draw
// stands them this frame, so the shadows meet the grass the frame draws, cut out by its texture's alpha: an
// acceleration structure is built over them where the view traces its light (`GrassCards`), and
// `grass/card_shadow.wgsl` draws them into the sun's nearest cascades where those shadow the frame. Over the reach's
// last `card_fade` metres each tuft shrinks to its foot, so its shadow fades out smoothly as the eye moves rather than
// popping at the reach; past it the grass casts no shadow. A triangle the frame leaves unwritten has every corner's `x`
// not a number, which the build and the draw take as no triangle.

// The models' arenas, the wind, and what is written: three floats a corner, then four words a triangle (its model and
// how many of its texture's widths a metre of it spans, then each corner's coordinate), and how many triangles stand.
@group(2) @binding(0) var<storage, read> positions: array<f32>;
@group(2) @binding(1) var<storage, read> uvs: array<f32>;
@group(2) @binding(2) var<storage, read> indices: array<u32>;
@group(2) @binding(3) var<uniform> wind: GrassWind;
@group(2) @binding(4) var<storage, read_write> card_vertices: array<u32>;
@group(2) @binding(5) var<storage, read_write> card_triangles: array<u32>;
@group(2) @binding(6) var<storage, read_write> card_count: atomic<u32>;
// The cascades' draw of them: their corners, one instance.
@group(2) @binding(7) var<storage, read_write> card_draw: array<u32, 4>;

// Words a triangle's corners and record take.
const CARD_VERTEX_WORDS: u32 = 9u;
const CARD_TRIANGLE_WORDS: u32 = 4u;

// A quiet not-a-number's bits, which mark a corner as none.
const NOT_A_NUMBER: u32 = 0x7fc00000u;

const WORKGROUP: u32 = 64u;

// Every triangle marked as none and the count cleared, before the tufts write theirs.
@compute @workgroup_size(64)
fn cs_clear_cards(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let triangle: u32 = compute_index(id, groups, WORKGROUP);

  if (triangle == 0u) {
    atomicStore(&card_count, 0u);
  }

  if (triangle >= arrayLength(&card_triangles) / CARD_TRIANGLE_WORDS) {
    return;
  }

  for (var corner: u32 = 0u; corner < 3u; corner++) {
    card_vertices[triangle * CARD_VERTEX_WORDS + corner * 3u] = NOT_A_NUMBER;
  }
}

// The share of its size a tuft stands as a card at `from_eye` metres from the eye: whole short of the fade, nothing
// past the reach.
fn card_share(from_eye: f32) -> f32 {
  return saturate((params.card_reach - from_eye) / params.card_fade);
}

// The ring's cell holding a slot `offset` slots from the eye's on each axis: `ring_axis` holds a slot in the cell its
// place is, taken round the ring's line.
fn window_cell(offset: vec2<i32>) -> u32 {
  let line: i32 = params.reach * 2 + 1;
  let at: vec2<i32> = ((params.center + offset) % line + line) % line;

  return u32(at.y * line + at.x);
}

// One tuft's cards: an invocation a tuft of each cell within `card_slots` of the eye's.
@compute @workgroup_size(64)
fn cs_grass_cards(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let index: u32 = compute_index(id, groups, WORKGROUP);
  let side: u32 = u32(params.card_slots) * 2u + 1u;
  let tuft: u32 = index % params.per_cell;
  let window: u32 = index / params.per_cell;

  if (window >= side * side) {
    return;
  }

  let offset: vec2<i32> = vec2<i32>(i32(window % side), i32(window / side)) - params.card_slots;

  if (any(abs(offset) > vec2<i32>(params.reach))) {
    return;
  }

  let cell: u32 = window_cell(offset);
  let slot: RingSlot = ring_slot(cell);
  let held: u32 = keys[cell * GRASS_KEY_WORDS + 3u];

  if (tuft >= min(held, params.per_cell) || !is_current(cell, slot)) {
    return;
  }

  let ring: RingCell = ring_cell(cell, slot);

  // A slot's middle within which some of its tufts may stand: a slot's width past the reach.
  let slot_reach: f32 = params.card_reach + GRASS_SLOT_METERS;

  if (ring.distance > min(params.fade * params.fade, slot_reach * slot_reach)) {
    return;
  }

  let source: u32 = (cell * params.per_cell + tuft) * 2u;
  let place: vec4<f32> = cached[source];
  let kept: vec4<f32> = cached[source + 1u];
  let kept_look: vec4<f32> = ring_tuft_look(ring, kept);
  let shrink: f32 = card_share(distance(place.xyz, vec3<f32>(params.eye.x, params.eye.y, -params.eye.z)));

  if (kept_look.x <= 0.0 || shrink <= 0.0 || is_in_puddle(place.xyz)) {
    return;
  }

  let look: vec4<f32> = vec4<f32>(kept_look.x * shrink, kept_look.yzw);
  let model_index: u32 = u32(kept.y);
  let model: GrassModel = models[model_index];
  let count: u32 = model.index_count / 3u;
  let first: u32 = atomicAdd(&card_count, count);

  if (first + count > arrayLength(&card_triangles) / CARD_TRIANGLE_WORDS) {
    return;
  }

  let stance: TuftStance = tuft_stance(wind, look);

  for (var triangle: u32 = 0u; triangle < count; triangle++) {
    var corners: array<vec3<f32>, 3>;
    var coordinates: array<vec2<f32>, 3>;

    for (var corner: u32 = 0u; corner < 3u; corner++) {
      let vertex: u32 = indices[model.first_index + triangle * 3u + corner];
      let position: vec3<f32> =
        vec3<f32>(positions[vertex * 3u], positions[vertex * 3u + 1u], positions[vertex * 3u + 2u]);
      let standing: vec3<f32> = tuft_standing(place, look.x, stance, position);
      let share: f32 = position.y / max(model.shape.w, 0.0001);

      corners[corner] = tuft_swayed(wind, standing, place.y, stance.waving, share, false);
      coordinates[corner] = vec2<f32>(uvs[vertex * 2u], uvs[vertex * 2u + 1u]);
    }

    let at: u32 = first + triangle;
    let world_area: f32 = length(cross(corners[1] - corners[0], corners[2] - corners[0]));
    let uv_step: vec2<f32> = coordinates[1] - coordinates[0];
    let uv_turn: vec2<f32> = coordinates[2] - coordinates[0];
    let uv_area: f32 = abs(uv_step.x * uv_turn.y - uv_step.y * uv_turn.x);
    let density: f32 = sqrt(uv_area / max(world_area, 1e-8));

    for (var corner: u32 = 0u; corner < 3u; corner++) {
      card_vertices[at * CARD_VERTEX_WORDS + corner * 3u] = bitcast<u32>(corners[corner].x);
      card_vertices[at * CARD_VERTEX_WORDS + corner * 3u + 1u] = bitcast<u32>(corners[corner].y);
      card_vertices[at * CARD_VERTEX_WORDS + corner * 3u + 2u] = bitcast<u32>(corners[corner].z);
      card_triangles[at * CARD_TRIANGLE_WORDS + 1u + corner] = pack2x16float(coordinates[corner]);
    }

    card_triangles[at * CARD_TRIANGLE_WORDS] = model_index | (pack2x16float(vec2<f32>(density, 0.0)) << 16u);
  }
}

// The cascades' draw of the cards that stand, as many as the cards hold at most.
@compute @workgroup_size(1)
fn cs_card_draw() {
  let count: u32 = min(atomicLoad(&card_count), arrayLength(&card_triangles) / CARD_TRIANGLE_WORDS);

  card_draw = array<u32, 4>(count * 3u, 1u, 0u, 0u);
}
