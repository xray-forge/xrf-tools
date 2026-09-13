use serde::Serialize;

use crate::report::archive_measure::ArchiveMeasure;

/// How much of a subject falls in one band of the file-size distribution.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSizeBand {
  /// Smallest size this band admits, inclusive.
  pub from: u64,
  /// Smallest size the next band admits, or `None` for the open-ended top band.
  pub to: Option<u64>,
  pub measure: ArchiveMeasure,
}

impl ArchiveSizeBand {
  /// Lower bound of every band, ascending, the first being zero-length files on their own.
  pub const FLOORS: &'static [u64] = &[
    0,
    1,
    1 << 10,
    1 << 12,
    1 << 14,
    1 << 16,
    1 << 18,
    1 << 20,
    1 << 22,
    1 << 24,
    1 << 26,
  ];

  /// The band a size falls in, as a position in [`Self::FLOORS`].
  pub(crate) fn index_of(size: u64) -> usize {
    let mut band: usize = 0;

    while band + 1 < Self::FLOORS.len() && size >= Self::FLOORS[band + 1] {
      band += 1;
    }

    band
  }

  /// The exclusive upper bound of a band, or `None` for the open-ended top one.
  pub(crate) fn ceiling(band: usize) -> Option<u64> {
    Self::FLOORS.get(band + 1).copied()
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveSizeBand;

  #[test]
  fn an_empty_file_is_not_grouped_with_a_small_one() {
    assert_eq!(ArchiveSizeBand::index_of(0), 0);
    assert_eq!(ArchiveSizeBand::index_of(1), 1);
    assert_eq!(
      ArchiveSizeBand::ceiling(0),
      Some(1),
      "the empty band admits nothing but zero"
    );
  }

  #[test]
  fn a_size_falls_in_the_band_its_floor_opens() {
    assert_eq!(ArchiveSizeBand::index_of(1023), 1, "still under a kilobyte");
    assert_eq!(
      ArchiveSizeBand::index_of(1024),
      2,
      "and the next band opens exactly at one"
    );
    assert_eq!(ArchiveSizeBand::index_of(4095), 2);
    assert_eq!(ArchiveSizeBand::index_of(4096), 3);
  }

  #[test]
  fn the_top_band_is_open_ended() {
    let top: usize = ArchiveSizeBand::FLOORS.len() - 1;

    assert_eq!(
      ArchiveSizeBand::index_of(u64::MAX),
      top,
      "nothing falls past the last floor"
    );
    assert_eq!(ArchiveSizeBand::ceiling(top), None, "and it has no ceiling to report");
  }
}
