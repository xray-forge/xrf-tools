use std::time::Duration;

/// What repeated runs of one command said about it.
///
/// Order statistics only. A mean of the durations would be pulled by the one round where a background task woke up, and
/// on these trees the spread between two correct runs reaches ±1000ms — the reason the whole harness exists rather than
/// a stopwatch. The memory figures are medians of per-round values for the same reason; the mean *within* a round is a
/// different thing, and is what says how much a process held for most of its life.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoundStatistics {
  pub median: Duration,
  pub fastest: Duration,
  pub slowest: Duration,
  pub peak_bytes: Option<u64>,
  pub mean_bytes: Option<u64>,
}

impl RoundStatistics {
  /// Summarizes measured rounds, or `None` when no round was timed.
  ///
  /// Memory is absent rather than zero when nothing was sampled, which a command finishing inside one sampling interval
  /// does. Timing is what makes a round a round, so the durations decide whether there is anything to report at all.
  pub fn of(rounds: &[Duration], peaks: &[u64], means: &[u64]) -> Option<Self> {
    if rounds.is_empty() {
      return None;
    }

    let mut sorted: Vec<Duration> = rounds.to_vec();

    sorted.sort_unstable();

    Some(Self {
      median: sorted[(sorted.len() - 1) / 2],
      fastest: sorted[0],
      slowest: sorted[sorted.len() - 1],
      peak_bytes: median_bytes(peaks),
      mean_bytes: median_bytes(means),
    })
  }
}

/// The middle value of a byte measurement taken over several rounds, or `None` when none was taken.
///
/// The same lower-middle rule the durations use, for the same reason: a figure that no round reached is a worse thing to
/// quote than one that a round did.
fn median_bytes(values: &[u64]) -> Option<u64> {
  if values.is_empty() {
    return None;
  }

  let mut sorted: Vec<u64> = values.to_vec();

  sorted.sort_unstable();

  Some(sorted[(sorted.len() - 1) / 2])
}

/// The order rounds are executed in: every binary once per round, rather than every round of one binary.
///
/// Interleaving is the whole protocol. Running all rounds of A and then all of B compares two different machine states
/// — file cache, background load, thermal headroom — and has already produced a 10–16% difference that did not exist.
/// Alternating means a drift over the session lands on every binary rather than on whichever went second.
pub fn interleaved(binaries: usize, rounds: usize) -> impl Iterator<Item = (usize, usize)> {
  (0..rounds).flat_map(move |round| (0..binaries).map(move |binary| (round, binary)))
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::{RoundStatistics, interleaved};

  fn ms(values: &[u64]) -> Vec<Duration> {
    values.iter().copied().map(Duration::from_millis).collect()
  }

  fn timed(values: &[u64]) -> RoundStatistics {
    RoundStatistics::of(&ms(values), &[], &[]).expect("a measured round to summarize")
  }

  #[test]
  fn summarizes_nothing_when_no_round_ran() {
    assert_eq!(RoundStatistics::of(&[], &[100], &[100]), None);
  }

  #[test]
  fn reports_the_middle_round_of_an_odd_count() {
    let statistics: RoundStatistics = timed(&[4802, 4755, 4771, 4788, 4763]);

    assert_eq!(statistics.median, Duration::from_millis(4771));
    assert_eq!(statistics.fastest, Duration::from_millis(4755));
    assert_eq!(statistics.slowest, Duration::from_millis(4802));
  }

  /// An averaged median would answer 4779, which no round produced.
  #[test]
  fn reports_an_observed_round_for_an_even_count() {
    assert_eq!(timed(&[4802, 4755, 4771, 4788]).median, Duration::from_millis(4771));
  }

  #[test]
  fn summarizes_a_single_round_as_itself() {
    let statistics: RoundStatistics = timed(&[4771]);

    assert_eq!(statistics.median, Duration::from_millis(4771));
    assert_eq!(statistics.fastest, Duration::from_millis(4771));
    assert_eq!(statistics.slowest, Duration::from_millis(4771));
  }

  #[test]
  fn reports_no_memory_when_nothing_was_sampled() {
    let statistics: RoundStatistics = timed(&[4771]);

    assert_eq!(statistics.peak_bytes, None);
    assert_eq!(statistics.mean_bytes, None);
  }

  #[test]
  fn reports_an_observed_peak_and_mean_from_the_middle_of_the_rounds() {
    let statistics: RoundStatistics = RoundStatistics::of(
      &ms(&[4771, 4802, 4755]),
      &[907_700_000, 787_800_000, 812_400_000],
      &[604_100_000, 512_300_000, 550_900_000],
    )
    .expect("a measured round to summarize");

    assert_eq!(statistics.peak_bytes, Some(812_400_000));
    assert_eq!(statistics.mean_bytes, Some(550_900_000));
  }

  /// Every binary runs once before any binary runs twice, so a drift over the session lands on all of them.
  #[test]
  fn alternates_binaries_within_each_round() {
    assert_eq!(
      interleaved(2, 3).collect::<Vec<_>>(),
      vec![(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (2, 1)]
    );
  }

  #[test]
  fn measures_one_binary_in_plain_sequence() {
    assert_eq!(interleaved(1, 3).collect::<Vec<_>>(), vec![(0, 0), (1, 0), (2, 0)]);
  }

  #[test]
  fn measures_nothing_when_no_round_was_asked_for() {
    assert_eq!(interleaved(2, 0).count(), 0);
  }
}
