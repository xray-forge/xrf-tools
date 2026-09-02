use std::collections::BTreeMap;
use std::sync::Mutex;

use xrf_job::JobHandle;

use crate::GamedataVerificationType;

/// What a verification sweep is working on right now, while several checks are working at once.
///
/// A sequential sweep could name the one check it was on. A parallel one has no such thing, and naming whichever
/// worker answered last reads as thrashing rather than as progress — so this names the whole set instead, which is the
/// honest answer to the same question and the one a window can render as a line.
///
/// Keyed by each check's position in the selection rather than by when it started, so the line reads in the order the
/// report and the console already use, and does not reshuffle itself every time an unrelated check finishes.
#[derive(Debug, Default)]
pub(crate) struct GamedataCheckSchedule {
  running: Mutex<BTreeMap<usize, GamedataVerificationType>>,
}

impl GamedataCheckSchedule {
  /// Records that the check at `index` has started, and says so.
  pub(crate) fn enter(&self, job: &JobHandle, index: usize, check: GamedataVerificationType) {
    self.publish(job, |running| {
      running.insert(index, check);
    });
  }

  /// Records that the check at `index` has finished, and says so.
  ///
  /// The set empties as the sweep drains, so the last checks still working are named alone rather than beside ones
  /// that finished minutes earlier.
  pub(crate) fn leave(&self, job: &JobHandle, index: usize) {
    self.publish(job, |running| {
      running.remove(&index);
    });
  }

  fn publish<F>(&self, job: &JobHandle, change: F)
  where
    F: FnOnce(&mut BTreeMap<usize, GamedataVerificationType>),
  {
    let mut running = self.running.lock().expect("check schedule is never poisoned");

    change(&mut running);

    job.set_detail((!running.is_empty()).then(|| {
      running
        .values()
        .map(ToString::to_string)
        .collect::<Vec<String>>()
        .join(", ")
    }));
  }
}
