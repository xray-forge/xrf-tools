use serde::{Deserialize, Serialize};

use crate::project::descriptor::DialogElementDescriptor;

/// One dialog or phrase element a reference search matched.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogReferenceDescriptor {
  /// Logical path of the file declaring the dialog.
  pub logical_path: String,
  pub dialog_id: String,
  /// The phrase holding the element, or `None` for one written on the dialog itself.
  pub phrase_id: Option<String>,
  pub element: DialogElementDescriptor,
  /// Whether the engine never reads it: a condition on an entry phrase no other phrase leads back to.
  pub is_ignored: bool,
}
