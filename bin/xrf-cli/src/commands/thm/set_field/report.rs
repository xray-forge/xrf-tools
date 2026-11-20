use serde::Serialize;

/// Which descriptors `thm set-field` changed, and how many it found already as asked or not texture descriptors.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ThmSetFieldReport {
  /// The descriptors whose bytes changed, or would have on a dry run, by name so a run is repeatable.
  pub changed: Vec<String>,
  pub is_dry_run: bool,
  /// Thumbnails below a directory that describe an object, a group, a sound or nothing the SDK reads.
  pub skipped: usize,
  /// Texture descriptors that already held every field as named, and were not rewritten.
  pub unchanged: usize,
}
