use xrf_utils::{encode_string_to_bytes, new_windows1251_encoder};

use crate::dom::{XmlDocument, XmlElement};
use crate::options::XmlParseOptions;

#[test]
fn parses_elements_attributes_and_text() {
  let document: XmlDocument = XmlDocument::parse(
    "<root><entry id=\"first\">value</entry><group><entry id=\"second\"/></group></root>",
    XmlParseOptions::default(),
  )
  .unwrap();

  let entries: Vec<&XmlElement> = document.elements_named("entry").collect();

  assert_eq!(document.root().name(), "root");
  assert_eq!(entries.len(), 2);
  assert_eq!(entries[0].attribute("id"), Some("first"));
  assert_eq!(entries[0].text(), "value");
}

#[test]
fn parses_declared_windows_1251_encoding() {
  let source: &str = "<?xml version=\"1.0\" encoding=\"windows-1251\"?><root><text>Привет</text></root>";
  let encoded: Vec<u8> = encode_string_to_bytes(source, new_windows1251_encoder()).unwrap();

  let document: XmlDocument = XmlDocument::parse_bytes(&encoded, XmlParseOptions::default()).unwrap();

  assert_eq!(document.elements_named("text").next().unwrap().text(), "Привет");
}

/// Bytes the declared encoding cannot decode are a defect of the document, never an IO failure.
#[test]
fn reports_undecodable_bytes_as_an_encoding_error() {
  let error = XmlDocument::parse_bytes(b"<root>\xff\xfe</root>", XmlParseOptions::default()).unwrap_err();

  assert!(matches!(error, xrf_error::XrfError::Encoding { .. }), "{error:?}");
}

#[test]
fn rejects_dtd_unless_enabled() {
  let source: &str = "<!DOCTYPE root [<!ELEMENT root EMPTY>]><root/>";

  assert!(XmlDocument::parse(source, XmlParseOptions::default()).is_err());
  assert!(XmlDocument::parse(source, XmlParseOptions { allow_dtd: true }).is_ok());
}

#[test]
fn finds_nested_elements_at_any_depth() {
  let document: XmlDocument = XmlDocument::parse(
    "<root><a><b><target id=\"deep\"/></b></a></root>",
    XmlParseOptions::default(),
  )
  .unwrap();

  assert_eq!(
    document.elements_named("target").next().unwrap().attribute("id"),
    Some("deep")
  );
}

#[test]
fn keeps_attributes_in_document_order() {
  let document: XmlDocument =
    XmlDocument::parse("<root a=\"1\" b=\"2\" c=\"3\"/>", XmlParseOptions::default()).unwrap();

  assert_eq!(
    document.root().attributes().collect::<Vec<_>>(),
    vec![("a", "1"), ("b", "2"), ("c", "3")]
  );
}

#[test]
fn separates_direct_children_from_all_descendants() {
  let document: XmlDocument = XmlDocument::parse(
    "<root><item id=\"1\"/><group><item id=\"2\"/></group></root>",
    XmlParseOptions::default(),
  )
  .unwrap();

  assert_eq!(document.root().children_named("item").count(), 1);
  assert_eq!(document.root().descendants_named("item").count(), 2);
}
