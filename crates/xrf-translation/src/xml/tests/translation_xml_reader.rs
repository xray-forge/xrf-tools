use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::xml::translation_xml_reader::TranslationXmlReader;

#[test]
fn returns_entries_in_file_order() -> XrfResult {
  let path = GeneratedResources::write(
    "xml_read/order.xml",
    "<string_table><string id=\"st_b\"><text>B</text></string><string id=\"st_a\"><text>A</text></string></string_table>",
  )?;

  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![
      (String::from("st_b"), String::from("B")),
      (String::from("st_a"), String::from("A")),
    ]
  );

  Ok(())
}

#[test]
fn keeps_duplicates_instead_of_refusing_them() -> XrfResult {
  let path = GeneratedResources::write(
    "xml_read/duplicates.xml",
    "<string_table><string id=\"st_dup\"><text>first</text></string><string id=\"st_dup\"><text>second</text></string></string_table>",
  )?;

  // Deciding which one the engine uses is the caller's rule, not the parser's. Real shipped files
  // contain these, and refusing them would make the editor unable to open what it exists to fix.
  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![
      (String::from("st_dup"), String::from("first")),
      (String::from("st_dup"), String::from("second")),
    ]
  );

  Ok(())
}

#[test]
fn skips_an_entry_with_no_text_element() -> XrfResult {
  let path = GeneratedResources::write(
    "xml_read/no_text.xml",
    "<string_table><string id=\"st_empty\"></string><string id=\"st_ok\"><text>ok</text></string></string_table>",
  )?;

  // The engine skips these too, with a message rather than a failure.
  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_ok"), String::from("ok"))]
  );

  Ok(())
}

#[test]
fn reads_a_comment_banner_that_xml_forbids() -> XrfResult {
  let path = GeneratedResources::write(
    "xml_read/banner.xml",
    "<string_table>\n<!-- ---- names ---- -->\n<string id=\"st_a\"><text>A</text></string>\n</string_table>",
  )?;

  // Shipped gamedata is full of these, and X-Ray's own reader accepts them.
  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_a"), String::from("A"))]
  );

  Ok(())
}

#[test]
fn resolves_entities_in_the_text() -> XrfResult {
  let path = GeneratedResources::write(
    "xml_read/entities.xml",
    "<string_table><string id=\"st_a\"><text>a &amp; b</text></string></string_table>",
  )?;

  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_a"), String::from("a & b"))]
  );

  Ok(())
}

#[test]
fn a_malformed_document_is_reported_rather_than_silently_empty() -> XrfResult {
  let path = GeneratedResources::write("xml_read/malformed.xml", "<string_table><string id=")?;

  assert!(TranslationXmlReader::read(&path).is_err());

  Ok(())
}
