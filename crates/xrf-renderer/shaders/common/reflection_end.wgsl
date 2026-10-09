// How a reflection ray ended, which the trace writes beside what it reflected (a reason's number over 255 in an
// eight-bit texel) and the ends debug view colours, one colour a reason. Nothing is traced at a texel left at none.

const END_UNTRACED: u32 = 0u;
// Met a surface where it crossed into it.
const END_MET_CROSSED: u32 = 1u;
// Met a surface it lay behind within the thickness, without crossing it.
const END_MET_THICK: u32 = 2u;
// Spent its crossings behind a surface, passing it a pixel at a time.
const END_SPENT_BEHIND: u32 = 3u;
// Spent its crossings in the clear.
const END_SPENT: u32 = 4u;
// Left the screen.
const END_OFF_SCREEN: u32 = 5u;
// Reached the distance it was traced to, or the far plane.
const END_FAR: u32 = 6u;
// Not walked: its surface too rough, reflecting the environment alone.
const END_ROUGH: u32 = 7u;
// Met what it could not trust: the sky, or a hit faded out at the screen's edge or the distance.
const END_UNTRUSTED: u32 = 8u;

// A reason as the ends texel stores it.
fn reflection_end_value(end: u32) -> f32 {
  return f32(end) / 255.0;
}

// The reason an ends texel stores.
fn reflection_end_code(value: f32) -> u32 {
  return u32(round(value * 255.0));
}

// The ends view's colour for a reason: green met by crossing, yellow met by thickness, red spent behind a surface,
// orange spent in the clear, blue off the screen, cyan at the distance, grey too rough, magenta untrusted, black
// untraced.
fn reflection_end_colour(end: u32) -> vec3<f32> {
  switch (end) {
    case END_MET_CROSSED: {
      return vec3<f32>(0.1, 0.8, 0.2);
    }
    case END_MET_THICK: {
      return vec3<f32>(0.95, 0.85, 0.1);
    }
    case END_SPENT_BEHIND: {
      return vec3<f32>(0.9, 0.1, 0.1);
    }
    case END_SPENT: {
      return vec3<f32>(0.95, 0.5, 0.1);
    }
    case END_OFF_SCREEN: {
      return vec3<f32>(0.15, 0.3, 0.95);
    }
    case END_FAR: {
      return vec3<f32>(0.1, 0.8, 0.9);
    }
    case END_ROUGH: {
      return vec3<f32>(0.45, 0.45, 0.45);
    }
    case END_UNTRUSTED: {
      return vec3<f32>(0.85, 0.2, 0.85);
    }
    default: {
      return vec3<f32>(0.0);
    }
  }
}
