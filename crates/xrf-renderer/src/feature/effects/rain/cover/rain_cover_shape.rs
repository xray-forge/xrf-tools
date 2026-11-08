use crate::scene::level::shadow::overhead_shape::OverheadShape;

/// A rain cover's texels across and the metres it reaches down, as the rain and the wetting read it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RainCoverShape {
  pub texels: f32,
  pub reach: f32,
}

impl RainCoverShape {
  /// The shape of a cover drawn as `shape`.
  pub(crate) fn of(shape: OverheadShape) -> Self {
    Self {
      texels: shape.resolution as f32,
      reach: shape.depth,
    }
  }
}
