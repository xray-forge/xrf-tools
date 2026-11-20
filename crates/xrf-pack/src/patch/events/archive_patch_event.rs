use std::path::Path;

use crate::pack::events::ArchivePackEvent;
use crate::patch::compare::ArchivePatchChange;
use crate::patch::config::ArchivePatchConfig;

/// One decision a patch run made, handed to its listener as it is made.
///
/// Borrowed from the run like [`ArchivePackEvent`], which it carries for the volumes a publishing run writes.
#[derive(Clone, Copy, Debug)]
pub enum ArchivePatchEvent<'e> {
  /// The two roots compared and the patch's own settings, once both sides are mounted.
  Settings {
    config: &'e ArchivePatchConfig,
    base_root: &'e Path,
    target_root: &'e Path,
  },
  /// What the comparison found: every added and modified entry in engine-name order, and the counts behind them.
  Comparison {
    added: &'e [ArchivePatchChange],
    modified: &'e [ArchivePatchChange],
    carried: usize,
    unchanged: usize,
    /// Entry pairs whose checksums had to be computed to decide.
    payloads_read: usize,
  },
  /// What the packer decided while publishing the difference.
  Pack(ArchivePackEvent<'e>),
}
