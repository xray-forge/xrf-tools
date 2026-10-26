//! What the rules left out of one packing run, and how much of it is worth keeping.

use crate::pack::source::ArchivePackSkipReason;
use crate::pack::source::{ArchivePackExcludedDirectory, ArchivePackSkippedFile};

/// Everything one walk rejected, counted always and named only for a run that is going to say them.
#[derive(Debug, Default)]
pub(crate) struct ArchivePackOmissions {
  is_recording: bool,
  count: usize,
  files: Vec<ArchivePackSkippedFile>,
  directories: Vec<ArchivePackExcludedDirectory>,
}

impl ArchivePackOmissions {
  pub(crate) fn new(is_recording: bool) -> Self {
    Self {
      is_recording,
      ..Self::default()
    }
  }

  /// A second collector under the same policy, for a caller that cannot reach this one while it is walking.
  pub(crate) fn new_sibling(&self) -> Self {
    Self::new(self.is_recording)
  }

  /// Count a file the rules rejected, keeping its name only if it will be said.
  pub(crate) fn record_file(&mut self, name: String, reason: ArchivePackSkipReason) {
    self.count += 1;

    if self.is_recording {
      self.files.push(ArchivePackSkippedFile { name, reason });
    }
  }

  /// Record a directory the rules rejected, and whether the rule covered everything below it.
  pub(crate) fn record_directory(&mut self, name: String, is_recursive: bool) {
    if self.is_recording {
      self
        .directories
        .push(ArchivePackExcludedDirectory { name, is_recursive });
    }
  }

  /// Take everything a sibling collected.
  pub(crate) fn absorb(&mut self, other: Self) {
    self.count += other.count;
    self.files.extend(other.files);
    self.directories.extend(other.directories);
  }

  /// Close the collection: order both groups by name, so two runs over one tree say the same things in the same order.
  pub(crate) fn finish(mut self) -> Self {
    self.files.sort_unstable_by(|left, right| left.name.cmp(&right.name));
    self.files.dedup_by(|later, earlier| later.name == earlier.name);

    self
      .directories
      .sort_unstable_by(|left, right| left.name.cmp(&right.name));
    self.directories.dedup_by(|later, earlier| later.name == earlier.name);

    self
  }

  /// How many files the rules rejected, which is reported whether or not their names were kept.
  pub(crate) const fn get_count(&self) -> usize {
    self.count
  }

  pub(crate) fn get_files(&self) -> &[ArchivePackSkippedFile] {
    &self.files
  }

  pub(crate) fn get_directories(&self) -> &[ArchivePackExcludedDirectory] {
    &self.directories
  }
}

#[cfg(test)]
mod tests {
  use super::ArchivePackOmissions;
  use crate::pack::source::ArchivePackSkipReason;

  fn omissions(is_recording: bool) -> ArchivePackOmissions {
    let mut omitted: ArchivePackOmissions = ArchivePackOmissions::new(is_recording);

    omitted.record_file(String::from("readme.txt"), ArchivePackSkipReason::SkipList);
    omitted.record_file(String::from("notes.md"), ArchivePackSkipReason::ExcludedExtension);
    omitted.record_directory(String::from("text"), false);

    let mut pruned: ArchivePackOmissions = omitted.new_sibling();

    pruned.record_directory(String::from("misc"), true);
    // The same directory reached again through a second included root.
    pruned.record_directory(String::from("misc"), true);
    omitted.absorb(pruned);

    omitted.finish()
  }

  #[test]
  fn a_quiet_run_counts_what_it_rejected_and_keeps_no_name() {
    let omitted: ArchivePackOmissions = omissions(false);

    assert_eq!(omitted.get_count(), 2);
    assert!(omitted.get_files().is_empty());
    assert!(omitted.get_directories().is_empty());
  }

  #[test]
  fn a_recorded_run_orders_both_groups_by_name_and_says_each_once() {
    let omitted: ArchivePackOmissions = omissions(true);

    assert_eq!(omitted.get_count(), 2);
    assert_eq!(
      omitted.get_files().iter().map(|file| &file.name).collect::<Vec<_>>(),
      ["notes.md", "readme.txt"]
    );
    // Whichever root reached `misc` first, it is one line, and the two kinds order together.
    assert_eq!(
      omitted
        .get_directories()
        .iter()
        .map(|directory| (directory.name.as_str(), directory.is_recursive))
        .collect::<Vec<_>>(),
      [("misc", true), ("text", false)]
    );
  }
}
