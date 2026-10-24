use xrf_visual::VisualMotionPose;

use crate::core::session::SessionId;

/// A motion baked for an open model, held as an opening of its own by the viewer playing it: what is played is view
/// state, so it lives beside the model rather than in it, and goes with the viewer or its tool.
pub struct PosedVisualMotion {
  /// The model's opening it was baked for.
  pub model: SessionId,
  pub pose: VisualMotionPose,
}
