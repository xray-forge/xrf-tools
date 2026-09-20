use std::collections::BTreeMap;

use xrf_typescript::swc_common::BytePos;
use xrf_typescript::swc_common::comments::Comments;

use crate::extern_manifest::ExternDocumentation;

/// Parses JSDoc comments associated with one TypeScript source file.
pub struct JsDocParser<'a> {
  comments: &'a dyn Comments,
}

impl<'a> JsDocParser<'a> {
  /// Create a parser over the comment store associated with one TypeScript source.
  pub fn new(comments: &'a dyn Comments) -> Self {
    Self { comments }
  }

  /// Read a leading JSDoc block into the manifest documentation fields.
  ///
  /// Empty descriptions and `@returns` tags are omitted from the result.
  pub fn parse(&self, position: BytePos) -> Option<ExternDocumentation> {
    let raw: String = self.raw(position)?;
    let description: Option<String> = documentation_description(&raw);
    let returns: Option<String> = documentation_tag(&raw, "returns");

    (description.is_some() || returns.is_some()).then_some(ExternDocumentation { description, returns })
  }

  /// Read documented parameter descriptions from a leading JSDoc block.
  ///
  /// Only `@param name description` entries with both a name and non-empty
  /// description are returned.
  pub fn parameter_docs(&self, position: BytePos) -> BTreeMap<String, String> {
    let Some(raw) = self.raw(position) else {
      return BTreeMap::new();
    };

    let mut result: BTreeMap<String, String> = BTreeMap::new();

    for line in documentation_lines(&raw) {
      let Some(value) = line.strip_prefix("@param") else {
        continue;
      };

      let mut parts = value.trim().splitn(2, char::is_whitespace);

      let Some(name) = parts.next().filter(|value| !value.is_empty()) else {
        continue;
      };

      let Some(description) = parts.next().map(normalize_doc_text).filter(|value| !value.is_empty()) else {
        continue;
      };
      result.insert(name.into(), description);
    }

    result
  }

  fn raw(&self, position: BytePos) -> Option<String> {
    Some(
      self
        .comments
        .get_leading(position)?
        .into_iter()
        .filter(|comment| comment.kind == xrf_typescript::swc_common::comments::CommentKind::Block)
        .map(|comment| comment.text.to_string())
        .collect::<Vec<String>>()
        .join("\n"),
    )
  }
}

fn documentation_description(raw: &str) -> Option<String> {
  let lines: Vec<String> = documentation_lines(raw)
    .into_iter()
    .take_while(|line| !line.starts_with('@'))
    .collect();
  let value: String = lines.join(documentation_line_ending(raw)).trim().to_string();

  (!value.is_empty()).then_some(value)
}

fn documentation_tag(raw: &str, tag: &str) -> Option<String> {
  let prefix: String = format!("@{tag}");
  let lines: Vec<String> = documentation_lines(raw);
  let start: usize = lines.iter().position(|line| line.starts_with(&prefix))?;

  let mut value: Vec<String> = vec![normalize_doc_text(&lines[start][prefix.len()..])];

  for line in lines.iter().skip(start + 1) {
    if line.starts_with('@') {
      break;
    }
    value.push(line.clone());
  }

  let result: String = value.join(documentation_line_ending(raw)).trim().to_string();

  (!result.is_empty()).then_some(result)
}

fn documentation_lines(raw: &str) -> Vec<String> {
  raw
    .lines()
    .map(|line| line.trim().trim_start_matches('*').trim().to_string())
    .collect()
}

fn documentation_line_ending(raw: &str) -> &str {
  if raw.contains("\r\n") { "\r\n" } else { "\n" }
}

fn normalize_doc_text(value: &str) -> String {
  value.trim().trim_start_matches('-').trim().to_string()
}
