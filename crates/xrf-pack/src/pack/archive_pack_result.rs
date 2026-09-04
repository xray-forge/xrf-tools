use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Serialize;
use xrf_job::JobOutcome;

use crate::pack::ArchivePackEntryOutcome;

/// What one packing run produced.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePackResult {
  /// Volumes written, in mount order.
  ///
  /// A volume appears here once it has been closed. On a forced run that stopped early this is the part of the set
  /// that is structurally complete — not the part that is usable, since a set missing its later volumes is missing
  /// entries. On any other run that did not finish it is empty, because such a run publishes nothing.
  pub volumes: Vec<PathBuf>,
  /// Every volume path this run created, closed or not.
  ///
  /// Wider than `volumes` on purpose. A volume is opened with `File::create`, so it exists — and has replaced whatever
  /// stood at that path — from the moment writing begins.
  ///
  /// Empty on a run that did not finish and was not forced: such a run began over a destination holding no volume of
  /// its set, so every file it made was its own and was removed again. A forced run is where this earns its place —
  /// there the same paths may have held a working set beforehand, deleting them would compound the loss, and the
  /// caller needs the list to say what is now on disk.
  pub volumes_opened: Vec<PathBuf>,
  /// Whether the run reached the end of its work or was stopped between entries.
  ///
  /// A cancelled pack publishes nothing and leaves the destination as it found it, unless it was forced — see
  /// `volumes_opened` for what a forced run leaves behind.
  pub outcome: JobOutcome,
  pub files_total: usize,
  /// Files the include, exclude, and skip rules left out.
  pub files_skipped: usize,
  pub files_stored: usize,
  pub files_compressed: usize,
  /// Files that shared an identical earlier payload and cost only a descriptor row.
  pub files_aliased: usize,
  /// Bytes of every selected source file, the data the run had to read.
  pub size_source: u64,
  /// Bytes of every closed volume, headers and descriptor tables included.
  pub size_written: u64,
  /// Everything the run took, which the three phase durations below divide between them.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  /// The share of `duration` spent walking the source tree, before anything was created.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub collect_duration: Duration,
  /// The share of `duration` spent reading, compressing, and placing the selected entries.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub write_duration: Duration,
  /// The share of `duration` spent closing the last volume and naming the set.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub finalize_duration: Duration,
  /// Source bytes per second over the whole run, so a reader compares two runs without dividing.
  ///
  /// Zero where the run took no measurable time, rather than a division a caller has to guard.
  pub speed: u64,
}

impl ArchivePackResult {
  /// Count one entry under the heading the volume would put it in.
  ///
  /// The one place the counts are decided, so `files_stored` means the same thing to a reader of the summary and to a
  /// reader of the transcript: a reverted or an empty entry is stored, whatever the line about it says it is.
  pub(crate) fn record_outcome(&mut self, outcome: ArchivePackEntryOutcome) {
    match outcome {
      ArchivePackEntryOutcome::Compressed => self.files_compressed += 1,
      ArchivePackEntryOutcome::Stored | ArchivePackEntryOutcome::Reverted | ArchivePackEntryOutcome::Empty => {
        self.files_stored += 1;
      }
      ArchivePackEntryOutcome::Aliased { .. } => self.files_aliased += 1,
    }
  }

  /// Close the clock on a run: how long it took, where that time went, and how fast it made the run.
  ///
  /// One method for all of it because each part is a function of the duration, and a result carrying one without the
  /// rest would answer the same question several ways.
  ///
  /// The marks are elapsed times taken as their phase ended, not instants, so a run that stopped simply never took
  /// the ones past where it stopped.
  pub(crate) fn measure(&mut self, started_at: Instant, collected_at: Duration, written_at: Duration) {
    self.duration = started_at.elapsed();

    let [collecting, writing, finalizing] = Self::split_phases(self.duration, collected_at, written_at);

    self.collect_duration = collecting;
    self.write_duration = writing;
    self.finalize_duration = finalizing;
    self.speed = Self::speed_of(self.size_source, self.duration);
  }

  /// The three phases of a run, in the order they happened.
  fn split_phases(duration: Duration, collected_at: Duration, written_at: Duration) -> [Duration; 3] {
    let collected_at: Duration = collected_at.min(duration);
    let written_at: Duration = written_at.clamp(collected_at, duration);

    [collected_at, written_at - collected_at, duration - written_at]
  }

  /// Bytes per second, saturating rather than wrapping and answering zero for a run too fast to time.
  fn speed_of(bytes: u64, duration: Duration) -> u64 {
    let seconds: f64 = duration.as_secs_f64();

    if seconds <= 0.0 {
      return 0;
    }

    // `as` saturates on overflow and truncates the fraction, both of which are the wanted rounding here.
    (bytes as f64 / seconds) as u64
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::ArchivePackResult;
  use crate::pack::ArchivePackEntryOutcome;

  #[test]
  fn measures_speed_as_source_bytes_per_second() {
    assert_eq!(ArchivePackResult::speed_of(2048, Duration::from_millis(500)), 4096);
    assert_eq!(ArchivePackResult::speed_of(2048, Duration::ZERO), 0);
  }

  #[test]
  fn splits_a_finished_run_into_phases_that_tile_it() {
    let [collecting, writing, finalizing] = ArchivePackResult::split_phases(
      Duration::from_millis(700),
      Duration::from_millis(40),
      Duration::from_millis(660),
    );

    assert_eq!(collecting, Duration::from_millis(40));
    assert_eq!(writing, Duration::from_millis(620));
    assert_eq!(finalizing, Duration::from_millis(40));
    assert_eq!(collecting + writing + finalizing, Duration::from_millis(700));
  }

  #[test]
  fn folds_a_stopped_run_into_the_phase_it_reached() {
    // Cancelled inside the walk: the marks past it were never taken, so they read as the run's own end.
    let [collecting, writing, finalizing] = ArchivePackResult::split_phases(
      Duration::from_millis(50),
      Duration::from_millis(90),
      Duration::from_millis(90),
    );

    assert_eq!(collecting, Duration::from_millis(50));
    assert_eq!(writing, Duration::ZERO);
    assert_eq!(finalizing, Duration::ZERO);
  }

  #[test]
  fn counts_every_outcome_the_volume_stored_as_stored() {
    let mut result: ArchivePackResult = ArchivePackResult::default();

    for outcome in [
      ArchivePackEntryOutcome::Compressed,
      ArchivePackEntryOutcome::Stored,
      ArchivePackEntryOutcome::Reverted,
      ArchivePackEntryOutcome::Empty,
      ArchivePackEntryOutcome::Aliased {
        source: "configs\\a.ltx",
      },
    ] {
      result.record_outcome(outcome);
    }

    // Three transcript lines, one heading: what the reader finds in the volume is a payload written as read.
    assert_eq!(result.files_stored, 3);
    assert_eq!(result.files_compressed, 1);
    assert_eq!(result.files_aliased, 1);
  }
}
