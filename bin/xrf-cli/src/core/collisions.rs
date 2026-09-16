//! Telling a person what a set of sources holds twice over, and what it cannot reach at all.
//!
//! Shared because both conditions belong to a set of sources rather than to any one command: `gamedata list` answers
//! for an installation and `archive verify` for a volume set, and the same fact reported in two shapes is the
//! per-command drift the reporting contract removes. The reported shapes are [`xrf_vfs::XrayPathCollision`] and the
//! override listing, deposited as they stand.

use xrf_output::OutputOptions;
use xrf_vfs::{XrayPathCollision, XrayShadowingEntry};

/// Warns about files a source holds but cannot reach, printing at most `limit` of them.
///
/// Always reported rather than behind a flag: unlike an override, which is how a patch is meant to work, an
/// unreachable file is an authoring mistake nobody asked to see because nobody knew about it.
pub fn print_collisions(output: &OutputOptions, collisions: &[XrayPathCollision], limit: usize) {
  if collisions.is_empty() {
    return;
  }

  xrf_output::warning!(
    output,
    "{} file(s) cannot be reached, another file of the same source claims their path:",
    collisions.len()
  );

  for collision in collisions.iter().take(limit) {
    xrf_output::warning!(
      output,
      "  {} is unreachable, {} answers '{}'",
      collision.unreachable,
      collision.kept,
      collision.logical_path
    );
  }

  if collisions.len() > limit {
    xrf_output::warning!(output, "  ... {} more not printed", collisions.len() - limit);
  }
}

/// Reports the engine paths a set of sources answers with more than one copy, printing at most `limit` of them.
pub fn print_overrides(output: &OutputOptions, overrides: &[XrayShadowingEntry], limit: usize) {
  if overrides.is_empty() {
    return;
  }

  let hidden: usize = overrides.iter().map(|entry| entry.shadowed.len()).sum();

  xrf_output::info!(
    output,
    "{} path(s) are held more than once, burying {hidden} cop(ies):",
    overrides.len()
  );

  for entry in overrides.iter().take(limit) {
    xrf_output::info!(
      output,
      "  '{}' answers from {}, hiding {} cop(ies)",
      entry.get_logical_path(),
      entry.get_asset().format_container(),
      entry.shadowed.len()
    );
  }

  if overrides.len() > limit {
    xrf_output::info!(output, "  ... {} more not printed", overrides.len() - limit);
  }
}
