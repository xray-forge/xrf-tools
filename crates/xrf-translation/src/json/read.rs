use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::types::{TranslationEntry, TranslationJson};

/// Read a multi-language translation source off disk.
///
/// For a caller holding one file and no mounted roots; everything reading a tree goes through the VFS
/// and calls [`parse_json`] with the bytes it already has.
///
/// # Errors
///
/// Returns an IO error when the file cannot be read, and a parsing error when it is not valid JSON or
/// repeats an id or a language.
pub(crate) fn read_json(path: &Path) -> XrfResult<TranslationJson> {
  let mut data: Vec<u8> = Vec::new();

  File::open(path)?.read_to_end(&mut data)?;

  parse_json(&format_path(path).to_string(), &data)
}

/// Parse a multi-language translation source already in hand.
///
/// Strict about duplicates, unlike the XML reader: this is a file the project authors and the build
/// consumes, so a repeated id or language is a mistake to report rather than shipped data to tolerate.
///
/// `subject` names the file only so a refusal can say which one it was; it is never resolved.
///
/// # Errors
///
/// Returns a parsing error when the bytes are not valid JSON or repeat an id or a language.
pub(crate) fn parse_json(subject: &str, data: &[u8]) -> XrfResult<TranslationJson> {
  serde_json::from_slice::<UniqueTranslationJson>(data)
    .map(|json| json.0)
    .map_err(|error| XrfError::new_parsing_error(format!("Failed to parse translation JSON '{subject}': {error}")))
}

struct UniqueTranslationJson(TranslationJson);

impl<'de> Deserialize<'de> for UniqueTranslationJson {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_map(UniqueTranslationJsonVisitor)
  }
}

struct UniqueTranslationJsonVisitor;

impl<'de> Visitor<'de> for UniqueTranslationJsonVisitor {
  type Value = UniqueTranslationJson;

  fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("a translation object with unique IDs")
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: MapAccess<'de>,
  {
    let mut translations: TranslationJson = Default::default();

    while let Some(id) = map.next_key::<String>()? {
      if translations.contains_key(&id) {
        return Err(A::Error::custom(format!("duplicate translation ID '{id}'")));
      }

      let entry: UniqueTranslationEntry = map.next_value()?;
      translations.insert(id, entry.0);
    }

    Ok(UniqueTranslationJson(translations))
  }
}

struct UniqueTranslationEntry(TranslationEntry);

impl<'de> Deserialize<'de> for UniqueTranslationEntry {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_map(UniqueTranslationEntryVisitor)
  }
}

struct UniqueTranslationEntryVisitor;

impl<'de> Visitor<'de> for UniqueTranslationEntryVisitor {
  type Value = UniqueTranslationEntry;

  fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("a translation entry with unique language keys")
  }

  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: MapAccess<'de>,
  {
    let mut entry: TranslationEntry = Default::default();

    while let Some(language) = map.next_key::<String>()? {
      if entry.contains_key(&language) {
        return Err(A::Error::custom(format!(
          "Duplicate translation language: '{language}'"
        )));
      }

      entry.insert(language, map.next_value()?);
    }

    Ok(UniqueTranslationEntry(entry))
  }
}
