use std::fmt::Display;
use std::mem;
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::{Output, OutputChannel, OutputRecord};

/// Retains every workflow message instead of rendering it.
///
/// Backs the buffering half of [`crate::OutputSequence`], and stands in for a terminal in a test
/// that asserts what a workflow said rather than printing it.
#[derive(Default)]
pub struct RecordingOutput {
  records: Mutex<Vec<OutputRecord>>,
}

impl RecordingOutput {
  /// Everything recorded so far, in the order it arrived.
  pub fn list_records(&self) -> Vec<OutputRecord> {
    self.lock().clone()
  }

  /// Everything recorded so far, leaving the sink empty.
  pub fn take_records(&self) -> Vec<OutputRecord> {
    mem::take(&mut *self.lock())
  }

  /// A panicking writer leaves the recorded messages intact rather than poisoning every later read:
  /// what was said before the panic is exactly what a caller is trying to find out.
  fn lock(&self) -> MutexGuard<'_, Vec<OutputRecord>> {
    self.records.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl Output for RecordingOutput {
  fn write(&self, channel: OutputChannel, message: &dyn Display) {
    self.lock().push(OutputRecord::render(channel, message));
  }
}

#[cfg(test)]
mod tests {
  use super::{Output, RecordingOutput};
  use crate::{OutputChannel, OutputRecord};

  #[test]
  fn retains_messages_in_arrival_order() {
    let output: RecordingOutput = RecordingOutput::default();

    output.write(OutputChannel::Info, &"first");
    output.write(OutputChannel::Error, &"second");

    assert_eq!(
      output.list_records(),
      vec![
        OutputRecord::new(OutputChannel::Info, "first"),
        OutputRecord::new(OutputChannel::Error, "second"),
      ]
    );
  }

  #[test]
  fn empties_itself_when_taken() {
    let output: RecordingOutput = RecordingOutput::default();

    output.write(OutputChannel::Info, &"first");

    assert_eq!(
      output.take_records(),
      vec![OutputRecord::new(OutputChannel::Info, "first")]
    );
    assert!(output.take_records().is_empty());
  }
}
