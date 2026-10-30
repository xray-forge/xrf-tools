// The views a baked impostor is drawn from and read in: the upper hemisphere laid out hemi-octahedrally on a square of
// `IMPOSTOR_FRAMES` × `IMPOSTOR_FRAMES` frames, each frame's direction at a point of the grid, so the edge frames look
// along the horizon and the middle one straight down (Brucks's octahedral impostors, the hemisphere Epic's Impostor
// Baker uses for ground-bound foliage). A direction between frames is read from the three around it, blended as the
// triangle of the grid it falls in weighs them. Directions are renderer space's, `+y` up; `ImpostorFrames` mirrors
// this in Rust for the bake's cameras.

const IMPOSTOR_FRAMES: u32 = 12u;

// The three frames a direction is read from and how much each weighs, summing to one.
struct ImpostorBlend {
  first: vec2<u32>,
  second: vec2<u32>,
  third: vec2<u32>,
  weights: vec3<f32>,
};

// Where a direction of the upper hemisphere lands on the square, `[0, 1]` each way: one from below is taken to the
// horizon.
fn hemi_octahedral_encode(direction: vec3<f32>) -> vec2<f32> {
  let upper: vec3<f32> = vec3<f32>(direction.x, max(direction.y, 0.0), direction.z);
  let folded: vec2<f32> = upper.xz / max(abs(upper.x) + upper.y + abs(upper.z), 1e-6);

  return vec2<f32>(folded.x + folded.y, folded.x - folded.y) * 0.5 + 0.5;
}

// The direction a point of the square stands for.
fn hemi_octahedral_decode(uv: vec2<f32>) -> vec3<f32> {
  let grid: vec2<f32> = uv * 2.0 - 1.0;
  let folded: vec2<f32> = vec2<f32>(grid.x + grid.y, grid.x - grid.y) * 0.5;

  return normalize(vec3<f32>(folded.x, 1.0 - abs(folded.x) - abs(folded.y), folded.y));
}

// The direction a frame was baked from.
fn impostor_frame_direction(frame: vec2<u32>) -> vec3<f32> {
  return hemi_octahedral_decode(vec2<f32>(frame) / f32(IMPOSTOR_FRAMES - 1u));
}

// A frame's right and up as its camera saw it, looking back along its direction: up the world's up laid flat across the
// view, or `-z` for the frame looking straight down.
fn impostor_frame_basis(direction: vec3<f32>) -> mat2x3<f32> {
  let hint: vec3<f32> = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, -1.0), abs(direction.y) > 0.999);
  let right: vec3<f32> = normalize(cross(hint, direction));

  return mat2x3<f32>(right, cross(direction, right));
}

// The three frames a direction falls between and their weights: the grid's cell it lands in, split along its diagonal.
fn impostor_frames(direction: vec3<f32>) -> ImpostorBlend {
  let at: vec2<f32> = hemi_octahedral_encode(direction) * f32(IMPOSTOR_FRAMES - 1u);
  let cell: vec2<f32> = min(floor(at), vec2<f32>(f32(IMPOSTOR_FRAMES - 2u)));
  let within: vec2<f32> = at - cell;
  let base: vec2<u32> = vec2<u32>(cell);

  if (within.x + within.y < 1.0) {
    return ImpostorBlend(base, base + vec2<u32>(1u, 0u), base + vec2<u32>(0u, 1u),
      vec3<f32>(1.0 - within.x - within.y, within.x, within.y));
  }

  return ImpostorBlend(base + vec2<u32>(1u, 1u), base + vec2<u32>(0u, 1u), base + vec2<u32>(1u, 0u),
    vec3<f32>(within.x + within.y - 1.0, 1.0 - within.x, 1.0 - within.y));
}
