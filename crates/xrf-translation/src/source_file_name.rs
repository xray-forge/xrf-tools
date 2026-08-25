use std::ffi::OsStr;
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
  pub fn parse<T: AsRef<OsStr> + ?Sized>(file_name: &'a T) -> Option<Self> {
    let file_name: &'a str = file_name.as_ref().to_str()?;
    let (source_stem, extension): (&'a str, &'a str) = file_name.rsplit_once('.')?;

    match extension {
      json::FILE_EXTENSION => Some(Self {
        stem: source_stem,
        kind: TranslationSourceFileKind::Json,
        xml_language: None,
      }),
      xml::FILE_EXTENSION => {
        let (stem, language): (&'a str, Option<TranslationLanguage>) = source_stem
          .rsplit_once('.')
          .and_then(|(stem, language_name)| {
            TranslationLanguage::from_str(language_name)
              .ok()
              .map(|language| (stem, language))
          })
          .filter(|(_, language)| *language != TranslationLanguage::All)
          .map_or((source_stem, None), |(stem, language)| (stem, Some(language)));

        Some(Self {
          stem,
          kind: TranslationSourceFileKind::Xml,
          xml_language: language,
        })
      }
      _ => None,
    }
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
}
