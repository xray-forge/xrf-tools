use serde::Serialize;

/// Outcome of a guarded ogf refs patch.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfRefsPatchReport {
  /// Size of the source file before patching.
  pub original_size: usize,
  /// Size of the patched buffer, written unless the patch was a dry run.
  pub patched_size: usize,
  /// How many references were rewritten, which is what distinguishes a real patch from a no-op.
  pub patched_count: u32,
  /// Bytes the source carried that the engine's loader never read, and that the patch discarded.
  pub discarded_size: usize,
  /// Whether the patched buffer was actually written to the destination.
  pub is_dry_run: bool,
}
