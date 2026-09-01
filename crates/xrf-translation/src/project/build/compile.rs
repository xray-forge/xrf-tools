use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;
use xrf_xml::serialize_xml;

use crate::language::{TranslationLanguage, find_unencodable_character};
use crate::project::build::translation_build_options::TranslationBuildOptions;
use crate::types::{TranslationJson, TranslationVariant};
use crate::xml::compiled::{TranslationCompiledXml, TranslationEntryCompiled};

/// Render one language's string table from a multi-language source.
///
/// A missing translation compiles to the id itself, which is what the engine falls back to anyway, so
/// an untranslated string shows its key in game rather than nothing at all.
///
/// # Errors
///
/// Returns an encoding error when a value cannot be written in the language's code page, and a
/// serialization error when the document cannot be produced.
pub(crate) fn compile_by_language(
  path: &Path,
  source: &TranslationJson,
  language: &TranslationLanguage,
  options: &TranslationBuildOptions,
) -> XrfResult<String> {
  let mut buffer: String = format!(
    "<?xml version=\"1.0\" encoding=\"{}\" ?>\n\n",
    language.get_language_encoding()
  );
  let mut compiled: TranslationCompiledXml = TranslationCompiledXml::default();

  let language_key: String = language.to_string();

  xrf_output::verbose!(
    options.output,
    "Building json file with {} entries, language '{language_key}'",
    source.len(),
  );

  for (key, entry) in source {
    let text: String = entry
      .get(&language_key)
      .map_or(key.clone(), |value| value.as_ref().map_or(key.clone(), render_variant));

    validate_entry_encoding(path, language, key, &text)?;

    compiled.string.push(TranslationEntryCompiled { id: key.clone(), text });
  }

  if options.is_sorted {
    compiled.string.sort_by(|first, second| first.id.cmp(&second.id))
  }

  buffer.push_str(&serialize_xml(&compiled)?);

  Ok(buffer)
}

fn validate_entry_encoding(path: &Path, language: &TranslationLanguage, id: &str, text: &str) -> XrfResult {
  for (field, value) in [("id", id), ("text", text)] {
    if let Some(character) = find_unencodable_character(value, language.new_language_encoder()) {
      return Err(XrfError::new_encoding_error(format!(
        "Translation '{}' entry '{}' {} cannot be encoded as {}: '{}' (U+{:04X})",
        format_path(path),
        id,
        field,
        language.get_language_encoding(),
        character,
        character as u32,
      )));
    }
  }

  Ok(())
}

fn render_variant(variant: &TranslationVariant) -> String {
  match variant {
    TranslationVariant::String(value) => value.clone(),
    // The engine reads `\n` in a string table as a line break, so a multi-line entry joins on it.
    TranslationVariant::MultiString(values) => values.join("\\n"),
  }
}
