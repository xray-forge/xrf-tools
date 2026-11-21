use serde::Serialize;

use crate::edit::{TextureDescriptorForm, TextureReferenceField};

/// A copy's descriptor field that names one of the original's files, and the copy's name for the same file.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextureReferenceRename {
  pub field: TextureReferenceField,
  /// The field's value, naming the original's file.
  pub from: String,
  /// The value naming the copy's file instead.
  pub to: String,
}

impl TextureReferenceRename {
  /// Sets the field to the copy's name.
  pub(crate) fn apply_to(&self, form: &mut TextureDescriptorForm) {
    let field: &mut String = match self.field {
      TextureReferenceField::Bump => &mut form.bump_name,
      TextureReferenceField::Detail => &mut form.detail_name,
    };

    field.clone_from(&self.to);
  }
}
