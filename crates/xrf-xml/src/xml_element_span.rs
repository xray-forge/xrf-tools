use std::ops::Range;

use xrf_error::{XrfError, XrfResult};

use crate::options::XmlParseOptions;
use crate::unescape::{unescape_attribute, unescape_text};

/// An element together with where it sits in the text it was parsed from.
///
/// Ranges are byte offsets into that text, which is what makes editing a document in place possible:
/// everything outside a spliced range keeps the bytes it was read with, including the comments and
/// indentation that re-serializing would drop.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlElementSpan {
  name: String,
  attributes: Vec<(String, String)>,
  element: Range<usize>,
  content: Option<Range<usize>>,
  text: String,
  children: Vec<XmlElementSpan>,
}

impl XmlElementSpan {
  pub(crate) fn parse_exact(input: &str, parsable: &str, options: XmlParseOptions) -> XrfResult<Self> {
    let document: roxmltree::Document = roxmltree::Document::parse_with_options(
      parsable,
      roxmltree::ParsingOptions {
        allow_dtd: options.allow_dtd,
        ..roxmltree::ParsingOptions::default()
      },
    )
    .map_err(|error| XrfError::new_parsing_error(format!("Failed to parse XML: {error}")))?;

    Ok(Self::from_node(input, parsable, document.root_element()))
  }

  /// One element of the parse of `parsable`, its ranges and values addressing `input`.
  fn from_node(input: &str, parsable: &str, node: roxmltree::Node<'_, '_>) -> Self {
    let name: String = node.tag_name().name().to_owned();
    let element: Range<usize> = node.range();
    let content: Option<Range<usize>> = content_range(input, &element, &name);

    Self {
      attributes: node
        .attributes()
        .map(|attribute| {
          let value: String = match Self::get_changed_source(input, parsable, attribute.range_value()) {
            Some(raw) => unescape_attribute(raw),
            None => attribute.value().to_owned(),
          };

          (attribute.name().to_owned(), value)
        })
        .collect(),
      children: node
        .children()
        .filter(roxmltree::Node::is_element)
        .map(|child| Self::from_node(input, parsable, child))
        .collect(),
      text: node
        .children()
        .filter(roxmltree::Node::is_text)
        .map(|child| match Self::get_changed_source(input, parsable, child.range()) {
          Some(raw) => unescape_text(raw),
          None => child.text().unwrap_or_default().to_owned(),
        })
        .collect(),
      content,
      element,
      name,
    }
  }

  /// The source of a node whose bytes the repair changed, which the parser's own values would misread.
  ///
  /// `None` for every node of a document that parsed as written, and for every node of a repaired one the repair left
  /// alone, so those keep exactly the values the parser resolved.
  fn get_changed_source<'a>(input: &'a str, parsable: &str, range: Range<usize>) -> Option<&'a str> {
    let raw: &'a str = input.get(range.clone())?;

    (parsable.get(range) != Some(raw)).then_some(raw)
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn attribute(&self, name: &str) -> Option<&str> {
    self
      .attributes
      .iter()
      .find(|(attribute, _)| attribute == name)
      .map(|(_, value)| value.as_str())
  }

  /// Iterates over `(name, value)` pairs in source order.
  ///
  /// Asking by name answers what a reader wants; enumerating answers what a validator wants, which is
  /// whether the element carries an attribute nobody expected.
  pub fn attributes(&self) -> impl Iterator<Item = (&str, &str)> {
    self
      .attributes
      .iter()
      .map(|(name, value)| (name.as_str(), value.as_str()))
  }

  /// Byte range of the whole element, both tags included.
  pub fn element_range(&self) -> &Range<usize> {
    &self.element
  }

  /// Byte range of everything between the tags. `None` for a self-closing element, which has no
  /// content to replace and has to be rewritten whole instead.
  pub fn content_range(&self) -> Option<&Range<usize>> {
    self.content.as_ref()
  }

  /// Text content with entities resolved, which is not what the content range holds.
  pub fn text(&self) -> &str {
    &self.text
  }

  pub fn children(&self) -> impl Iterator<Item = &Self> {
    self.children.iter()
  }

  pub fn child_named<'a>(&'a self, name: &'a str) -> Option<&'a Self> {
    self.children.iter().find(move |child| child.name == name)
  }

  pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Self> + 'a {
    self.children.iter().filter(move |child| child.name == name)
  }
}

/// Locate the span between an element's tags without trusting its text children.
///
/// Derived from the element's own range rather than from a text node, because an element may hold no
/// text child at all, or several once entity references split it. Both cases still have exactly one
/// stretch of source between the tags, and that stretch is what an edit replaces.
fn content_range(input: &str, element: &Range<usize>, name: &str) -> Option<Range<usize>> {
  let source: &str = input.get(element.clone())?;
  let open_end: usize = source.find('>')? + 1;

  if source[..open_end].ends_with("/>") {
    return None;
  }

  let closing: String = format!("</{name}>");

  if !source.ends_with(&closing) {
    return None;
  }

  let content_end: usize = element.end.checked_sub(closing.len())?;
  let content_start: usize = element.start + open_end;

  if content_start > content_end {
    return None;
  }

  Some(content_start..content_end)
}

#[cfg(test)]
mod tests;
