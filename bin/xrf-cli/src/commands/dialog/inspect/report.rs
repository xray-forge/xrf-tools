use serde::Serialize;
use xrf_dialog::DialogDescriptor;

/// What `dialog inspect` answers about one dialog.
///
/// The dialog carries the file it was read from, so this adds only what is about the question: the other files
/// declaring the same id, which a person asks again with `--file`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogInspectReport {
  pub dialog: DialogDescriptor,
  /// Logical paths of the other files declaring the id, in logical-path order.
  pub also_declared_in: Vec<String>,
}
