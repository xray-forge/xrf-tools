// Which frames a pixel traces on where the traces take turns: its own turn of a list's `turns`, and, under ray
// budgets, which pixel of its quad takes the quad's ray.

// The order a quad's pixels take its rays in, the diagonals first, so two rays fall across each other.
const QUAD_ORDER: array<u32, 4> = array<u32, 4>(0u, 3u, 1u, 2u);

// The turn of `turns` a pixel traces on: its place in an 8 × 8 ordered dither, so each turn takes exactly one in
// `turns` of them, every one within a 4 × 2 block, and no turn passes the list's room.
fn trace_turn_of(cell: vec2<u32>, turns: u32) -> u32 {
  let x: u32 = cell.x & 7u;
  let mixed: u32 = x ^ (cell.y & 7u);
  let place: u32 = ((mixed & 1u) << 5u) | ((x & 1u) << 4u) | ((mixed & 2u) << 2u) | ((x & 2u) << 1u) |
    ((mixed & 4u) >> 1u) | ((x & 4u) >> 2u);

  return place * turns / 64u;
}

// Whether a pixel at `cell` takes its turn of `turns` on `frame` (`trace_turn_of`).
fn is_on_trace_turn(cell: vec2<u32>, turns: u32, frame: u32) -> bool {
  return (trace_turn_of(cell, turns) + frame) % turns == 0u;
}

// Whether a pixel of a quad under ray budgets takes its turn of `turns` on `frame`: the quad's, by its own place in the
// dither, so on that frame whichever of its pixels traces asks the scene, and its quad-mates take the answer, within
// `turns` frames whatever `turns` is, where every pixel of the quad waits the same `turns`. Taken a pixel at a time,
// a pixel's turn would fall on frames its quad gives the ray to another pixel for as long as the frame's step and
// `turns` share their pattern: at 3, 9 or 15 turns, tens to hundreds of frames.
fn is_on_quad_trace_turn(pixel: vec2<u32>, turns: u32, frame: u32) -> bool {
  return is_on_trace_turn(pixel / 2u, turns, frame);
}

// The place in `QUAD_ORDER` a quad's rays start from on `frame`, from the place `picked` its noise picks: the sum of the
// frame's base-4 digits, so each four frames from a multiple of four start once at each place, yet a pixel's traced
// frames fall on every residue of four in turn, and so on every cell of the block it traces (`reflection_texel` turns
// the cell by the frame). Stepped by the frame alone, a one-ray pixel would trace one cell of its block always.
fn quad_turn(frame: u32, picked: u32) -> u32 {
  // The digits summed a pair of bits, a nibble and a byte at a time.
  let pairs: u32 = (frame & 0x33333333u) + ((frame >> 2u) & 0x33333333u);
  let nibbles: u32 = (pairs & 0x0f0f0f0fu) + ((pairs >> 4u) & 0x0f0f0f0fu);
  let digits: u32 = (nibbles * 0x01010101u) >> 24u;

  return (digits + picked) % 4u;
}
