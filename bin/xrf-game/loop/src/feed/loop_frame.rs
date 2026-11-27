use crate::state::loop_blend::LoopBlend;

/// What presentation reads after a tick: the tick and the game state's own frame. The one way state leaves the
/// simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopFrame<F> {
  pub tick: u64,
  pub state: F,
}

impl<F: LoopBlend> LoopBlend for LoopFrame<F> {
  /// The tick is the next frame's once past this one.
  fn blend(&self, next: &Self, alpha: f32) -> Self {
    Self {
      tick: if alpha > 0.0 { next.tick } else { self.tick },
      state: self.state.blend(&next.state, alpha),
    }
  }
}
