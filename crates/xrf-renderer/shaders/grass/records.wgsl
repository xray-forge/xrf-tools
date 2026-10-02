// The grass's records, as `pass/grass_uniform.rs` and `scene/level/level_grass.rs` write them.

// Where the camera stands and what the planting is set to, `CDetailManager`'s terms.
struct GrassParams {
  // The eye, in the engine's space: renderer space with `z` negated.
  eye: vec4<f32>,
  // The view's six planes in renderer space, pointing in.
  planes: array<vec4<f32>, 6>,
  // The slot the camera stands over, `iFloor(EYE / dm_slot_size + 0.5)`, on each axis.
  center: vec2<i32>,
  // `dm_size`: slots planted each way from the camera's.
  reach: i32,
  // `d_size`: steps a slot's candidates are laid across, one more candidate than steps each way.
  steps: i32,
  // The grid's size in slots, and the world slot its first cell stands for, negated.
  grid: vec4<i32>,
  // `dm_fade`: metres from the eye at which a slot has shrunk to nothing.
  fade: f32,
  // How far a candidate is jittered off its step, `density / 1.7`.
  jitter: f32,
  // What every tuft is scaled by, `ps_current_detail_height`.
  height: f32,
  // `r_ssaDISCARD`: the screen area below which a tuft is dropped.
  discard_below: f32,
  // Counts what a slot plants changing, so a slot planted under another stands for nothing.
  generation: u32,
  // Tufts a cell of the ring holds, rings of cells around the camera's, items the lists hold, and models.
  per_cell: u32,
  bands: u32,
  capacity: u32,
  model_count: u32,
  // Metres the largest tuft reaches past its ground at a height of one, which a slot's sphere grows by.
  tuft_reach: f32,
  pad0: u32,
  pad1: u32,
};

// One detail model: its least and most scale, radius and height, then whether it waves, its base texture's slot, its
// alpha reference and its first index and base vertex in the grass's arenas.
struct GrassModel {
  shape: vec4<f32>,
  is_waving: f32,
  texture: u32,
  alpha_reference: f32,
  index_count: u32,
  first_index: u32,
  pad0: u32,
  pad1: u32,
  pad2: u32,
};

// Words a cached slot's key takes: the world slot it holds on each axis, the generation it was planted under, its tufts.
const GRASS_KEY_WORDS: u32 = 4u;

// Words a planted slot takes in the level's records: its stored sixteen bytes, then its triangle bin's start and length.
const GRASS_SLOT_WORDS: u32 = 6u;

// Floats a collision triangle takes: its three corners.
const GRASS_TRIANGLE_FLOATS: u32 = 9u;

// Metres a detail slot spans, `dm_slot_size`.
const GRASS_SLOT_METERS: f32 = 2.0;
