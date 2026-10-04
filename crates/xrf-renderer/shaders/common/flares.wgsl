// The lens flare as `pass/flare_uniform.rs` writes it, for the pass that measures how much of the sun shows and the one
// that draws the flares over the frame.

struct Flares {
  // xyz: towards the sun in view space; w: real seconds since the last frame.
  to_sun: vec4<f32>,
  // xyz: towards the sun in renderer space; w: how far the lens flare has faded in, `m_StateBlend`.
  sun: vec4<f32>,
  // rgb: the sun's colour as a vertex colour holds it; w: how many flares follow.
  color: vec4<f32>,
  // x: the gradient's radius; y: its opacity; z: one where it is drawn.
  gradient: vec4<f32>,
  // Each flare: x where along the line from the screen's centre to the sun it stands, y its radius, z its opacity.
  flares: array<vec4<f32>, 16>,
};

// Where the sun's rays are tested from and against: how far apart the four side rays bend, `fScale`.
const FLARE_RAY_SPREAD: f32 = 0.02;

// How fast the flares follow how much of the sun shows, a second: `BLEND_DEC_SPEED`.
const FLARE_BLEND_SPEED: f32 = 4.0;

// Under this a flare is not drawn, `EPS_L`.
const FLARE_EPSILON: f32 = 0.001;
