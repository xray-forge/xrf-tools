use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;

use crate::json;
use crate::language::TranslationLanguage;
use crate::xml;

/// A source translation filename and the semantics carried by its extension and optional suffix.
///
/// JSON sources carry translations for every language. XML sources are neutral unless their
/// `<name>.<language>.xml` suffix names one language. Both formats build to an XML table named
/// after the normalized source stem.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TranslationSourceFileName<'a> {
  stem: &'a str,
  kind: TranslationSourceFileKind,
  xml_language: Option<TranslationLanguage>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TranslationSourceFileKind {
  Json,
  Xml,
}

impl<'a> TranslationSourceFileName<'a> {
  /// Parse one supported translation source filename.
  ///
  /// The result borrows its stem from the filename, so the input is borrowed too. Non-Unicode
  /// names are not translation sources because their language suffix cannot be interpreted.
  ///
  /// The extension and the language suffix are matched without case, because the two halves of this
  /// crate see the same file spelled differently: a reader goes through the VFS, whose logical paths
  /// are lower case by definition, while the build and the verifier walk the host and see whatever the
  /// author typed. Matching exactly meant `ST_A.JSON` opened in the editor and was skipped by the
  /// build, with only an info line to say so. The stem keeps its original case, which is what the
  /// build writes its target as.
  ///
  /// Case folding stops at the filename. `TranslationLanguage::from_str` stays exact, because it also
  /// reads language keys inside JSON sources, and those are content rather than host names.
  pub fn parse<T: AsRef<OsStr> + ?Sized>(file_name: &'a T) -> Option<Self> {
    let file_name: &'a str = file_name.as_ref().to_str()?;
    let (source_stem, extension): (&'a str, &'a str) = file_name.rsplit_once('.')?;

    if extension.eq_ignore_ascii_case(json::FILE_EXTENSION) {
      return Some(Self {
        stem: source_stem,
        kind: TranslationSourceFileKind::Json,
        xml_language: None,
      });
    }

    if extension.eq_ignore_ascii_case(xml::FILE_EXTENSION) {
      let (stem, language): (&'a str, Option<TranslationLanguage>) = source_stem
        .rsplit_once('.')
        .and_then(|(stem, language_name)| parse_language(language_name).map(|language| (stem, language)))
        .filter(|(_, language)| *language != TranslationLanguage::All)
        .map_or((source_stem, None), |(stem, language)| (stem, Some(language)));

      return Some(Self {
        stem,
        kind: TranslationSourceFileKind::Xml,
        xml_language: language,
      });
    }

    None
  }

  /// The filename stem the build uses for its XML output.
  pub fn get_stem(&self) -> &'a str {
    self.stem
  }

  /// The file format that determines how the source is read and built.
  pub fn get_kind(&self) -> TranslationSourceFileKind {
    self.kind
  }

  /// The language an XML source suffix names, if it names one.
  pub fn get_xml_language(&self) -> Option<TranslationLanguage> {
    self.xml_language
  }
}

/// Whether a host path names a multi-language JSON source.
///
/// One definition, because the build, the verifier and the initializer each decide this and three
/// spellings of it drifted once already. `Path::file_name` is correct here and only here: these are
/// host paths from a directory walk, not engine identities.
pub(crate) fn is_json_source(path: &Path) -> bool {
  path
    .file_name()
    .and_then(TranslationSourceFileName::parse)
    .is_some_and(|name| name.get_kind() == TranslationSourceFileKind::Json)
}

/// A language suffix as a host filename spells it, which may be in any case.
///
/// Allocates only for a name that is not already lower case, so the common path costs nothing.
fn parse_language(name: &str) -> Option<TranslationLanguage> {
  if name.chars().any(|character| character.is_ascii_uppercase()) {
    return TranslationLanguage::from_str(&name.to_ascii_lowercase()).ok();
  }

  TranslationLanguage::from_str(name).ok()
}

#[cfg(test)]
mod tests {
  use std::ffi::{OsStr, OsString};

  use crate::TranslationLanguage;

  use super::{TranslationSourceFileKind, TranslationSourceFileName};

  #[test]
  fn parses_supported_source_file_names() {
    let xml: TranslationSourceFileName =
      TranslationSourceFileName::parse(OsStr::new("dialogs.ukr.xml")).expect("XML source filename to parse");

    assert_eq!(xml.get_stem(), "dialogs");
    assert_eq!(xml.get_xml_language(), Some(TranslationLanguage::Ukrainian));
    assert_eq!(xml.get_kind(), TranslationSourceFileKind::Xml);

    let json_name = String::from("items.eng.json");
    let json: TranslationSourceFileName =
      TranslationSourceFileName::parse(&json_name).expect("JSON source filename to parse");

    assert_eq!(json.get_stem(), "items.eng");
    assert_eq!(json.get_xml_language(), None);
    assert_eq!(json.get_kind(), TranslationSourceFileKind::Json);

    let xml_name = OsString::from("dialogs.rus.xml");
    let os_string = TranslationSourceFileName::parse(&xml_name).expect("OsString filename to parse");

    assert_eq!(os_string.get_stem(), "dialogs");
    assert_eq!(os_string.get_xml_language(), Some(TranslationLanguage::Russian));

    assert!(TranslationSourceFileName::parse("dialogs.all.xml").is_some_and(|name| name.get_xml_language().is_none()));
    assert_eq!(TranslationSourceFileName::parse("dialogs.ukr.txt"), None);
  }

  #[test]
  fn reads_a_host_name_in_any_case_the_author_typed_it() {
    // The VFS lower-cases logical paths, so a reader sees `st_a.json` where the build, walking the
    // host, sees `ST_A.JSON`. Matching exactly meant the editor opened a file the build then skipped,
    // with only an info line to say so.
    let shouted = TranslationSourceFileName::parse("ST_A.JSON").expect("an upper-case extension still parses");

    assert_eq!(shouted.get_kind(), TranslationSourceFileKind::Json);
    // The stem keeps the author's spelling, because that is what the build names its target after.
    assert_eq!(shouted.get_stem(), "ST_A");

    let suffixed = TranslationSourceFileName::parse("Dialogs.UKR.Xml").expect("an upper-case suffix still parses");

    assert_eq!(suffixed.get_xml_language(), Some(TranslationLanguage::Ukrainian));
    assert_eq!(suffixed.get_stem(), "Dialogs");

    // `all` is the neutral marker in either case, so it names no language either way.
    assert!(TranslationSourceFileName::parse("dialogs.ALL.xml").is_some_and(|name| name.get_xml_language().is_none()));
    // Folding stops at the extension and the suffix: an unrelated extension is still not a source.
    assert_eq!(TranslationSourceFileName::parse("dialogs.ukr.TXT"), None);
  }
}
