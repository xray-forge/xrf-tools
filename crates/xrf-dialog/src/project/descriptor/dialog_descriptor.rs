use serde::{Deserialize, Serialize};

use crate::conversation::dialog::Dialog;
use crate::project::descriptor::dialog_element_descriptor::DialogElementDescriptor;
use crate::project::descriptor::dialog_phrase_descriptor::DialogPhraseDescriptor;
use crate::project::text::dialog_text_language::DialogTextLanguage;

/// One whole dialog: its own elements and every phrase it declares.
///
/// What a selection fetches, against the summary the project index already gave. Both names it was
/// addressed by are echoed back, so a response arriving late cannot be read as another dialog's — the
/// same rule the asset commands follow.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogDescriptor {
  /// Logical path of the file holding it, as the project keys that file.
  pub logical_path: String,
  pub id: String,
  /// Selection priority, negative for a dialog meant to sort last.
  pub priority: Option<i32>,
  /// Dialog-level elements — preconditions, info gates, `init_func` — excluding the phrases.
  pub elements: Vec<DialogElementDescriptor>,
  /// The language the phrase text was resolved in, echoed back.
  pub language: Option<String>,
  /// Phrases in document order.
  ///
  /// Empty is legitimate: `dm_traveler_dialog` carries only a precondition and an init function and
  /// builds its phrases from script at runtime.
  pub phrases: Vec<DialogPhraseDescriptor>,
}

impl DialogDescriptor {
  /// Describe a dialog found under a logical path, resolving its lines where a text tree was read.
  pub fn new(logical_path: impl Into<String>, dialog: &Dialog, text: Option<DialogTextLanguage<'_>>) -> Self {
    Self {
      logical_path: logical_path.into(),
      id: dialog.get_id().to_owned(),
      priority: dialog.get_priority(),
      elements: dialog.get_elements().iter().map(Into::into).collect(),
      language: text.map(|text| text.get_language().to_owned()),
      phrases: dialog
        .get_phrases()
        .iter()
        .map(|phrase| DialogPhraseDescriptor::new(phrase, text))
        .collect(),
    }
  }
}
