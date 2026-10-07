use crate::graph::timing::GraphTimer;

/// Lets an encoder pass time stages of its own, as a bridge recording several passes does: each `mark` ends a stage
/// named for it. A pass that marks is timed by its marks alone.
pub struct PassMarker<'c> {
  pub(crate) timer: Option<&'c mut GraphTimer>,
  /// Whose the pass is, which its stages' costs are summed by.
  pub(crate) owner: u32,
  pub(crate) is_marked: bool,
}

impl PassMarker<'_> {
  /// Ends the stage `name`, where the frame is timed.
  pub fn mark(&mut self, encoder: &mut wgpu::CommandEncoder, name: &'static str) {
    if let Some(timer) = self.timer.as_deref_mut() {
      timer.stamp(encoder, Some((self.owner, name.to_string())));
      self.is_marked = true;
    }
  }
}
