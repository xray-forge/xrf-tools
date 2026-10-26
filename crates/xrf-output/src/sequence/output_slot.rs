use std::sync::Arc;

use crate::{OutputOptions, OutputSequence, RecordingOutput};

/// One unit of work's reserved position in an [`OutputSequence`].
///
/// Everything logged through [`OutputSlot::get_output`] is held until the slot is dropped and every
/// earlier position has been released. Dropping is what commits, so a unit of work that returns
/// early, fails, or panics still takes its turn rather than stalling the positions behind it.
pub struct OutputSlot<'sequence> {
  pub(crate) sequence: &'sequence OutputSequence,
  pub(crate) index: usize,
  pub(crate) output: OutputOptions,
  pub(crate) recorder: Arc<RecordingOutput>,
}

impl OutputSlot<'_> {
  /// Where this position's work logs, at the sequence's own verbosity.
  pub fn get_output(&self) -> &OutputOptions {
    &self.output
  }
}

impl Drop for OutputSlot<'_> {
  fn drop(&mut self) {
    self.sequence.commit(self.index, self.recorder.take_records());
  }
}
