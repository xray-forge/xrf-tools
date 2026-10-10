#import "generated/structs"

// The grass's records, as `pass/grass_uniform.rs` and `scene/level/level_grass.rs` write them; its models'
// (`GrassModel`) as `pass/grass_model_record.rs` declares them.

// Steps a ground normal's `x` and `z` are kept on each, as a tuft's record packs them.
const GROUND_NORMAL_STEPS: u32 = 1024u;

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
  // Radians of slope past which a tuft is not planted, thinning over the last fifth; a right angle for none.
  slope_limit: f32,
  // Metres from the eye within which the cards stand a tuft, the last of them over which it shrinks away, and
  // the slots each way from the eye's that reach spans.
  card_reach: f32,
  card_fade: f32,
  card_slots: i32,
  pad: vec2<u32>,
};

// Words a cached slot's key takes: the world slot it holds on each axis, the generation it was planted under, its tufts.
const GRASS_KEY_WORDS: u32 = 4u;

// Words a planted slot takes in the level's records: its stored sixteen bytes, then its triangle bin's start and length.
const GRASS_SLOT_WORDS: u32 = 6u;

// Floats a collision triangle takes: its three corners.
const GRASS_TRIANGLE_FLOATS: u32 = 9u;

// A slope's limit standing for none, a right angle; the share of it a tuft's limit is drawn down by at most, so the
// grass thins up a bank.
const SLOPE_NONE: f32 = 1.5707;
const SLOPE_THINNING: f32 = 0.2;

// The lighting model's slice of the material table the grass is lit by, the default Blinn's `(1 + 0.5) / 4` as the
// G-buffer packs it: 8 of 31 steps over four thirds.
const GRASS_MATERIAL_SLICE: f32 = 32.0 / 93.0;

// `DetailSlot::ID_Empty`: a slot's mark for a model it does not plant.
const GRASS_EMPTY_ID: u32 = 0x3fu;

// Metres a detail slot spans, `dm_slot_size`.
const GRASS_SLOT_METERS: f32 = 2.0;
