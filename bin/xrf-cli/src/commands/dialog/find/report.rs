use serde::Serialize;
use xrf_dialog::DialogReferenceDescriptor;

/// What `dialog find` answers: the names searched for and every element naming one of them.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogFindReport {
  /// Info portions searched for, as asked.
  pub infos: Vec<String>,
  /// Script functions searched for, as asked.
  pub functors: Vec<String>,
  /// Every matching element, in file, dialog and document order.
  pub references: Vec<DialogReferenceDescriptor>,
}
