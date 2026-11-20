use std::path::Path;

use crate::pack::ArchivePackEntryOutcome;
use crate::pack::config::ArchivePackConfig;
use crate::pack::source::{ArchivePackExcludedDirectory, ArchivePackSkippedFile};

/// One decision a packing run made, handed to its listener as it is made.
///
/// Borrowed from the run, so emitting one costs a call and nothing more; a listener that keeps one copies what it
/// needs. A placement or a volume comes as it happens, so a run that stops has already named the volume it was writing
/// and the last entry it reached. The walk's omissions are the exception: the walk reaches them in whatever order the
/// host enumerates, so they come ordered, in one selection, once the selection is settled.
#[derive(Clone, Copy, Debug)]
pub enum ArchivePackEvent<'e> {
  /// The settings the run works with, before it does anything.
  Settings(&'e ArchivePackConfig),
  /// What the walk selected and what the rules left out, before any volume exists.
  ///
  /// The directories are the rows the archive will carry, from registration rather than the spellings the walk
  /// reached. An excluded directory is its own record because the walk never reaches inside a recursively excluded
  /// one, so nothing else could tell a rule that applied from a directory that was not there.
  Selection {
    directories: &'e [String],
    excluded_directories: &'e [ArchivePackExcludedDirectory],
    skipped_files: &'e [ArchivePackSkippedFile],
  },
  /// How one entry landed, with its size as read and as written.
  Entry {
    name: &'e str,
    outcome: ArchivePackEntryOutcome<'e>,
    size_real: u32,
    size_compressed: u32,
  },
  /// A volume was opened. It comes before the file is created: from then on the path exists and has replaced whatever
  /// stood there, so a run that fails inside the volume has already named it.
  VolumeOpened { path: &'e Path },
  /// A volume was closed once its descriptor table was written, at `size` bytes holding `entries` rows.
  VolumeClosed { path: &'e Path, size: u64, entries: usize },
}
