use serde::{Deserialize, Serialize};

use crate::dialog_element::DialogElement;
use crate::dialog_element_kind::DialogElementKind;

/// One child element of a dialog or a phrase, as written.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogElementDescriptor {
  /// The element name as written, such as `give_info`.
  pub name: String,
  /// What that name means to the engine.
  pub kind: DialogElementKind,
  /// Text content, with entity references already resolved.
  pub value: String,
}

impl From<&DialogElement> for DialogElementDescriptor {
  fn from(element: &DialogElement) -> Self {
    Self {
      name: element.get_name().to_owned(),
      kind: element.get_kind(),
      value: element.get_value().to_owned(),
    }
  }
}
