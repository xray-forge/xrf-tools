use serde::{Deserialize, Serialize};

use crate::conversation::dialog_phrase::DialogPhrase;
use crate::project::descriptor::dialog_element_descriptor::DialogElementDescriptor;
use crate::project::text::dialog_text_language::DialogTextLanguage;

/// One line of a conversation, as a canvas draws it.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogPhraseDescriptor {
  /// Unique within its dialog, and what `next` references. The entry phrase is `0`.
  pub id: String,
  /// Translation key of the line, which is what the file holds.
  ///
  /// Absent for a phrase whose line comes from `script_text`, which is a state and not a defect:
  /// Anomaly does it 107 times.
  pub text_key: Option<String>,
  /// The line itself, in the language this dialog was described for.
  pub text: Option<String>,
  /// Whether selecting this phrase ends the conversation.
  pub is_final: bool,
  /// Whether the phrase sits inside a `phrase_list` rather than directly under its dialog.
  ///
  /// Both forms occur, and a later insertion has to reproduce the one the file already uses.
  pub is_in_phrase_list: bool,
  /// Ids that may follow this one, **in the order the player is offered them**.
  pub next: Vec<String>,
  /// Every child element in document order, including the ones projected above.
  pub elements: Vec<DialogElementDescriptor>,
}

impl DialogPhraseDescriptor {
  /// Describe a phrase, resolving its line when a text tree was read for the requested language.
  pub fn new(phrase: &DialogPhrase, text: Option<DialogTextLanguage<'_>>) -> Self {
    let text_key: Option<String> = phrase.get_text().map(str::to_owned);

    Self {
      id: phrase.get_id().to_owned(),
      // Resolved only where there is a key to resolve. A `script_text` phrase builds its line at
      // runtime, so looking one up would report it as untranslated when nothing is missing.
      text: text_key
        .as_deref()
        .and_then(|key| text.and_then(|text| text.resolve(key))),
      text_key,
      is_final: phrase.is_final(),
      is_in_phrase_list: phrase.is_in_phrase_list(),
      next: phrase.list_next().into_iter().map(str::to_owned).collect(),
      elements: phrase.get_elements().iter().map(Into::into).collect(),
    }
  }
}
