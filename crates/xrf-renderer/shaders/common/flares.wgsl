#import "generated/structs"

// The lens flare as `pass/flare_uniform.rs` writes it, for the pass that measures how much of the sun shows and the one
// that draws the flares over the frame.

// Where the sun's rays are tested from and against: how far apart the four side rays bend, `fScale`.
const FLARE_RAY_SPREAD: f32 = 0.02;

// How fast the flares follow how much of the sun shows, a second: `BLEND_DEC_SPEED`.
const FLARE_BLEND_SPEED: f32 = 4.0;

// Under this a flare is not drawn, `EPS_L`.
const FLARE_EPSILON: f32 = 0.001;
