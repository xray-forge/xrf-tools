use crate::project::dialog_text_index::DialogTextIndex;

/// One language of one text index, which is everything resolving a phrase needs.
#[derive(Clone, Copy, Debug)]
pub struct DialogTextLanguage<'a> {
  pub(super) index: &'a DialogTextIndex,
  pub(super) language: &'a str,
}

impl DialogTextLanguage<'_> {
  /// The language this resolves in, which a response echoes back so a caller knows what it got.
  pub fn get_language(&self) -> &str {
    self.language
  }

  /// The line one key reads as, or `None` when this language has no text for it.
  pub fn resolve(&self, key: &str) -> Option<String> {
    self.index.resolve(key, self.language)
  }
}
