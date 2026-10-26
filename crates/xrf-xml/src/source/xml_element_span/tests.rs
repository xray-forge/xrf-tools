use std::ops::Range;

use crate::source::{XmlElementSpan, XmlSourceDocument};
use crate::xml_parse_options::XmlParseOptions;

fn parse(source: &str) -> XmlSourceDocument {
  XmlSourceDocument::parse(String::from(source)).expect("Expected a parsed document")
}

fn text_of(document: &XmlSourceDocument) -> &XmlElementSpan {
  document
    .root()
    .child_named("string")
    .and_then(|string| string.child_named("text"))
    .expect("Expected a text element")
}

/// The source the root element's content range addresses.
fn root_content(source: &str) -> Option<String> {
  let document: XmlSourceDocument = parse(source);

  document
    .root()
    .content_range()
    .map(|range| document.source()[range.clone()].to_owned())
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
fn finds_content_past_a_closing_angle_bracket_quoted_in_an_attribute() {
  assert_eq!(root_content("<text note=\"a > b\">old</text>").as_deref(), Some("old"));
  assert_eq!(root_content("<text note='a /> b'>old</text>").as_deref(), Some("old"));
  assert_eq!(root_content("<text note=\"a /> b\"/>"), None);
}

#[test]
fn finds_content_before_a_closing_tag_with_blanks_or_a_prefix() {
  assert_eq!(root_content("<text>old</text >").as_deref(), Some("old"));
  assert_eq!(
    root_content("<n:text xmlns:n=\"urn:n\">old</n:text>").as_deref(),
    Some("old")
  );
}

#[test]
fn finds_content_holding_markup_comments_and_cdata() {
  assert_eq!(
    root_content("<root><!-- a --><child/><![CDATA[</root>]]>tail</root>").as_deref(),
    Some("<!-- a --><child/><![CDATA[</root>]]>tail")
  );
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
fn addresses_bytes_past_multibyte_text() {
  let document: XmlSourceDocument = parse("<string_table><string id=\"п\"><text>Привет</text></string></string_table>");
  let range: Range<usize> = text_of(&document).content_range().unwrap().clone();

  assert_eq!(&document.source()[range], "Привет");
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
fn reads_cdata_as_written_when_a_banner_forced_the_repair() {
  let document: XmlSourceDocument = parse("<!-- -- --><root><![CDATA[A & B <!-- -- -->]]></root>");

  assert_eq!(document.root().text(), "A & B <!-- -- -->");
}

#[test]
fn leaves_a_well_formed_document_to_the_strict_parse() {
  let document: XmlSourceDocument =
    parse("<string_table><string id=\"a\"><text>a &amp; b</text></string></string_table>");

  // Repairing would have blanked the entity, so this proves the strict pass is what ran.
  assert_eq!(text_of(&document).text(), "a & b");
}

#[test]
fn refuses_a_dtd_whose_entities_would_point_ranges_into_the_declaration() {
  assert!(
    XmlSourceDocument::parse(String::from(
      "<!DOCTYPE root [<!ENTITY value '<text>old</text>'>]><root>&value;</root>"
    ))
    .is_err()
  );
}

#[test]
fn refuses_nesting_past_the_depth_bound_rather_than_overflowing() {
  let nested = |depth: usize| format!("{}x{}", "<a>".repeat(depth), "</a>".repeat(depth));

  let deepest: String = nested(XmlParseOptions::MAX_DEPTH);

  // The deepest document accepted is parsed, cloned, compared, formatted and dropped on a main thread's stack, since
  // each of those recurses once per level.
  std::thread::Builder::new()
    .stack_size(1 << 20)
    .spawn(move || {
      let document: XmlSourceDocument = XmlSourceDocument::parse(deepest).unwrap();

      assert_eq!(document.clone(), document);
      assert!(format!("{document:?}").len() > XmlParseOptions::MAX_DEPTH);
    })
    .unwrap()
    .join()
    .unwrap();

  // Refused by the strict parse and its repaired retry alike, neither of which may recurse that deep.
  let error: String = XmlSourceDocument::parse(nested(100_000)).unwrap_err().to_string();

  assert!(
    error.contains(&format!("deeper than {} levels", XmlParseOptions::MAX_DEPTH)),
    "{error}"
  );
}

#[test]
fn reports_a_document_that_survives_neither_pass() {
  assert!(XmlSourceDocument::parse(String::from("<string_table><string id=")).is_err());
}

#[test]
fn hands_the_source_back_for_splicing() {
  let source: &str = "<string_table><string id=\"a\"><text>x</text></string></string_table>";
  let document: XmlSourceDocument = parse(source);

  assert_eq!(document.into_source(), source);
}
