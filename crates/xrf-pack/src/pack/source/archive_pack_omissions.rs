//! What the rules left out of one packing run, and how much of it is worth keeping.
//!
//! The walk finds omissions in whatever order the host enumerates its directories, while a transcript has to read the
//! same on every machine, so these are collected and ordered rather than said where they are found. Nothing written
//! depends on them: this is the one part of a run that exists only to be reported.

use crate::pack::ArchivePackSkipReason;

/// One file the rules left out, by full engine name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchivePackSkippedFile {
  pub(crate) name: String,
  pub(crate) reason: ArchivePackSkipReason,
}

/// One directory the rules left out, so a run can say so without the walk having read below it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchivePackExcludedDirectory {
  pub(crate) name: String,
  /// Whether everything below it was pruned as well, or only the directory row itself.
  pub(crate) is_recursive: bool,
}

/// Everything one walk rejected, counted always and named only for a run that is going to say them.
///
/// The recording policy lives here rather than at each call site: a caller records unconditionally and this decides
/// whether a name is worth a string that outlives the entry. A quiet run therefore holds one counter, exactly what it
/// held before a run could say anything, whatever the count reaches.
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
  ///
  /// The prune decision is made inside the walker's own filter, which borrows the walk for as long as it runs, so what
  /// it rejects is gathered beside this and [`Self::absorb`]ed once the walk is over.
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
  ///
  /// Uncounted: `count` answers how many files were left out, and a directory is not one. A recursive rule hides an
  /// unknown number of files behind one name, which is exactly why it is worth its own line.
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
  ///
  /// Ordered and deduplicated by the name as the walk spelled it, not by the name the engine would fold it to. Two
  /// included roots reaching one directory are one line; two spellings a case-sensitive host keeps apart are two
  /// files, and a transcript that folded them would claim the run rejected something it never saw. Folding is
  /// registration's question, and this crate already answers it in exactly one place.
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
  use crate::pack::ArchivePackSkipReason;

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
