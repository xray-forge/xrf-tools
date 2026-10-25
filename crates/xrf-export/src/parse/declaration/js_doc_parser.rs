use std::collections::BTreeMap;

use xrf_typescript::swc_common::BytePos;
use xrf_typescript::swc_common::comments::{CommentKind, Comments};

use crate::manifest::ExternDocumentation;

/// Parses JSDoc comments associated with one TypeScript source file.
pub(crate) struct JsDocParser<'a> {
  comments: &'a dyn Comments,
}

impl<'a> JsDocParser<'a> {
  /// The tag a parameter's description is written under.
  const PARAM_TAG: &'static str = "@param";

  /// The tag the return's description is written under.
  const RETURNS_TAG: &'static str = "@returns";

  /// Create a parser over the comment store associated with one TypeScript source.
  pub fn new(comments: &'a dyn Comments) -> Self {
    Self { comments }
  }

  /// Read a leading JSDoc block into the manifest documentation fields.
  ///
  /// Empty descriptions and `@returns` tags are omitted from the result.
  pub fn parse(&self, position: BytePos) -> Option<ExternDocumentation> {
    let raw: String = self.read_raw(position)?;
    let description: Option<String> = Self::read_description(&raw);
    let returns: Option<String> = Self::read_tag(&raw, Self::RETURNS_TAG);

    (description.is_some() || returns.is_some()).then_some(ExternDocumentation { description, returns })
  }

  /// Read documented parameter descriptions from a leading JSDoc block.
  ///
  /// Only `@param name description` entries with both a name and non-empty
  /// description are returned.
  pub fn parameter_docs(&self, position: BytePos) -> BTreeMap<String, String> {
    let Some(raw) = self.read_raw(position) else {
      return BTreeMap::new();
    };

    let mut result: BTreeMap<String, String> = BTreeMap::new();

    for line in Self::list_lines(&raw) {
      let Some(value) = line.strip_prefix(Self::PARAM_TAG) else {
        continue;
      };

      let mut parts = value.trim().splitn(2, char::is_whitespace);

      let Some(name) = parts.next().filter(|value| !value.is_empty()) else {
        continue;
      };

      let Some(description) = parts.next().map(Self::normalize_text).filter(|value| !value.is_empty()) else {
        continue;
      };

      result.insert(name.into(), description);
    }

    result
  }

  /// The block comments leading a position, joined.
  fn read_raw(&self, position: BytePos) -> Option<String> {
    Some(
      self
        .comments
        .get_leading(position)?
        .into_iter()
        .filter(|comment| comment.kind == CommentKind::Block)
        .map(|comment| comment.text.to_string())
        .collect::<Vec<String>>()
        .join("\n"),
    )
  }

  /// The text before the first tag.
  fn read_description(raw: &str) -> Option<String> {
    let lines: Vec<String> = Self::list_lines(raw)
      .into_iter()
      .take_while(|line| !line.starts_with('@'))
      .collect();
    let value: String = lines.join(Self::get_line_ending(raw)).trim().to_string();

    (!value.is_empty()).then_some(value)
  }

  /// A tag's text, its own line and every line after it up to the next tag.
  fn read_tag(raw: &str, tag: &str) -> Option<String> {
    let lines: Vec<String> = Self::list_lines(raw);
    let start: usize = lines.iter().position(|line| line.starts_with(tag))?;

    let mut value: Vec<String> = vec![Self::normalize_text(&lines[start][tag.len()..])];

    for line in lines.iter().skip(start + 1) {
      if line.starts_with('@') {
        break;
      }

      value.push(line.clone());
    }

    let result: String = value.join(Self::get_line_ending(raw)).trim().to_string();

    (!result.is_empty()).then_some(result)
  }

  /// Each line of a block with its `*` margin taken off.
  fn list_lines(raw: &str) -> Vec<String> {
    raw
      .lines()
      .map(|line| line.trim().trim_start_matches('*').trim().to_string())
      .collect()
  }

  /// The line ending the block is written with, which the joined text keeps.
  fn get_line_ending(raw: &str) -> &'static str {
    if raw.contains("\r\n") { "\r\n" } else { "\n" }
  }

  /// A tag's text without the `-` that separates it from the name.
  fn normalize_text(value: &str) -> String {
    value.trim().trim_start_matches('-').trim().to_string()
  }
}
