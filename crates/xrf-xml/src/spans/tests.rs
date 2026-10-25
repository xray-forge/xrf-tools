use std::ops::Range;

use crate::options::XmlParseOptions;
use crate::spans::{XmlElementSpan, XmlSourceDocument};

const OPTIONS: XmlParseOptions = XmlParseOptions { allow_dtd: false };

fn parse(source: &str) -> XmlSourceDocument {
  XmlSourceDocument::parse(String::from(source), OPTIONS).expect("Expected a parsed document")
}

fn text_of(document: &XmlSourceDocument) -> &XmlElementSpan {
  document
    .root()
    .child_named("string")
    .and_then(|string| string.child_named("text"))
    .expect("Expected a text element")
}

#[test]
fn locates_content_without_disturbing_the_rest_of_the_document() {
  let source: &str =
    "<string_table>\n\t<!-- note -->\n\t<string id=\"a\">\n\t\t<text>first</text>\n\t</string>\n</string_table>";
  let document: XmlSourceDocument = parse(source);
  let range: Range<usize> = text_of(&document).content_range().unwrap().clone();

  assert_eq!(&document.source()[range.clone()], "first");

  let mut edited: String = document.source().to_owned();
  edited.replace_range(range, "second");

  // The comment and every tab survive, which is the whole reason ranges are carried around.
  assert_eq!(
    edited,
    "<string_table>\n\t<!-- note -->\n\t<string id=\"a\">\n\t\t<text>second</text>\n\t</string>\n</string_table>"
  );
}

#[test]
fn the_source_travels_with_the_ranges_that_address_it() {
  let document: XmlSourceDocument = parse("<string_table><string id=\"a\"><text>x</text></string></string_table>");
  let range: Range<usize> = text_of(&document).content_range().unwrap().clone();

  // Ranges are only meaningful against this exact string, so the document hands it back rather than
  // leaving the caller to keep the right one alive by hand.
  assert_eq!(&document.source()[range], "x");
}

#[test]
fn gives_an_empty_content_range_for_an_empty_element() {
  let document: XmlSourceDocument = parse("<string_table><string id=\"a\"><text></text></string></string_table>");
  let range: Range<usize> = text_of(&document).content_range().unwrap().clone();

  assert!(range.is_empty());
  assert_eq!(&document.source()[range], "");
}

#[test]
fn reports_no_content_range_for_a_self_closing_element() {
  let document: XmlSourceDocument = parse("<string_table><string id=\"a\"><text/></string></string_table>");

  assert!(text_of(&document).content_range().is_none());
}

#[test]
fn keeps_entities_in_the_range_while_resolving_them_in_the_text() {
  let document: XmlSourceDocument =
    parse("<string_table><string id=\"a\"><text>a &amp; b</text></string></string_table>");
  let element: &XmlElementSpan = text_of(&document);

  assert_eq!(element.text(), "a & b");
  assert_eq!(
    &document.source()[element.content_range().unwrap().clone()],
    "a &amp; b"
  );
}

#[test]
fn carries_attributes_and_element_ranges() {
  let document: XmlSourceDocument = parse("<string_table><string id=\"a\"><text>x</text></string></string_table>");
  let string: &XmlElementSpan = document.root().child_named("string").unwrap();

  assert_eq!(string.attribute("id"), Some("a"));
  assert_eq!(
    &document.source()[string.element_range().clone()],
    "<string id=\"a\"><text>x</text></string>"
  );
}

#[test]
fn parses_a_comment_banner_xml_forbids_and_the_engine_accepts() {
  let source: &str =
    "<string_table>\n\t<!-- ---- names ---- -->\n\t<string id=\"a\">\n\t\t<text>x</text>\n\t</string>\n</string_table>";
  let document: XmlSourceDocument = parse(source);

  // The range still addresses the original, banner and all.
  assert_eq!(
    &document.source()[text_of(&document).content_range().unwrap().clone()],
    "x"
  );
}

#[test]
fn parses_a_bare_ampersand_in_text() {
  let document: XmlSourceDocument =
    parse("<string_table><string id=\"a\"><text>Smith & Wesson &amp; sons</text></string></string_table>");
  let element: &XmlElementSpan = text_of(&document);

  assert_eq!(
    &document.source()[element.content_range().unwrap().clone()],
    "Smith & Wesson &amp; sons"
  );
  // The text reads what the engine shows: the bare ampersand as written, never the repair's stand-in for it.
  assert_eq!(element.text(), "Smith & Wesson & sons");
}

#[test]
fn reads_a_bare_ampersand_in_an_attribute_and_leaves_untouched_values_to_the_parser() {
  let document: XmlSourceDocument =
    parse("<string_table><string id=\"a&b\" note=\"x &amp; y\"><text>~ kept</text></string></string_table>");
  let string: &XmlElementSpan = document.root().child_named("string").unwrap();

  assert_eq!(string.attribute("id"), Some("a&b"));
  assert_eq!(string.attribute("note"), Some("x & y"));
  // A real tilde in a node the repair did not touch is the parser's value, unchanged.
  assert_eq!(text_of(&document).text(), "~ kept");
}

#[test]
fn leaves_a_well_formed_document_to_the_strict_parse() {
  let document: XmlSourceDocument =
    parse("<string_table><string id=\"a\"><text>a &amp; b</text></string></string_table>");

  // Repairing would have blanked the entity, so this proves the strict pass is what ran.
  assert_eq!(text_of(&document).text(), "a & b");
}

#[test]
fn reports_a_document_that_survives_neither_pass() {
  assert!(XmlSourceDocument::parse(String::from("<string_table><string id="), OPTIONS).is_err());
}

#[test]
fn hands_the_source_back_for_splicing() {
  let source: &str = "<string_table><string id=\"a\"><text>x</text></string></string_table>";
  let document: XmlSourceDocument = parse(source);

  assert_eq!(document.into_source(), source);
}
