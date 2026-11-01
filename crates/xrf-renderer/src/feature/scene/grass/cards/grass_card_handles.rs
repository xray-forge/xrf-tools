use xrf_renderer_core::GraphBuffer;

/// The grass's cards as a frame's graph knows them, once stood this frame: their corners, their triangles' records and
/// the cascades' indirect draw of them.
#[derive(Clone, Copy, Debug)]
pub struct GrassCardHandles {
  pub vertices: GraphBuffer,
  pub triangles: GraphBuffer,
  pub draw: GraphBuffer,
}
