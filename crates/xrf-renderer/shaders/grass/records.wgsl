#import "generated/structs"

// The grass's records, as `scene/level/grass_level.rs` writes them; what the planting is set to (`GrassParams`), as
// `pass/grass_uniform.rs` declares it, and its models' (`GrassModel`) as `pass/grass_model_record.rs` does.

// Steps a ground normal's `x` and `z` are kept on each, as a tuft's record packs them.
const GROUND_NORMAL_STEPS: u32 = 1024u;

// Words a cached slot's key takes: the world slot it holds on each axis, the generation it was planted under, its
// tufts.
const GRASS_KEY_WORDS: u32 = 4u;

// Words a planted slot takes in the level's records: its stored sixteen bytes, then its triangle bin's start and
// length.
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
