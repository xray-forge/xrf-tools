/// What a view hands its units as a frame its graph recorded ends: the depth target it drew into, which a unit keeping
/// last frame's depth trades for its own.
pub struct ViewFrameEnd<'a> {
  pub depth: Option<&'a mut wgpu::TextureView>,
}
