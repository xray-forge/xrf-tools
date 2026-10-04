#import "common/compute_grid"
#import "grass/records"

// The passes planting the grass, as `CDetailManager` does. A ring of slots around the camera is its cache: a slot
// coming into it is planted once, the nearest first within a budget a frame, and every frame culls what the ring holds,
// then sorts what it keeps into a draw a model.

@group(0) @binding(0) var<uniform> params: GrassParams;
@group(0) @binding(1) var<storage, read> grid: array<u32>;
@group(0) @binding(2) var<storage, read> slot_records: array<u32>;
@group(0) @binding(3) var<storage, read> bins: array<u32>;
@group(0) @binding(4) var<storage, read> triangles: array<f32>;
@group(0) @binding(5) var<storage, read> dither: array<u32, 256>;
@group(0) @binding(6) var<storage, read> models: array<GrassModel>;
// Items planted a model, then the whole frame's in the last.
@group(0) @binding(7) var<storage, read_write> counts: array<atomic<u32>>;
// Where each model's range goes on filling, as the items are sorted into it.
@group(0) @binding(8) var<storage, read_write> cursors: array<atomic<u32>>;
// One indexed draw a model.
@group(0) @binding(9) var<storage, read_write> args: array<u32>;

// The ring: a key a cell, a vector a cell (its ground's middle height and half its height, its hemisphere and sun), and
// two vectors a cached tuft (its place and turn, then its size, model and wave).
@group(1) @binding(0) var<storage, read_write> keys: array<u32>;
@group(1) @binding(1) var<storage, read_write> shapes: array<vec4<f32>>;
@group(1) @binding(2) var<storage, read_write> cached: array<vec4<f32>>;
// Stale cells a band counts, then the schedule: the band the frame's planting stops in, what it plants there, tickets.
@group(1) @binding(3) var<storage, read_write> band_counts: array<atomic<u32>>;
@group(1) @binding(4) var<storage, read_write> schedule: array<atomic<u32>, 3>;
// The frame's items as planted, each one's model, and the same sorted by model, which the draws read.
@group(1) @binding(5) var<storage, read_write> planted: array<vec4<f32>>;
@group(1) @binding(6) var<storage, read_write> planted_models: array<u32>;
@group(1) @binding(7) var<storage, read_write> sorted: array<vec4<f32>>;

// Slots a frame plants at most: `dm_max_decompress`, the engine's own seven on the CPU, grown for a GPU that plants
// them side by side. A step of the camera stales a row of the ring, well within it; a jump stales the whole ring, which
// then fills over some frames, the nearest first.
const DECOMPRESS_BUDGET: u32 = 2048u;

const STOP_WORD: u32 = 0u;
const LEFT_WORD: u32 = 1u;
const TICKET_WORD: u32 = 2u;

// The seed `cache_Decompress` starts every one of a slot's four generators from, before its slot is mixed in.
const SEED: u32 = 0x12071980u;

// `DetailSlot::ID_Empty`: a corner planting nothing.
const EMPTY_ID: u32 = 0x3fu;

// `EPS`: the determinant below which a ray is taken to lie in a triangle's plane.
const RAY_EPSILON: f32 = 0.0000100;

// `EPS_L`, which every slot's box grows by.
const BOX_GROWTH: f32 = 0.0010000;

const TURN: f32 = 6.2831853;

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

// `CRandom::randI()`: the state advanced, and bits 16 to 30 of it.
fn random(state: ptr<function, u32>) -> u32 {
  *state = *state * 214013u + 2531011u;

  return (*state >> 16u) & 0x7fffu;
}

// `CRandom::randF()`: `randI() / 32767`.
fn random_float(state: ptr<function, u32>) -> f32 {
  return f32(random(state)) / 32767.0;
}

// `CRandom::randFs(range)`.
fn random_signed(state: ptr<function, u32>, range: f32) -> f32 {
  return random_float(state) * range * 2.0 - range;
}

// Whether a sphere lies wholly outside one of the view's planes.
fn is_outside(center: vec3<f32>, radius: f32) -> bool {
  for (var plane: u32 = 0u; plane < 6u; plane++) {
    if (dot(params.planes[plane].xyz, center) + params.planes[plane].w < -radius) {
      return true;
    }
  }

  return false;
}

@compute @workgroup_size(64)
fn clear_schedule(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let index: u32 = compute_index(id, groups, 64u);

  if (index < params.bands) {
    atomicStore(&band_counts[index], 0u);
  }
}

// `cache_Update`'s task list: every stale cell of the ring, counted into the band it stands in.
@compute @workgroup_size(64)
fn rank(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let cell: u32 = compute_index(id, groups, 64u);

  if (cell >= ring_cells()) {
    return;
  }

  let slot: RingSlot = ring_slot(cell);

  if (!is_current(cell, slot)) {
    atomicAdd(&band_counts[u32(slot.band)], 1u);
  }
}

// The task performer's pick, nearest first: the band the frame's budget runs out in, and how many of its cells it still
// plants there.
@compute @workgroup_size(1)
fn select_band() {
  var taken: u32 = 0u;
  // Past every band: all that is stale is planted.
  var stop: u32 = params.bands;
  var left: u32 = 0u;

  for (var band: u32 = 0u; band < params.bands; band++) {
    let count: u32 = atomicLoad(&band_counts[band]);

    if (taken + count > DECOMPRESS_BUDGET) {
      stop = band;
      left = DECOMPRESS_BUDGET - taken;
      break;
    }

    taken += count;
  }

  atomicStore(&schedule[STOP_WORD], stop);
  atomicStore(&schedule[LEFT_WORD], left);
  atomicStore(&schedule[TICKET_WORD], 0u);
}

// Whether a stale cell is planted this frame: any in a band nearer than the one the budget runs out in, and in that one
// the first to take a ticket, as many as the budget has left.
fn is_admitted(band: i32) -> bool {
  let stop: u32 = atomicLoad(&schedule[STOP_WORD]);

  if (u32(band) < stop) {
    return true;
  }

  if (u32(band) == stop) {
    return atomicAdd(&schedule[TICKET_WORD], 1u) < atomicLoad(&schedule[LEFT_WORD]);
  }

  return false;
}

// `InterpolateAndDither`'s density: an object's four corner densities, each `255 * a / 15`, interpolated both ways over
// the slot and averaged, as `Interpolate` does.
fn density(palette: u32, column: u32, row: u32) -> i32 {
  let c0: f32 = f32(palette & 0xfu) * 255.0 / 15.0;
  let c1: f32 = f32((palette >> 4u) & 0xfu) * 255.0 / 15.0;
  let c2: f32 = f32((palette >> 8u) & 0xfu) * 255.0 / 15.0;
  let c3: f32 = f32((palette >> 12u) & 0xfu) * 255.0 / 15.0;
  let fx: f32 = f32(column) / f32(params.steps);
  let fy: f32 = f32(row) / f32(params.steps);
  let c01: f32 = c0 * (1.0 - fx) + c1 * fx;
  let c23: f32 = c2 * (1.0 - fx) + c3 * fx;
  let c02: f32 = c0 * (1.0 - fy) + c2 * fy;
  let c13: f32 = c1 * (1.0 - fy) + c3 * fy;
  let interpolated: f32 = ((1.0 - fy) * c01 + fy * c23 + (1.0 - fx) * c02 + fx * c13) / 2.0;

  return clamp(i32(floor(interpolated + 0.5)), 0, 255);
}

// `CDB::TestRayTri` straight down, culling back faces as the planting asks: how far below the origin the ray meets the
// triangle, or minus one where it misses.
fn ray_range(origin: vec3<f32>, p0: vec3<f32>, p1: vec3<f32>, p2: vec3<f32>) -> f32 {
  let direction: vec3<f32> = vec3<f32>(0.0, -1.0, 0.0);
  let edge1: vec3<f32> = p1 - p0;
  let edge2: vec3<f32> = p2 - p0;
  let pvec: vec3<f32> = cross(direction, edge2);
  let det: f32 = dot(edge1, pvec);
  let tvec: vec3<f32> = origin - p0;
  let u: f32 = dot(tvec, pvec);
  let qvec: vec3<f32> = cross(tvec, edge1);
  let v: f32 = dot(direction, qvec);

  if (det >= RAY_EPSILON && u >= 0.0 && u <= det && v >= 0.0 && u + v <= det) {
    return dot(edge2, qvec) / max(det, RAY_EPSILON);
  }

  return -1.0;
}

fn triangle_corner(triangle: u32, corner: u32) -> vec3<f32> {
  let at: u32 = triangle * GRASS_TRIANGLE_FLOATS + corner * 3u;

  return vec3<f32>(triangles[at], triangles[at + 1u], triangles[at + 2u]);
}

// `cache_Decompress`, a thread a cell of the ring: a stale cell the schedule admits plants the world slot it holds now,
// with the same generators seeded the same way and drawn in the same order, the same dither, the same ray cast down onto
// the slot's own triangles, so a slot plants what the engine plants in it.
@compute @workgroup_size(64)
fn decompress(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let cell: u32 = compute_index(id, groups, 64u);

  if (cell >= ring_cells()) {
    return;
  }

  let slot: RingSlot = ring_slot(cell);

  // Asking takes a ticket, which only a stale cell may spend.
  if (is_current(cell, slot) || !is_admitted(slot.band)) {
    return;
  }

  let key: u32 = cell * GRASS_KEY_WORDS;

  keys[key] = bitcast<u32>(slot.x);
  keys[key + 1u] = bitcast<u32>(slot.z);
  keys[key + 2u] = params.generation;
  keys[key + 3u] = 0u;

  let cell_x: i32 = slot.x + params.grid.z;
  let cell_z: i32 = slot.z + params.grid.w;

  if (cell_x < 0 || cell_z < 0 || cell_x >= params.grid.x || cell_z >= params.grid.y) {
    return;
  }

  let record: u32 = grid[u32(cell_z * params.grid.x + cell_x)];

  if (record == 0u) {
    return;
  }

  let at: u32 = (record - 1u) * GRASS_SLOT_WORDS;
  let w0: u32 = slot_records[at];
  let w1: u32 = slot_records[at + 1u];
  let w2: u32 = slot_records[at + 2u];
  let w3: u32 = slot_records[at + 3u];
  let bin_start: u32 = slot_records[at + 4u];
  let bin_count: u32 = slot_records[at + 5u];
  let base: f32 = f32(w0 & 0xfffu) * 0.2 - 200.0;
  let top: f32 = base + f32((w0 >> 12u) & 0xffu) * 0.1;
  let ids: array<u32, 4> = array<u32, 4>((w0 >> 20u) & 0x3fu, (w0 >> 26u) & 0x3fu, w1 & 0x3fu, (w1 >> 6u) & 0x3fu);
  let sun: f32 = f32((w1 >> 12u) & 0xfu) / 15.0;
  let hemi: f32 = f32((w1 >> 16u) & 0xfu) / 15.0;
  let palettes: array<u32, 4> = array<u32, 4>(w2 & 0xffffu, w2 >> 16u, w3 & 0xffffu, w3 >> 16u);

  shapes[cell] = vec4<f32>((base + top) * 0.5, (top - base) * 0.5, hemi, sun);

  // `vis.box`, grown by `EPS_L`.
  let min_x: f32 = f32(slot.x) * GRASS_SLOT_METERS - BOX_GROWTH;
  let min_z: f32 = f32(slot.z) * GRASS_SLOT_METERS - BOX_GROWTH;
  let min_y: f32 = base - BOX_GROWTH;
  let max_y: f32 = top + BOX_GROWTH;
  let seed: u32 = SEED ^ bitcast<u32>(slot.x * slot.z);
  var selection: u32 = seed;
  var jitter: u32 = seed;
  var yaw: u32 = seed;
  var scale: u32 = seed;
  let steps: i32 = params.steps;
  let first: u32 = cell * params.per_cell;
  var held: u32 = 0u;

  for (var z: i32 = 0; z <= steps; z++) {
    for (var x: i32 = 0; x <= steps; x++) {
      let shift_x: u32 = random(&jitter) % 16u;
      let shift_z: u32 = random(&jitter) % 16u;
      // `InterpolateAndDither`, clamped to the last step as the engine clamps it.
      let column: u32 = u32(min(x, steps - 1));
      let row: u32 = u32(min(z, steps - 1));
      let threshold: i32 = i32(dither[((column + shift_x) % 16u) * 16u + (row + shift_z) % 16u]);
      var mask: u32 = 0u;
      var selected: u32 = 0u;

      for (var object: u32 = 0u; object < 4u; object++) {
        if (ids[object] != EMPTY_ID && density(palettes[object], column, row) > threshold) {
          mask |= 1u << object;
          selected += 1u;
        }
      }

      if (selected == 0u) {
        continue;
      }

      var pick: u32 = 0u;

      if (selected > 1u) {
        pick = random(&selection) % selected;
      }

      var chosen: u32 = 0u;
      var seen: u32 = 0u;

      for (var object: u32 = 0u; object < 4u; object++) {
        if ((mask & (1u << object)) != 0u) {
          if (seen == pick) {
            chosen = object;
          }

          seen += 1u;
        }
      }

      let model: u32 = ids[chosen];

      if (model >= params.model_count) {
        continue;
      }

      // `Item_P.set(rx + randFs, vMax.y, rz + randFs)`: MSVC evaluates the arguments right to left, so `z` draws first.
      let jitter_z: f32 = random_signed(&jitter, params.jitter);
      let jitter_x: f32 = random_signed(&jitter, params.jitter);
      let px: f32 = f32(x) / f32(steps) * GRASS_SLOT_METERS + min_x + jitter_x;
      let pz: f32 = f32(z) / f32(steps) * GRASS_SLOT_METERS + min_z + jitter_z;
      var y: f32 = min_y - 5.0;
      // The slot is walked in the engine's space and its triangles are in the renderer's, whose z runs the other way.
      let origin: vec3<f32> = vec3<f32>(px, max_y, -pz);

      for (var entry: u32 = bin_start; entry < bin_start + bin_count; entry++) {
        let triangle: u32 = bins[entry];
        let range: f32 = ray_range(origin, triangle_corner(triangle, 0u), triangle_corner(triangle, 1u),
          triangle_corner(triangle, 2u));

        if (range >= 0.0) {
          y = max(y, max_y - range);
        }
      }

      if (y < min_y) {
        continue;
      }

      let shape: vec4<f32> = models[model].shape;
      let least: f32 = shape.x * 0.5;
      // Before the settings' height scales it, which a frame applies, so a height changed plants nothing again.
      let size: f32 = random_float(&scale) * (shape.y * 0.9 - least) + least;
      let turn: f32 = random_float(&yaw) * TURN;
      // The engine picks a waving tuft's wave by its own unseeded generator; this picks it by place, so it holds.
      let wave: f32 = select(1.0, 2.0, abs((x + z * 7 + slot.x * 3 + slot.z * 5) % 3) == 0);

      if (held < params.per_cell) {
        let to: u32 = (first + held) * 2u;

        cached[to] = vec4<f32>(px, y, -pz, turn);
        cached[to + 1u] = vec4<f32>(size, f32(model), wave, 0.0);
        held += 1u;
      }
    }
  }

  keys[key + 3u] = held;
}

@compute @workgroup_size(64)
fn clear_counts(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let index: u32 = compute_index(id, groups, 64u);

  if (index <= params.model_count) {
    atomicStore(&counts[index], 0u);
  }
}

// `UpdateVisibleM`, a thread a cell of the ring: a cell not planted with the slot it holds now, outside the view or past
// `dm_fade` passes over, and each of its tufts is shrunk by the slot's distance, dropped where too small to see, stilled
// where too small to see sway, culled by the view, and appended.
@compute @workgroup_size(64)
fn cull(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let cell: u32 = compute_index(id, groups, 64u);

  if (cell >= ring_cells()) {
    return;
  }

  let slot: RingSlot = ring_slot(cell);
  let held: u32 = keys[cell * GRASS_KEY_WORDS + 3u];

  // A stale cell still waiting on the schedule holds another slot's tufts, or tufts of another density.
  if (held == 0u || !is_current(cell, slot)) {
    return;
  }

  let ground: vec4<f32> = shapes[cell];
  // The slot's distance from the eye fades every tuft in it, by squared metres from one to `dm_fade`.
  let center: vec3<f32> = vec3<f32>(f32(slot.x) * 2.0 + 1.0, ground.x, f32(slot.z) * 2.0 + 1.0);
  let offset: vec3<f32> = params.eye.xyz - center;
  let distance: f32 = dot(offset, offset);
  let fade_limit: f32 = params.fade * params.fade;

  if (distance > fade_limit) {
    return;
  }

  // `testSAABB`: a slot outside the view passes over whole, its sphere grown by the largest tuft.
  let bound: f32 = sqrt(2.0 + ground.y * ground.y) + BOX_GROWTH + params.height * params.tuft_reach;

  if (is_outside(vec3<f32>(center.x, center.y, -center.z), bound)) {
    return;
  }

  let shrink: f32 = 1.0 - select((distance - 1.0) / (fade_limit - 1.0), 0.0, distance < 1.0);
  let first: u32 = cell * params.per_cell;

  for (var tuft: u32 = 0u; tuft < min(held, params.per_cell); tuft++) {
    let source: u32 = (first + tuft) * 2u;
    let placed: vec4<f32> = cached[source];
    let kept: vec4<f32> = cached[source + 1u];
    let model: u32 = u32(kept.y);
    let shape: vec4<f32> = models[model].shape;
    // A tuft's screen area by its slot's distance, against the engine's thresholds.
    let shrunk: f32 = kept.x * params.height * shrink;
    let area: f32 = shrunk * shrunk * shape.z * shape.z / max(distance, 0.0001);

    if (area < params.discard_below) {
      continue;
    }

    let is_waving: bool = models[model].is_waving > 0.5 && area > params.discard_below * 16.0;

    if (is_outside(placed.xyz + vec3<f32>(0.0, shape.w * shrunk * 0.5, 0.0), shape.z * shrunk)) {
      continue;
    }

    let at: u32 = atomicAdd(&counts[params.model_count], 1u);

    if (at >= params.capacity) {
      continue;
    }

    planted[at * 2u] = placed;
    planted[at * 2u + 1u] = vec4<f32>(shrunk, ground.z, ground.w, select(0.0, kept.z, is_waving));
    planted_models[at] = model;
    atomicAdd(&counts[model], 1u);
  }
}

// Lays each model's items out after the last's and writes its indexed draw, the range's start as its first instance.
@compute @workgroup_size(1)
fn arrange() {
  var first: u32 = 0u;

  for (var model: u32 = 0u; model < params.model_count; model++) {
    let count: u32 = atomicLoad(&counts[model]);
    let at: u32 = model * 5u;

    args[at] = models[model].index_count;
    args[at + 1u] = count;
    args[at + 2u] = models[model].first_index;
    args[at + 3u] = 0u;
    args[at + 4u] = first;
    atomicStore(&cursors[model], first);
    first += count;
  }
}

// Sorts the items into each model's range, a thread an item.
@compute @workgroup_size(64)
fn scatter(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
  let item: u32 = compute_index(id, groups, 64u);

  if (item >= min(atomicLoad(&counts[params.model_count]), params.capacity)) {
    return;
  }

  let at: u32 = atomicAdd(&cursors[planted_models[item]], 1u);

  sorted[at * 2u] = planted[item * 2u];
  sorted[at * 2u + 1u] = planted[item * 2u + 1u];
}
