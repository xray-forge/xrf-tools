use std::ops::Range;

use xrf_xml::{XmlElementSpan, XmlEscape};

/// Indentation used when a file holds no entry to copy the style from.
const DEFAULT_STRING_INDENT: &str = "\t";
const DEFAULT_TEXT_INDENT: &str = "\t\t";

/// The formatting an existing file already uses, so an inserted entry does not announce itself.
pub(crate) struct XmlLayout {
  line_ending: &'static str,
  string_indent: String,
  text_indent: String,
}

impl XmlLayout {
  pub(crate) fn detect(source: &str, strings: &[&XmlElementSpan]) -> Self {
    let last: Option<&&XmlElementSpan> = strings.last();

    Self {
      line_ending: if source.contains("\r\n") { "\r\n" } else { "\n" },
      string_indent: last
        .map(|element| line_indent(source, element.element_range().start))
        .filter(|indent| !indent.is_empty())
        .unwrap_or_else(|| DEFAULT_STRING_INDENT.to_owned()),
      text_indent: last
        .and_then(|element| element.child_named("text"))
        .map(|text| line_indent(source, text.element_range().start))
        .filter(|indent| !indent.is_empty())
        .unwrap_or_else(|| DEFAULT_TEXT_INDENT.to_owned()),
    }
  }

  pub(crate) fn render_entry(&self, id: &str, text: &str) -> String {
    let Self {
      line_ending,
      string_indent,
      text_indent,
    } = self;

    format!(
      "{line_ending}{string_indent}<string id=\"{}\">{line_ending}{text_indent}<text>{}</text>{line_ending}{string_indent}</string>",
      XmlEscape::attribute(id),
      XmlEscape::text(text),
    )
  }

  /// Append after the last entry, or just inside the table when there is none.
  pub(crate) fn insertion_offset(&self, root: &XmlElementSpan, strings: &[&XmlElementSpan]) -> usize {
    strings.last().map_or_else(
      || {
        root
          .content_range()
          .map_or(root.element_range().end, |range| range.start)
      },
      |element| element.element_range().end,
    )
  }
}

/// Take the entry's own line with it, so removing one does not leave an indented blank behind.
pub(crate) fn removal_range(source: &str, element: &Range<usize>) -> Range<usize> {
  let bytes: &[u8] = source.as_bytes();
  let mut start: usize = element.start;

  while start > 0 && matches!(bytes[start - 1], b' ' | b'\t') {
    start -= 1;
  }

  if start > 0 && bytes[start - 1] == b'\n' {
    start -= 1;

    if start > 0 && bytes[start - 1] == b'\r' {
      start -= 1;
    }
  }

  start..element.end
}

fn line_indent(source: &str, offset: usize) -> String {
  let bytes: &[u8] = source.as_bytes();
  let mut start: usize = offset;

  while start > 0 && matches!(bytes[start - 1], b' ' | b'\t') {
    start -= 1;
  }

  source[start..offset].to_owned()
}
