use std::path::Path;

use xrf_error::XrfResult;
use xrf_utils::XRayEncoding;
use xrf_xml::XmlEncoding;

use crate::language::translation_language::TranslationLanguage;

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
  pub(crate) fn get_language(&self) -> Option<TranslationLanguage> {
    self.directory_name.and_then(TranslationLanguage::from_directory_name)
  }

  /// The code page a string table's bytes are in, a byte order mark aside.
  ///
  /// Shared by reading and writing on purpose: resolving it differently in each direction would decode a file one way
  /// and re-encode it another, corrupting every byte an edit never touched.
  ///
  /// A language the directory names is read in the code page the game draws it with
  /// ([`TranslationLanguage::get_encoding`]), whatever the file declares: no engine reads the declaration, and shipped
  /// tables contradict theirs. Clear Sky's Polish tables declare Windows-1251 and hold Windows-1250, so trusting the
  /// declaration showed them as Cyrillic. A declaration speaks only for a language XRF has no code page for, and a file
  /// with neither is read as the unprefixed fonts draw it.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when a file of an unknown language declares a code page there is no encoder for.
  pub(crate) fn resolve_encoding(&self, data: &[u8]) -> XrfResult<XRayEncoding> {
    let Some(language) = self.get_language() else {
      return Ok(XmlEncoding::find_declared(data)?.unwrap_or_else(|| TranslationLanguage::All.get_encoding()));
    };

    let encoding: XRayEncoding = language.get_encoding();

    if let Ok(Some(declared)) = XmlEncoding::find_declared(data)
      && declared != encoding
    {
      log::debug!(
        "Translation XML '{}' declares {}, read as {} as the game draws '{language}'",
        self.file_name,
        declared.name(),
        language.get_encoding_label(),
      );
    }

    Ok(encoding)
  }
}
