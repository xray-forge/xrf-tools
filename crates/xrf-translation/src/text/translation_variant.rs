use serde::{Deserialize, Serialize};

/// One translation's text, which is a single line or a run of them joined on build.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(untagged)]
pub enum TranslationVariant {
  String(String),
  MultiString(Vec<String>),
}

impl TranslationVariant {
  /// The literal pair of characters the engine's UI reads in a string table as a line break.
  pub const LINE_BREAK: &str = "\\n";

  /// Read a string table value as the form that keeps its structure: the lines it breaks into, or itself.
  ///
  /// Lossless, because [`Self::to_single_line`] joins the lines back on the same literal, so the text that reaches the
  /// game is byte for byte what was read. What the array buys is a description a translator can read and review in
  /// the source instead of one 400-character line.
  pub fn from_table_text(text: &str) -> Self {
    if text.contains(Self::LINE_BREAK) {
      return Self::MultiString(text.split(Self::LINE_BREAK).map(String::from).collect());
    }

    Self::String(text.to_owned())
  }

  /// Render the value as the single line a string table holds.
  ///
  /// A multi-line entry joins on [`Self::LINE_BREAK`] rather than losing its structure. Only a JSON source can hold
  /// the array form.
  pub fn to_single_line(&self) -> String {
    match self {
      Self::String(value) => value.clone(),
      Self::MultiString(values) => values.join(Self::LINE_BREAK),
    }
  }
}
