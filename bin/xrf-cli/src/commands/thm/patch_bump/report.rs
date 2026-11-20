use serde::Serialize;
use xrf_thm::ThmBumpMode;

/// What `thm patch-bump` found a descriptor's bump declaration to be, and how the file's size changed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ThmPatchBumpReport {
  pub original_size: usize,
  /// The size of the patched file, which a dry run does not write.
  pub patched_size: usize,
  pub previous_name: String,
  /// Serialized as the number the file stores.
  pub previous_mode: ThmBumpMode,
  pub is_dry_run: bool,
}
