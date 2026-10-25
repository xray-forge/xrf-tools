use std::path::Path;

use crate::language::TranslationLanguage;

/// Where a string table says which language it holds.
///
/// Two names rather than a path, because that is all the rule needs and because the two path domains
/// spell a path differently: an engine identity is `\`-separated where a host path is not, so a
/// reader working in either domain can supply these without converting between them.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TranslationIdentity<'a> {
  /// `st_dialogs.xml`, which names the file but no longer says anything about its language.
  ///
  /// Kept because a refusal or a warning has to say which file it is about.
  pub file_name: &'a str,
  /// The `rus` of `text\rus\st_dialogs.xml`, which is how gamedata carries the language, and the
  /// language an importer declares for a tree that carries none.
  pub directory_name: Option<&'a str>,
}

impl<'a> TranslationIdentity<'a> {
  /// Read both names off a host path.
  pub(crate) fn from_path(path: &'a Path) -> Self {
    Self {
      file_name: path.file_name().and_then(|name| name.to_str()).unwrap_or_default(),
      directory_name: path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str()),
    }
  }

  /// The language these names imply, if any.
  pub(super) fn get_language(&self) -> Option<TranslationLanguage> {
    self.directory_name.and_then(TranslationLanguage::from_directory_name)
  }
}
