use xrf_output::{OutputChannel, OutputOptions};
use xrf_utils::format_bytes;

use crate::patch::compare::{ArchivePatchChange, ArchivePatchComparison};
use crate::patch::config::ArchivePatchConfig;
use crate::patch::world::ArchivePatchWorld;

/// The only place a patch run says what it decided.
///
/// Separate from the pack narrator rather than an extension of it: what a comparison has to say is the classification
/// of each entry and where each side read it from, which a packing transcript has no place for. The two meet only at
/// the volume lines, which the pack narrator keeps owning because a patch's volumes are written by the pack writer.
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

  /// The options the pack half narrates through, so both halves of one run share a verbosity.
  pub(crate) fn get_output(&self) -> &'a OutputOptions {
    self.output
  }

  /// What the run was asked to compare, before it costs anything to find out it was the wrong pair.
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
        world.describe_roots()
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

  /// One line per classified entry, then the counts.
  ///
  /// In engine-name order within each class, and the classes in the order a reader acts on them: what the patch
  /// carries first, then what it cannot. Console output is capped; the report carries every entry, which is the same
  /// division `core/collisions.rs` already draws for a warning block nobody can read a thousand lines of.
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

/// Entries of one class a transcript prints before it stops being something a person reads.
///
/// Forty, matching the shared collision block, so two capped listings in one tool do not disagree about how much is
/// too much.
const CONSOLE_ENTRY_CAP: usize = 40;
