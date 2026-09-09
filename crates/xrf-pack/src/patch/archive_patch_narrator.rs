use xrf_output::{OutputChannel, OutputOptions};
use xrf_utils::format_bytes;

use crate::patch::compare::{ArchivePatchChange, ArchivePatchComparison};
use crate::patch::config::ArchivePatchConfig;
use crate::patch::world::ArchivePatchWorld;

/// Verbose archive comparison output. Volume output belongs to the pack narrator.
pub(crate) struct ArchivePatchNarrator<'a> {
  output: &'a OutputOptions,
  is_recording: bool,
}

impl<'a> ArchivePatchNarrator<'a> {
  pub(crate) fn new(output: &'a OutputOptions) -> Self {
    Self {
      is_recording: output.is_visible(OutputChannel::Verbose),
      output,
    }
  }

  /// Output options shared with the pack narrator.
  pub(crate) fn get_output(&self) -> &'a OutputOptions {
    self.output
  }

  /// Prints comparison roots and patch settings.
  pub(crate) fn describe_settings(
    &self,
    config: &ArchivePatchConfig,
    base: &ArchivePatchWorld,
    target: &ArchivePatchWorld,
  ) {
    if !self.is_recording {
      return;
    }

    for world in [base, target] {
      xrf_output::verbose!(
        self.output,
        "{}: {}",
        world.get_role().as_label(),
        world.describe_root()
      );
    }

    xrf_output::verbose!(
      self.output,
      "Patch '{}', volume cap {}, {} scope prefix(es), {} ignored prefix(es), {} excluded extension(s)",
      config.name,
      format_bytes(config.max_volume_size),
      config.include.len(),
      config.ignore.len(),
      config.exclude_extensions.len()
    );
  }

  /// Prints added, modified, and removed entries in engine-name order within each class, then totals.
  ///
  /// Each class is capped at `CONSOLE_ENTRY_CAP`; the report retains all entries.
  pub(crate) fn describe_comparison(&self, comparison: &ArchivePatchComparison) {
    if !self.is_recording {
      return;
    }

    Self::describe_class(self.output, "added", &comparison.added);
    Self::describe_class(self.output, "modified", &comparison.modified);
    Self::describe_class(self.output, "absent from target", &comparison.removed);

    xrf_output::verbose!(
      self.output,
      "Compared: {} carried, {} absent from target, {} unchanged, {} payload(s) read to decide",
      comparison.get_carried_count(),
      comparison.removed.len(),
      comparison.unchanged,
      comparison.payloads_read
    );
  }

  fn describe_class(output: &OutputOptions, label: &str, changes: &[ArchivePatchChange]) {
    for change in changes.iter().take(CONSOLE_ENTRY_CAP) {
      xrf_output::verbose!(output, "  {label}: {}", change.name);
    }

    if changes.len() > CONSOLE_ENTRY_CAP {
      xrf_output::verbose!(
        output,
        "  {label}: and {} more, see the report",
        changes.len() - CONSOLE_ENTRY_CAP
      );
    }
  }
}

/// Maximum console entries per change class.
const CONSOLE_ENTRY_CAP: usize = 40;
