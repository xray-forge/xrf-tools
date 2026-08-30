//! Telling a person which files a mounted world holds but cannot reach.
//!
//! Shared because the condition belongs to a set of sources rather than to any one command: `gamedata list` answers for
//! an installation and `archive verify` for a volume set, and the same fact reported in two shapes is the per-command
//! drift the reporting contract removes. The reported shape itself is [`xrf_vfs::XrayPathCollision`], deposited as it
//! stands.

use xrf_output::OutputOptions;
use xrf_vfs::XrayPathCollision;

/// Warns about files a source holds but cannot reach, printing at most `limit` of them.
///
/// Always reported rather than behind a flag: unlike a shadowed entry, which is how an override is meant to work, an
/// unreachable file is an authoring mistake nobody asked to see because nobody knew about it.
pub fn print_collisions(output: &OutputOptions, collisions: &[XrayPathCollision], limit: usize) {
  if collisions.is_empty() {
    return;
  }

  xrf_output::warning!(
    output,
    "{} file(s) cannot be reached, another file claims their path:",
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
