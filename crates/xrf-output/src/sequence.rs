use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::{OutputOptions, OutputRecord, RecordingOutput};

/// Releases the messages of parallel work in the order the work was listed in.
///
/// Work spread across threads finishes in whatever order the threads happen to reach the end, so a
/// worker that logs directly makes the run's output depend on timing: two runs of one binary over
/// one tree say the same things in a different order, and neither can be compared with the other.
///
/// A sequence gives each unit of work a [`OutputSlot`] holding its position. Messages sent through a
/// slot are held until every earlier position has been released, then written in position order.
/// Ordering therefore comes from the input rather than from how the work was scheduled, while a run
/// whose early positions finish first still prints them immediately instead of waiting for the
/// slowest worker.
///
/// ```
/// # use xrf_output::{OutputOptions, OutputSequence, OutputSlot};
/// # let output: OutputOptions = OutputOptions::default();
/// # let paths: Vec<&str> = vec!["a.ogf", "b.ogf"];
/// let sequence: OutputSequence = OutputSequence::new(&output, paths.len());
///
/// // Whichever order these finish in, the messages leave in path order.
/// for (index, path) in paths.iter().enumerate() {
///   let slot: OutputSlot = sequence.new_slot(index);
///
///   xrf_output::verbose!(slot.get_output(), "Verify mesh: {path}");
/// }
/// ```
pub struct OutputSequence {
  sink: OutputOptions,
  state: Mutex<SequenceState>,
}

struct SequenceState {
  /// The position to release next; everything before it has already been written.
  next: usize,
  /// Messages of positions that have finished but cannot be written yet, by position.
  pending: Vec<Option<Vec<OutputRecord>>>,
}

impl OutputSequence {
  /// Orders the messages of `length` units of work into `sink`.
  pub fn new(sink: &OutputOptions, length: usize) -> Self {
    Self {
      sink: sink.clone(),
      state: Mutex::new(SequenceState {
        next: 0,
        pending: (0..length).map(|_| None).collect(),
      }),
    }
  }

  /// Reserves the position `index` in the sequence.
  ///
  /// Take it as the first thing a unit of work does, before anything that can return early: a
  /// position nobody takes holds back every later one until the sequence itself is dropped.
  pub fn new_slot(&self, index: usize) -> OutputSlot<'_> {
    let recorder: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());

    OutputSlot {
      output: self.sink.with_output(recorder.clone()),
      recorder,
      sequence: self,
      index,
    }
  }

  fn commit(&self, index: usize, records: Vec<OutputRecord>) {
    let mut state: MutexGuard<'_, SequenceState> = self.lock();

    if index >= state.pending.len() {
      state.pending.resize_with(index + 1, || None);
    }

    state.pending[index] = Some(records);

    self.release(&mut state);
  }

  /// Writes each position from `next` that has arrived, stopping at the first that has not.
  ///
  /// Writing happens under the lock intentionally. Releasing outside it would let two threads that
  /// each completed a run interleave their writes, which is the one thing a sequence exists to
  /// prevent.
  fn release(&self, state: &mut SequenceState) {
    while state.next < state.pending.len() {
      let Some(records) = state.pending[state.next].take() else {
        break;
      };

      state.next += 1;

      self.write_all(&records);
    }
  }

  fn write_all(&self, records: &[OutputRecord]) {
    for record in records {
      self.sink.write_record(record);
    }
  }

  /// A panicking worker leaves the sequence usable rather than poisoning it: its own position is
  /// committed as it unwinds, and every later position must still be able to release.
  fn lock(&self) -> MutexGuard<'_, SequenceState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl Drop for OutputSequence {
  /// Writes whatever is still held, in position order.
  ///
  /// Only a position whose slot was never taken can still be held here, and it would otherwise keep
  /// every later position from ever being written. Releasing on the way out makes a missed slot cost
  /// its messages their place in the order rather than their existence.
  fn drop(&mut self) {
    let mut state: MutexGuard<'_, SequenceState> = self.lock();

    while state.next < state.pending.len() {
      let position: usize = state.next;

      if let Some(records) = state.pending[position].take() {
        self.write_all(&records);
      }

      state.next += 1;
    }
  }
}

/// One unit of work's reserved position in an [`OutputSequence`].
///
/// Everything logged through [`OutputSlot::get_output`] is held until the slot is dropped and every
/// earlier position has been released. Dropping is what commits, so a unit of work that returns
/// early, fails, or panics still takes its turn rather than stalling the positions behind it.
pub struct OutputSlot<'sequence> {
  sequence: &'sequence OutputSequence,
  index: usize,
  output: OutputOptions,
  recorder: Arc<RecordingOutput>,
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

#[cfg(test)]
mod tests {
  use std::panic::{self, AssertUnwindSafe};
  use std::sync::Arc;
  use std::thread;
  use std::time::Duration;

  use super::{OutputSequence, OutputSlot};
  use crate::{OutputOptions, OutputVerbosity, RecordingOutput};

  fn recording() -> (Arc<RecordingOutput>, OutputOptions) {
    let output: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
    let options: OutputOptions = OutputOptions::new(output.clone(), OutputVerbosity::Verbose);

    (output, options)
  }

  fn messages(output: &RecordingOutput) -> Vec<String> {
    output
      .list_records()
      .into_iter()
      .map(|record| record.get_message().to_string())
      .collect()
  }

  #[test]
  fn releases_positions_in_order_whatever_order_they_finish_in() {
    let (recorded, options) = recording();

    {
      let sequence: OutputSequence = OutputSequence::new(&options, 4);
      let slots: Vec<OutputSlot> = (0..4).map(|index| sequence.new_slot(index)).collect();

      for (index, slot) in slots.iter().enumerate() {
        crate::info!(slot.get_output(), "position {index}");
      }

      // Finish them in an order no scheduler would have produced.
      let mut slots: Vec<Option<OutputSlot>> = slots.into_iter().map(Some).collect();

      for index in [3, 1, 0, 2] {
        drop(slots[index].take());
      }
    }

    assert_eq!(
      messages(&recorded),
      vec![
        String::from("position 0"),
        String::from("position 1"),
        String::from("position 2"),
        String::from("position 3"),
      ]
    );
  }

  #[test]
  fn holds_later_positions_until_an_earlier_one_arrives() {
    let (recorded, options) = recording();
    let sequence: OutputSequence = OutputSequence::new(&options, 3);

    let first: OutputSlot = sequence.new_slot(0);
    crate::info!(first.get_output(), "first");

    {
      let third: OutputSlot = sequence.new_slot(2);
      crate::info!(third.get_output(), "third");

      let second: OutputSlot = sequence.new_slot(1);
      crate::info!(second.get_output(), "second");
    }

    // Position 0 is still open, so nothing behind it may be written yet.
    assert!(messages(&recorded).is_empty());

    drop(first);

    assert_eq!(
      messages(&recorded),
      vec![String::from("first"), String::from("second"), String::from("third")]
    );
  }

  #[test]
  fn orders_positions_finished_on_their_own_threads() {
    let (recorded, options) = recording();

    {
      let sequence: OutputSequence = OutputSequence::new(&options, 16);

      thread::scope(|scope| {
        for index in 0..16 {
          let sequence: &OutputSequence = &sequence;

          scope.spawn(move || {
            let slot: OutputSlot = sequence.new_slot(index);

            crate::info!(slot.get_output(), "position {index}");

            // Later positions intentionally finish first.
            thread::sleep(Duration::from_millis((16 - index as u64) * 2));
          });
        }
      });
    }

    assert_eq!(
      messages(&recorded),
      (0..16).map(|index| format!("position {index}")).collect::<Vec<_>>()
    );
  }

  #[test]
  fn a_position_that_says_nothing_still_lets_the_next_one_through() {
    let (recorded, options) = recording();
    let sequence: OutputSequence = OutputSequence::new(&options, 2);

    drop(sequence.new_slot(0));

    {
      let second: OutputSlot = sequence.new_slot(1);
      crate::info!(second.get_output(), "second");
    }

    assert_eq!(messages(&recorded), vec![String::from("second")]);
  }

  #[test]
  fn a_panicking_position_keeps_what_it_said_and_frees_the_rest() {
    let (recorded, options) = recording();
    let sequence: OutputSequence = OutputSequence::new(&options, 2);

    let panicked: thread::Result<()> = panic::catch_unwind(AssertUnwindSafe(|| {
      let slot: OutputSlot = sequence.new_slot(0);

      crate::error!(slot.get_output(), "said before failing");

      panic!("worker failed");
    }));

    assert!(panicked.is_err());

    {
      let second: OutputSlot = sequence.new_slot(1);
      crate::info!(second.get_output(), "second");
    }

    assert_eq!(
      messages(&recorded),
      vec![String::from("said before failing"), String::from("second")]
    );
  }

  #[test]
  fn writes_what_a_missed_position_held_back_when_the_sequence_ends() {
    let (recorded, options) = recording();

    {
      let sequence: OutputSequence = OutputSequence::new(&options, 2);
      // Position 0 is never taken, so position 1 cannot be released while the sequence lives.
      let second: OutputSlot = sequence.new_slot(1);

      crate::info!(second.get_output(), "second");

      drop(second);

      assert!(messages(&recorded).is_empty());
    }

    assert_eq!(messages(&recorded), vec![String::from("second")]);
  }

  #[test]
  fn holds_nothing_a_muted_verbosity_would_not_have_printed() {
    let output: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
    let options: OutputOptions = OutputOptions::new(output.clone(), OutputVerbosity::Silent);

    {
      let sequence: OutputSequence = OutputSequence::new(&options, 1);
      let slot: OutputSlot = sequence.new_slot(0);

      crate::verbose!(slot.get_output(), "detail");
      crate::error!(slot.get_output(), "failure");
    }

    assert_eq!(messages(&output), vec![String::from("failure")]);
  }
}
