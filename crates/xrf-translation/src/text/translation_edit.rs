use serde::{Deserialize, Serialize};

use crate::text::translation_variant::TranslationVariant;

/// One change to a translation entry, in whichever kind of file holds it.
///
/// Format-neutral on purpose. It used to live beside the XML writer and carry a bare `String`, which
/// quietly could not express what a JSON source already holds: an entry whose text is an array of
/// lines. Editing one of those flattened it on save, and roughly 190 entries across ten files in the
/// engine's own translations are that shape.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TranslationEdit {
  /// Replace the winning entry's value, or append the entry when the file has none.
  Set { id: String, value: TranslationVariant },
  /// Remove the entry entirely, shadowed duplicates included.
  Remove { id: String },
}

impl TranslationEdit {
  pub fn id(&self) -> &str {
    match self {
      Self::Set { id, .. } | Self::Remove { id } => id,
    }
  }
}
