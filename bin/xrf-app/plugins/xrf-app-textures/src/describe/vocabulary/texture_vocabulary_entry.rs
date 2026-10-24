use serde::Serialize;

/// One value a descriptor field can take, under the name the SDK gives it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureVocabularyEntry {
  /// The number stored in the file.
  pub value: u32,
  /// The SDK's own identifier for it, which is the name an author of a `.thm` would recognise.
  pub label: String,
}

impl TextureVocabularyEntry {
  /// One enum's named values, each paired with the number it is stored as.
  pub fn of_all<T: Copy + Into<u32>>(named: impl IntoIterator<Item = T>, label: fn(T) -> String) -> Vec<Self> {
    named
      .into_iter()
      .map(|value| Self {
        value: value.into(),
        label: label(value),
      })
      .collect()
  }
}
