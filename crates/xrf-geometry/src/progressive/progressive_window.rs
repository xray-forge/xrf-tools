/// One level of detail of a progressive mesh, `FSlideWindow`: the run of its index buffer to draw and the vertices
/// that run reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgressiveWindow {
  /// Where the run starts, in indices.
  pub offset: u32,
  pub triangles: u16,
  pub vertices: u16,
}
