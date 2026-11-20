use xrf_error::{XrfError, XrfResult};
use xrf_utils::{encode_string_to_bytes, natural_cmp};
use xrf_xml::XmlEscape;

use crate::language::translation_code_page::TranslationCodePage;
use crate::language::translation_language::TranslationLanguage;
use crate::project::describe::translation_finding::TranslationFinding;
use crate::text::translation_json::TranslationJson;
use crate::xml::translation_line_limit::TranslationLineLimit;

/// Renders one language's string table from a multi-language source, as the game will load it.
pub(crate) struct TranslationTableCompiler;

impl TranslationTableCompiler {
  /// The table's bytes, in the language's code page, refusing one the game would mangle or refuse.
  ///
  /// A missing translation compiles to the id itself, which is what the engine falls back to anyway, so an
  /// untranslated string shows its key in game rather than nothing at all.
  ///
  /// # Errors
  ///
  /// Returns an encoding error naming the entry when a value cannot be written in the language's code page, and an
  /// invalid error naming it when its line is longer than the game's XML reader takes.
  pub(crate) fn compile(
    subject: &str,
    source: &TranslationJson,
    language: TranslationLanguage,
    is_sorted: bool,
  ) -> XrfResult<Vec<u8>> {
    for (id, text) in Self::entries(source, language) {
      if let Some((field, character)) = Self::find_unencodable(&id, &text, language) {
        return Err(XrfError::new_encoding_error(format!(
          "Translation '{subject}' entry '{id}' {field} cannot be encoded as {}: {}",
          language.get_encoding_label(),
          TranslationCodePage::describe(character),
        )));
      }
    }

    let rendered: String = Self::render(source, language, is_sorted);
    let bytes: Vec<u8> = encode_string_to_bytes(&rendered, language.get_encoding())?;

    if let Some((number, line)) = TranslationLineLimit::find_overlong(&bytes).first() {
      return Err(XrfError::new_invalid_error(format!(
        "Translation '{subject}' entry '{}' would not load in '{language}': {}",
        Self::id_at_line(&rendered, *number).unwrap_or_default(),
        TranslationLineLimit::describe(*number, line.len())
      )));
    }

    Ok(bytes)
  }

  /// Every reason the game would mangle or refuse this language's table, as findings rather than a refusal: what
  /// [`Self::compile`] refuses on, all of it rather than the first.
  pub(crate) fn inspect(
    subject: &str,
    source: &TranslationJson,
    language: TranslationLanguage,
  ) -> Vec<TranslationFinding> {
    let mut findings: Vec<TranslationFinding> = Vec::new();

    for (id, text) in Self::entries(source, language) {
      if let Some((field, character)) = Self::find_unencodable(&id, &text, language) {
        findings.push(TranslationFinding::new(
          "translations.unencodable",
          Some(subject.to_owned()),
          format!(
            "'{id}' {field} in '{language}' holds {}, which {} cannot write, so the game shows another character",
            TranslationCodePage::describe(character),
            language.get_encoding_label()
          ),
        ));
      }
    }

    let rendered: String = Self::render(source, language, false);
    // Lossy, because an unencodable character is already reported above and still takes its bytes on the line.
    let bytes: Vec<u8> = language.get_encoding().encode(&rendered).0.into_owned();

    for (number, line) in TranslationLineLimit::find_overlong(&bytes) {
      findings.push(TranslationFinding::new(
        "translations.overlong",
        Some(subject.to_owned()),
        format!(
          "'{}' in '{language}' would not load: its table's {}",
          Self::id_at_line(&rendered, number).unwrap_or_default(),
          TranslationLineLimit::describe(number, line.len())
        ),
      ));
    }

    findings
  }

  /// The table as text, before it is encoded: each entry a `<string id="…">` holding its `<text>`, indented by two
  /// spaces a level and escaped as XRF writes XML by hand ([`XmlEscape`]).
  pub(crate) fn render(source: &TranslationJson, language: TranslationLanguage, is_sorted: bool) -> String {
    let mut entries: Vec<(String, String)> = Self::entries(source, language).collect();

    // The canonical source order, so a sorted table lists its entries the way the source does once formatted.
    if is_sorted {
      entries.sort_by(|(first, _), (second, _)| natural_cmp(first, second));
    }

    let mut rendered: String = format!(
      "<?xml version=\"1.0\" encoding=\"{}\" ?>\n\n<string_table>",
      language.get_encoding_label()
    );

    for (id, text) in &entries {
      rendered.push_str(&format!(
        "\n  <string id=\"{}\">\n    <text>{}</text>\n  </string>",
        XmlEscape::attribute(id),
        XmlEscape::text(text)
      ));
    }

    if !entries.is_empty() {
      rendered.push('\n');
    }

    rendered.push_str("</string_table>");
    rendered
  }

  /// Every entry's id and the text this language's table holds for it.
  fn entries(source: &TranslationJson, language: TranslationLanguage) -> impl Iterator<Item = (String, String)> {
    let language_key: String = language.to_string();

    source.iter().map(move |(id, entry)| {
      let text: String = entry
        .get(&language_key)
        .and_then(Option::as_ref)
        .map_or_else(|| id.clone(), |variant| variant.to_single_line());

      (id.clone(), text)
    })
  }

  /// The first character of an entry its language's code page cannot hold, and whether it is in the id or the text.
  fn find_unencodable(id: &str, text: &str, language: TranslationLanguage) -> Option<(&'static str, char)> {
    [("id", id), ("text", text)].into_iter().find_map(|(field, value)| {
      TranslationCodePage::find_unencodable(value, language.get_encoding()).map(|character| (field, character))
    })
  }

  /// The id of the entry one line of a rendered table belongs to: the nearest `<string id="…">` at or above it.
  ///
  /// Reliable because the table is one this rendered, where text is escaped and cannot open an element.
  fn id_at_line(rendered: &str, number: usize) -> Option<String> {
    rendered
      .lines()
      .take(number)
      .collect::<Vec<&str>>()
      .into_iter()
      .rev()
      .find_map(|line| {
        line
          .trim_start()
          .strip_prefix("<string id=\"")
          .and_then(|rest| rest.split_once('"'))
          .map(|(id, _)| id.to_owned())
      })
  }
}
