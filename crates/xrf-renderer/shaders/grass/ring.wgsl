#import "grass/planting_bindings"

// The world slot a cell of the ring holds while the camera stands where it does, and how many rings out it is.
struct RingSlot {
  x: i32,
  z: i32,
  band: i32,
};

// Cells the ring holds now: `dm_cache_line` squared.
fn ring_cells() -> u32 {
  let line: u32 = u32(params.reach) * 2u + 1u;

  return line * line;
}

// One axis of the ring: the slot `first + ((cell - first) mod line)`, wrapped for a negative `first`.
fn ring_axis(center: i32, line: i32, cell: i32) -> i32 {
  let first: i32 = center - params.reach;

  return first + (((cell - first) % line) + line) % line;
}

// `cache_Update`'s ring: of the slots within reach of the camera's, the one a cell holds is the one whose place in the
// ring, taken round, is the cell's, so a step of the camera changes only the row or column it steps into.
fn ring_slot(cell: u32) -> RingSlot {
  let line: i32 = params.reach * 2 + 1;
  let x: i32 = ring_axis(params.center.x, line, i32(cell) % line);
  let z: i32 = ring_axis(params.center.y, line, i32(cell) / line);

  return RingSlot(x, z, max(abs(x - params.center.x), abs(z - params.center.y)));
}

// Whether a cell was planted with the slot it holds now under this generation, so what it holds stands.
fn is_current(cell: u32, slot: RingSlot) -> bool {
  let key: u32 = cell * GRASS_KEY_WORDS;

  return keys[key] == bitcast<u32>(slot.x) && keys[key + 1u] == bitcast<u32>(slot.z)
    && keys[key + 2u] == params.generation;
}

// A current cell as the cull reads it: its slot's middle in the engine's space, its squared distance from the eye, the
// share its tufts shrink to by it, from one at the eye to nought at `dm_fade` by squared metres, and its shape.
struct RingCell {
  center: vec3<f32>,
  distance: f32,
  shrink: f32,
  ground: vec4<f32>,
}

fn ring_cell(cell: u32, slot: RingSlot) -> RingCell {
  let ground: vec4<f32> = shapes[cell];
  let center: vec3<f32> = vec3<f32>(f32(slot.x) * 2.0 + 1.0, ground.x, f32(slot.z) * 2.0 + 1.0);
  let offset: vec3<f32> = params.eye.xyz - center;
  let distance: f32 = dot(offset, offset);
  let fade_limit: f32 = params.fade * params.fade;
  let shrink: f32 = 1.0 - select((distance - 1.0) / (fade_limit - 1.0), 0.0, distance < 1.0);

  return RingCell(center, distance, shrink, ground);
}

// A cached tuft's look as the cull keeps it in `cell`: its size shrunk, its slot's hemisphere and sun, and its wave,
// nought to two, with its ground's normal, a whole number of `GROUND_NORMAL_STEPS` squared, in one float it holds
// exactly; a size of nought where its screen area is under `r_ssaDISCARD`, and no wave where it is small enough that
// the engine stops waving it.
fn ring_tuft_look(cell: RingCell, kept: vec4<f32>) -> vec4<f32> {
  let model: GrassModel = models[u32(kept.y)];
  let shrunk: f32 = kept.x * params.height * cell.shrink;
  let area: f32 = shrunk * shrunk * model.shape.z * model.shape.z / max(cell.distance, 0.0001);

  if (area < params.discard_below) {
    return vec4<f32>(0.0);
  }

  let is_waving: bool = model.is_waving > 0.5 && area > params.discard_below * 16.0;

  return vec4<f32>(shrunk, cell.ground.z, cell.ground.w, select(0.0, kept.z, is_waving) + 4.0 * kept.w);
}
