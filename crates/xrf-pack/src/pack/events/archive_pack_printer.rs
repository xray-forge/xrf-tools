use std::path::Path;
use std::sync::Arc;

use xrf_output::{OutputChannel, OutputOptions};
use xrf_utils::{format_bytes, format_bytes_pair};

use crate::pack::ArchivePackEntryOutcome;
use crate::pack::config::ArchivePackMode;
use crate::pack::events::{ArchivePackEvent, ArchivePackListener};

/// What a person reads of a packing run: one verbose line per setting block, directory, omission, placed entry and
/// volume, in the words xrCompress used for the same outcomes.
///
/// The run itself prints nothing and hands out [`ArchivePackEvent`]s; this says them, so a transcript reads as one
/// account whichever phase made the decision.
pub struct ArchivePackPrinter;

impl ArchivePackPrinter {
  /// A listener printing every event to `output`, or none when verbose lines would not be shown, so a run nobody is
  /// reading skips the work of producing its events.
  pub fn new_listener(output: &OutputOptions) -> Option<ArchivePackListener> {
    if !output.is_visible(OutputChannel::Verbose) {
      return None;
    }

    let output: OutputOptions = output.clone();

    Some(Arc::new(move |event: &ArchivePackEvent<'_>| {
      Self::print_event(event, &output);
    }))
  }

  /// Says one event at verbose level.
  pub fn print_event(event: &ArchivePackEvent<'_>, output: &OutputOptions) {
    match *event {
      // The settings a transcript needs to explain its own later lines: why a file was stored, where a volume was cut,
      // why a leftover was skipped. The selection rules are not repeated, since the lines that follow show what they
      // selected.
      ArchivePackEvent::Settings(config) => xrf_output::verbose!(
        output,
        "Pack mode: {}, volume cap {}, skip list {}, {} excluded extension(s)",
        match config.volume_settings.mode {
          ArchivePackMode::Compress => "compress",
          ArchivePackMode::Store => "store",
        },
        format_bytes(config.volume_settings.max_volume_size),
        if config.is_with_skip_list { "on" } else { "off" },
        config.exclude_extensions.len()
      ),
      // Grouped rather than interleaved, and in this order, because a reader comparing two runs finds the whole
      // selection in one place instead of sifting it out of the placements.
      ArchivePackEvent::Selection {
        directories,
        excluded_directories,
        skipped_files,
      } => {
        for directory in directories {
          xrf_output::verbose!(output, "Directory: {directory}");
        }

        for directory in excluded_directories {
          xrf_output::verbose!(
            output,
            "Excluded directory: {} ({} rule)",
            directory.name,
            if directory.is_recursive { "recursive" } else { "shallow" }
          );
        }

        for file in skipped_files {
          xrf_output::verbose!(output, "Skipped: {} ({})", file.name, file.reason.as_label());
        }
      }
      ArchivePackEvent::Entry {
        name,
        outcome,
        size_real,
        size_compressed,
      } => Self::print_entry(output, name, outcome, size_real, size_compressed),
      ArchivePackEvent::VolumeOpened { path } => {
        xrf_output::verbose!(output, "Opened volume: {}", Self::get_volume_name(path));
      }
      ArchivePackEvent::VolumeClosed { path, size, entries } => xrf_output::verbose!(
        output,
        "Closed volume: {}, {}, {entries} entries",
        Self::get_volume_name(path),
        format_bytes(size)
      ),
    }
  }

  fn print_entry(
    output: &OutputOptions,
    name: &str,
    outcome: ArchivePackEntryOutcome<'_>,
    size_real: u32,
    size_compressed: u32,
  ) {
    match outcome {
      ArchivePackEntryOutcome::Compressed => {
        let (real, compressed): (String, String) = format_bytes_pair(u64::from(size_real), u64::from(size_compressed));

        xrf_output::verbose!(
          output,
          "Compressed: {name}, {real} -> {compressed} ({:.1}%)",
          100.0 * f64::from(size_compressed) / f64::from(size_real)
        );
      }
      ArchivePackEntryOutcome::Stored => {
        xrf_output::verbose!(output, "Stored: {name}, {}", format_bytes(u64::from(size_real)));
      }
      ArchivePackEntryOutcome::Reverted => {
        xrf_output::verbose!(
          output,
          "Reverted: {name}, {} (compression saved nothing)",
          format_bytes(u64::from(size_real))
        );
      }
      ArchivePackEntryOutcome::Empty => {
        xrf_output::verbose!(output, "Empty: {name}");
      }
      ArchivePackEntryOutcome::Aliased { source } => {
        // The `ALIAS (<source>)` line xrCompress logged for this decision, by logical entry rather than host path.
        xrf_output::verbose!(output, "Aliased: {name} -> {source}");
      }
    }
  }

  /// The volume's own file name, which is what a reader of the transcript recognises it by.
  fn get_volume_name(path: &Path) -> String {
    path.file_name().map_or_else(
      || path.display().to_string(),
      |name| name.to_string_lossy().into_owned(),
    )
  }
}
