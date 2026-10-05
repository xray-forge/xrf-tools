use crate::graph::timing::GraphTimer;

/// Lets an encoder pass time stages of its own, as a bridge recording several passes does: each `mark` ends a stage
/// named for it. A pass that marks is timed by its marks alone.
pub struct PassMarker<'c> {
  pub(crate) timer: Option<&'c mut GraphTimer>,
  pub(crate) is_marked: bool,
}

impl PassMarker<'_> {
  /// Ends the stage `name`, where the frame is timed.
  pub fn mark(&mut self, encoder: &mut wgpu::CommandEncoder, name: &'static str) {
    if let Some(timer) = self.timer.as_deref_mut() {
      timer.stamp(encoder, Some(name.to_string()));
      self.is_marked = true;
    }
  }
}
