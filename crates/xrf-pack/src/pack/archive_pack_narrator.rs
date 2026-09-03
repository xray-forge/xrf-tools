//! Everything a packing run says about itself, in one place.
//!
//! The phases decide; this says. Keeping the phrasing here rather than beside each decision is what lets a transcript
//! read as one account of a run: the walk, the writer and the packer would otherwise each own a third of a vocabulary
//! they have to keep consistent by hand.
//!
//! A placement or a volume is said as it happens, so a run that stops has already named the volume it was writing and
//! the last entry it reached. The walk's omissions are the exception: it reaches them in whatever order the host
//! enumerates, so [`ArchivePackOmissions`] orders them and they are said as one block once the selection is settled.

use std::path::Path;

use xrf_output::{OutputChannel, OutputOptions};
use xrf_utils::{format_bytes, format_bytes_pair};

use crate::pack::ArchivePackEntryOutcome;
use crate::pack::config::{ArchivePackConfig, ArchivePackMode};
use crate::pack::source::ArchivePackOmissions;

/// Says what a packing run decided, at verbose level, or says nothing at all.
///
/// Whether anything is being read is answered once, at construction, and every method here returns on it before it
/// does anything else — so a quiet run costs one predictable branch per decision, allocates nothing, and holds
/// nothing. [`Self::is_recording`] publishes the same answer for a caller whose cost is the work it would have to do
/// to be narrated at all, rather than the line itself. Saying changes no byte of what is written.
pub(crate) struct ArchivePackNarrator<'o> {
  output: &'o OutputOptions,
  is_recording: bool,
}

impl<'o> ArchivePackNarrator<'o> {
  pub(crate) fn new(output: &'o OutputOptions) -> Self {
    Self {
      output,
      is_recording: output.is_visible(OutputChannel::Verbose),
    }
  }

  /// Whether anything said here would be rendered.
  pub(crate) const fn is_recording(&self) -> bool {
    self.is_recording
  }

  /// Say what the run is about to work with, before it does anything.
  ///
  /// The settings a transcript needs to explain its own later lines: why a file was stored, where a volume was cut,
  /// why a leftover was skipped. The selection rules are not repeated, since the lines that follow show what they
  /// selected.
  pub(crate) fn describe_settings(&self, config: &ArchivePackConfig) {
    if !self.is_recording {
      return;
    }

    xrf_output::verbose!(
      self.output,
      "Pack mode: {}, volume cap {}, skip list {}, {} excluded extension(s)",
      match config.mode {
        ArchivePackMode::Compress => "compress",
        ArchivePackMode::Store => "store",
      },
      format_bytes(config.max_volume_size),
      if config.is_with_skip_list { "on" } else { "off" },
      config.exclude_extensions.len()
    );
  }

  /// Say what the walk selected and what the rules left out, as one block before any volume exists.
  ///
  /// Grouped rather than interleaved, and in this order, because a reader comparing two runs finds the whole selection
  /// in one place instead of sifting it out of the placements. Directories come from registration, so they are the
  /// rows an archive will actually carry rather than the spellings the walk happened to reach; the two omission groups
  /// come from the walk, already ordered by [`ArchivePackOmissions`].
  ///
  /// An excluded directory is worth its own line because the walk never reaches inside a recursively excluded one:
  /// without it, a transcript could not tell a rule that applied from a directory that was not there.
  pub(crate) fn describe_selection(&self, directories: &[String], omitted: &ArchivePackOmissions) {
    if !self.is_recording {
      return;
    }

    for directory in directories {
      xrf_output::verbose!(self.output, "Directory: {directory}");
    }

    for directory in omitted.get_directories() {
      xrf_output::verbose!(
        self.output,
        "Excluded directory: {} ({} rule)",
        directory.name,
        if directory.is_recursive { "recursive" } else { "shallow" }
      );
    }

    for file in omitted.get_files() {
      xrf_output::verbose!(self.output, "Skipped: {} ({})", file.name, file.reason.as_label());
    }
  }

  /// Say how one entry landed, in the words xrCompress used for the same outcomes.
  pub(crate) fn describe_entry(
    &self,
    name: &str,
    outcome: ArchivePackEntryOutcome,
    size_real: u32,
    size_compressed: u32,
  ) {
    if !self.is_recording {
      return;
    }

    match outcome {
      ArchivePackEntryOutcome::Compressed => {
        let (real, compressed): (String, String) = format_bytes_pair(u64::from(size_real), u64::from(size_compressed));

        xrf_output::verbose!(
          self.output,
          "Compressed: {name}, {real} -> {compressed} ({:.1}%)",
          100.0 * f64::from(size_compressed) / f64::from(size_real)
        );
      }
      ArchivePackEntryOutcome::Stored => {
        xrf_output::verbose!(self.output, "Stored: {name}, {}", format_bytes(u64::from(size_real)));
      }
      ArchivePackEntryOutcome::Reverted => {
        xrf_output::verbose!(
          self.output,
          "Reverted: {name}, {} (compression saved nothing)",
          format_bytes(u64::from(size_real))
        );
      }
      ArchivePackEntryOutcome::Empty => {
        xrf_output::verbose!(self.output, "Empty: {name}");
      }
      ArchivePackEntryOutcome::Aliased { source } => {
        // The `ALIAS (<source>)` line xrCompress logged for this decision, by logical entry rather than host path.
        xrf_output::verbose!(self.output, "Aliased: {name} -> {source}");
      }
    }
  }

  /// Say that a volume has been opened, which is said before the file is created.
  ///
  /// From that moment the path exists and has replaced whatever stood there, so a run that fails inside the volume has
  /// already named it.
  pub(crate) fn describe_opened_volume(&self, path: &Path) {
    if !self.is_recording {
      return;
    }

    xrf_output::verbose!(self.output, "Opened volume: {}", volume_name(path));
  }

  /// Say what a volume came to once its descriptor table was written.
  pub(crate) fn describe_closed_volume(&self, path: &Path, size: u64, entries: usize) {
    if !self.is_recording {
      return;
    }

    xrf_output::verbose!(
      self.output,
      "Closed volume: {}, {}, {entries} entries",
      volume_name(path),
      format_bytes(size)
    );
  }
}

/// The volume's own file name, which is what a reader of the transcript recognises it by.
fn volume_name(path: &Path) -> String {
  path.file_name().map_or_else(
    || path.display().to_string(),
    |name| name.to_string_lossy().into_owned(),
  )
}
