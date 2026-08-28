use std::time::Duration;

/// What repeated runs of one command said about it.
///
/// Order statistics only. A mean would be pulled by the one round where a background task woke up, and on these trees
/// the spread between two correct runs reaches ±1000ms — the reason the whole harness exists rather than a stopwatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoundStatistics {
  pub median: Duration,
  pub fastest: Duration,
  pub slowest: Duration,
}

impl RoundStatistics {
  /// Summarizes measured rounds, or `None` when nothing was measured.
  ///
  /// The median of an even count is the lower of the two middle rounds rather than their average: a report is read as
  /// evidence, and a figure that no round actually produced is a worse thing to quote than one that did.
  pub fn of(rounds: &[Duration]) -> Option<Self> {
    if rounds.is_empty() {
      return None;
    }

    let mut sorted: Vec<Duration> = rounds.to_vec();

    sorted.sort_unstable();

    Some(Self {
      median: sorted[(sorted.len() - 1) / 2],
      fastest: sorted[0],
      slowest: sorted[sorted.len() - 1],
    })
  }
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

  #[test]
  fn summarizes_nothing_when_no_round_ran() {
    assert_eq!(RoundStatistics::of(&[]), None);
  }

  #[test]
  fn reports_the_middle_round_of_an_odd_count() {
    let statistics: RoundStatistics = RoundStatistics::of(&ms(&[4802, 4755, 4771, 4788, 4763])).unwrap();

    assert_eq!(statistics.median, Duration::from_millis(4771));
    assert_eq!(statistics.fastest, Duration::from_millis(4755));
    assert_eq!(statistics.slowest, Duration::from_millis(4802));
  }

  /// An averaged median would answer 4779, which no round produced.
  #[test]
  fn reports_an_observed_round_for_an_even_count() {
    let statistics: RoundStatistics = RoundStatistics::of(&ms(&[4802, 4755, 4771, 4788])).unwrap();

    assert_eq!(statistics.median, Duration::from_millis(4771));
  }

  #[test]
  fn summarizes_a_single_round_as_itself() {
    let statistics: RoundStatistics = RoundStatistics::of(&ms(&[4771])).unwrap();

    assert_eq!(statistics.median, Duration::from_millis(4771));
    assert_eq!(statistics.fastest, Duration::from_millis(4771));
    assert_eq!(statistics.slowest, Duration::from_millis(4771));
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
