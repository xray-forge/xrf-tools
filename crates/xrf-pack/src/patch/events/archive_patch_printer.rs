use std::sync::Arc;

use xrf_output::{OutputChannel, OutputOptions};
use xrf_utils::{format_bytes, format_path};

use crate::pack::events::ArchivePackPrinter;
use crate::patch::compare::ArchivePatchChange;
use crate::patch::events::{ArchivePatchEvent, ArchivePatchListener};
use crate::patch::world::ArchivePatchRole;

/// What a person reads of a patch run: the roots and settings, then each added and modified entry and the totals, at
/// verbose level. The volumes a publishing run writes are said by [`ArchivePackPrinter`].
pub struct ArchivePatchPrinter;

impl ArchivePatchPrinter {
  /// Console entries listed per change class; the report keeps every entry.
  const ENTRY_CAP: usize = 40;

  /// A listener printing every event to `output`, or none when verbose lines would not be shown, so a run nobody is
  /// reading skips the work of producing its events.
  pub fn new_listener(output: &OutputOptions) -> Option<ArchivePatchListener> {
    if !output.is_visible(OutputChannel::Verbose) {
      return None;
    }

    let output: OutputOptions = output.clone();

    Some(Arc::new(move |event: &ArchivePatchEvent<'_>| {
      Self::print_event(event, &output);
    }))
  }

  /// Says one event at verbose level.
  pub fn print_event(event: &ArchivePatchEvent<'_>, output: &OutputOptions) {
    match event {
      ArchivePatchEvent::Settings {
        config,
        base_root,
        target_root,
      } => {
        for (role, root) in [
          (ArchivePatchRole::Base, base_root),
          (ArchivePatchRole::Target, target_root),
        ] {
          xrf_output::verbose!(output, "{}: {}", role.as_label(), format_path(root));
        }

        xrf_output::verbose!(
          output,
          "Patch '{}', volume cap {}, {} scope prefix(es), {} ignored prefix(es), {} excluded extension(s)",
          config.name,
          format_bytes(config.max_volume_size),
          config.include.len(),
          config.ignore.len(),
          config.exclude_extensions.len()
        );
      }
      ArchivePatchEvent::Comparison {
        added,
        modified,
        carried,
        unchanged,
        payloads_read,
      } => {
        Self::print_class(output, "added", added);
        Self::print_class(output, "modified", modified);

        xrf_output::verbose!(
          output,
          "Compared: {carried} carried, {unchanged} unchanged, {payloads_read} payload(s) read to decide"
        );
      }
      ArchivePatchEvent::Pack(event) => ArchivePackPrinter::print_event(event, output),
    }
  }

  /// One class's entries in engine-name order, capped at [`Self::ENTRY_CAP`].
  fn print_class(output: &OutputOptions, label: &str, changes: &[ArchivePatchChange]) {
    for change in changes.iter().take(Self::ENTRY_CAP) {
      xrf_output::verbose!(output, "  {label}: {}", change.name);
    }

    if changes.len() > Self::ENTRY_CAP {
      xrf_output::verbose!(
        output,
        "  {label}: and {} more, see the report",
        changes.len() - Self::ENTRY_CAP
      );
    }
  }
}
