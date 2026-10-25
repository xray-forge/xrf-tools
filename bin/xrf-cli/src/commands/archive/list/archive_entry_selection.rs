use clap::ArgMatches;
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};

/// The name-table records to include in a listing.
#[derive(Clone, Copy)]
pub(crate) enum ArchiveEntrySelection {
  All,
  Files,
  Directories,
}

impl ArchiveEntrySelection {
  /// What `--files` and `--directories` asked for, as `list` and `find` declare them: every record when neither.
  pub(crate) fn of(matches: &ArgMatches) -> Self {
    if matches.get_flag("files") {
      Self::Files
    } else if matches.get_flag("directories") {
      Self::Directories
    } else {
      Self::All
    }
  }

  /// The records of `project` this selection keeps, ordered by name.
  pub(crate) fn select(self, project: &ArchiveProject) -> Vec<&ArchiveFileDescriptor> {
    let mut entries: Vec<&ArchiveFileDescriptor> = project
      .files
      .values()
      .filter(|entry| match self {
        Self::All => true,
        Self::Files => !entry.is_directory,
        Self::Directories => entry.is_directory,
      })
      .collect();

    entries.sort_unstable_by(|left, right| left.name.cmp(&right.name));
    entries
  }
}
