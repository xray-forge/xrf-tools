// A quad drawn as two triangles of six vertices: its corners from minus one to one across and down, and, for a quad of
// four vertices of its own, which of them each draws (`QuadIB`).

const QUAD_CORNERS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
  vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
  vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
);

const QUAD_INDICES: array<u32, 6> = array<u32, 6>(0u, 1u, 2u, 3u, 2u, 1u);
