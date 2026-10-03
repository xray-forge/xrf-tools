use std::collections::HashMap;

use xrf_error::{XrfError, XrfResult};

use crate::include::{expand_xml_includes, list_xml_includes};

/// Expand `source` against a set of named files, failing on a name the set does not hold.
fn expand(source: &str, files: &[(&str, &str)]) -> XrfResult<String> {
  let files: HashMap<&str, &str> = files.iter().copied().collect();

  let expanded: Vec<u8> = expand_xml_includes(source.as_bytes(), &mut |name: &str| {
    files
      .get(name)
      .map(|content| content.as_bytes().to_vec())
      .ok_or_else(|| XrfError::new_not_found_error(format!("No include '{name}'")))
  })?;

  Ok(String::from_utf8(expanded).unwrap())
}

#[test]
fn returns_a_document_without_includes_as_written() -> XrfResult {
  let source: &str = "<root>\r\n  <a>#included</a>\n</root>";

  assert_eq!(expand(source, &[])?, source);

  Ok(())
}

#[test]
fn splices_an_included_file_in_place_of_its_line() -> XrfResult {
  let expanded: String = expand(
    "<root>\r\n\t#include \"gameplay\\dialogs.xml\"\r\n</root>\r\n",
    &[("gameplay\\dialogs.xml", "<a>1</a>\r\n<b>2</b>")],
  )?;

  assert_eq!(expanded, "<root>\r\n<a>1</a>\r\n<b>2</b>\r\n</root>\r\n");

  Ok(())
}

#[test]
fn expands_includes_inside_included_files() -> XrfResult {
  let expanded: String = expand(
    "<root>\n#include \"outer.xml\"\n</root>",
    &[
      ("outer.xml", "<outer>\n#include \"inner.xml\"\n</outer>\n"),
      ("inner.xml", "<inner/>\n"),
    ],
  )?;

  assert_eq!(expanded, "<root>\n<outer>\n<inner/>\n</outer>\n</root>");

  Ok(())
}

#[test]
fn accepts_blanks_around_the_directive_and_a_name_missing_its_closing_quote() -> XrfResult {
  let expanded: String = expand(
    "  #include   \"a.xml\n#include\t\"a.xml\" trailing\n",
    &[("a.xml", "<a/>\n")],
  )?;

  assert_eq!(expanded, "<a/>\n<a/>\n");

  Ok(())
}

#[test]
fn drops_the_byte_order_mark_and_declaration_of_an_included_file() -> XrfResult {
  let expanded: String = expand(
    "<?xml version=\"1.0\"?>\n<root>\n#include \"part.xml\"\n</root>",
    &[(
      "part.xml",
      "\u{feff}\r\n<?xml version=\"1.0\" encoding=\"windows-1251\"?>\r\n<part/>\r\n",
    )],
  )?;

  // The including document keeps its own declaration, which decides how the whole is decoded.
  assert_eq!(expanded, "<?xml version=\"1.0\"?>\n<root>\n\r\n<part/>\r\n</root>");

  Ok(())
}

#[test]
fn lists_the_names_a_document_includes_without_reading_them() {
  let source: &str = "<root>\n#include \"a.xml\"\n  #include \"b\\c.xml\"\n#include broken\n</root>";

  assert_eq!(list_xml_includes(source.as_bytes()), ["a.xml", "b\\c.xml"]);
  assert!(list_xml_includes(b"<root/>").is_empty());
}

#[test]
fn refuses_an_unquoted_or_empty_name() {
  assert!(expand("#include a.xml\n", &[]).is_err());
  assert!(expand("#include \"\"\n", &[]).is_err());
}

#[test]
fn refuses_an_include_it_cannot_read() {
  let error: XrfError = expand("#include \"missing.xml\"\n", &[]).unwrap_err();

  assert!(error.to_string().contains("missing.xml"));
}

#[test]
fn refuses_includes_nested_past_the_engine_limit() {
  // A file including itself nests until the limit stops it.
  let error: XrfError = expand("#include \"self.xml\"\n", &[("self.xml", "#include \"self.xml\"\n")]).unwrap_err();

  assert!(error.to_string().contains("128"));
}
