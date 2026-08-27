use std::collections::HashMap;

use xrf_translation::{TranslationEntry, TranslationProjectDescriptor, TranslationVariant};

/// Every translation key a project's text tree offers, flattened across the files holding them.
///
/// Flattened because a phrase names a key and nothing else: `<text>about_skadovsk_stalkers_1</text>`
/// says which string, never which file. Resolving through the per-file map would mean scanning every
/// file for every phrase, which is the wrong shape for a canvas drawing ninety-six of them.
///
/// A key defined in two files resolves to the **last** one read, which is the copy
/// `CStringTable::Load` leaves in the engine's own table. The duplicate is still reported as a
/// finding by the reader that produced the descriptor; this only decides which text is shown.
#[derive(Debug, Default)]
pub struct DialogTextIndex {
  languages: Vec<String>,
  entries: HashMap<String, TranslationEntry>,
}

impl DialogTextIndex {
  /// Flatten a translations project into a key lookup.
  pub fn from_descriptor(descriptor: &TranslationProjectDescriptor) -> Self {
    let mut entries: HashMap<String, TranslationEntry> = HashMap::new();

    for file in descriptor.files.values() {
      for (id, languages) in &file.entries {
        // Later files overwrite earlier ones, which is the engine's own last-wins rule. The map is
        // replaced rather than merged per language: two files defining one key are two competing
        // definitions, and merging them would invent a row neither file holds.
        entries.insert(id.clone(), languages.clone());
      }
    }

    Self {
      languages: descriptor.languages.clone(),
      entries,
    }
  }

  /// Languages the text tree offers, in the order it discovered them.
  pub fn get_languages(&self) -> &[String] {
    &self.languages
  }

  /// The language a caller naming none gets.
  ///
  /// The first the tree offers rather than a hard-coded `eng`, because a project may not ship one —
  /// `gamedata` holds a single language and it is not always English.
  pub fn get_default_language(&self) -> Option<&str> {
    self.languages.first().map(String::as_str)
  }

  /// Whether a language is one this tree actually holds.
  pub fn has_language(&self, language: &str) -> bool {
    self.languages.iter().any(|known| known == language)
  }

  /// The line one key reads as in one language, as the engine would render it.
  ///
  /// `None` covers three different situations on purpose, because a caller showing text treats them
  /// alike: no such key, a key the language has no row for, and a row explicitly set to null. Which
  /// one it was is a question for validation, not for drawing a node.
  pub fn resolve(&self, key: &str, language: &str) -> Option<String> {
    self
      .entries
      .get(key)?
      .get(language)?
      .as_ref()
      .map(TranslationVariant::to_single_line)
  }

  /// Whether the tree defines a key at all, in any language.
  ///
  /// Separate from [`Self::resolve`] because they answer different questions: a key nothing defines is
  /// a broken reference, while a key defined in other languages is untranslated work.
  pub fn contains_key(&self, key: &str) -> bool {
    self.entries.contains_key(key)
  }

  /// Count of distinct keys, which is what says whether a text tree was found at all.
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// This index narrowed to one language, or `None` when it holds no such language.
  ///
  /// The pair is bound into one value because a key and a language are meaningless apart: passing an
  /// index and a language as two arguments lets a caller resolve against a language the index never
  /// had, and answer `None` for every key without ever learning why.
  pub fn in_language<'a>(&'a self, language: &'a str) -> Option<DialogTextLanguage<'a>> {
    self
      .has_language(language)
      .then_some(DialogTextLanguage { index: self, language })
  }
}

/// One language of one text index, which is everything resolving a phrase needs.
#[derive(Clone, Copy, Debug)]
pub struct DialogTextLanguage<'a> {
  index: &'a DialogTextIndex,
  language: &'a str,
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
