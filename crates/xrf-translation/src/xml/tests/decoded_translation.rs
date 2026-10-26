use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;
use xrf_utils::{encode_string_to_bytes, new_windows1250_encoder, new_windows1251_encoder};

use crate::xml::translation_xml_reader::TranslationXmlReader;

fn write_encoded(relative_path: &str, source: &str, encoding: xrf_utils::XRayEncoding) -> XrfResult<PathBuf> {
  Ok(GeneratedResources::write(
    relative_path,
    encode_string_to_bytes(source, encoding)?,
  )?)
}

#[test]
fn a_language_directory_is_read_in_the_code_page_the_game_draws_it_with() -> XrfResult {
  // Shaped like Clear Sky's Polish tables: declared Windows-1251, written in Windows-1250 for the `_cent` fonts. The
  // engine reads no declaration, so trusting this one showed every Polish letter as Cyrillic.
  let source: &str = "<?xml version=\"1.0\" encoding=\"windows-1251\"?><string_table><string id=\"st_test\"><text>Do następnego razu!</text></string></string_table>";
  let path: PathBuf = write_encoded("xml_encoding/pol/st_dialogs.xml", source, new_windows1250_encoder())?;

  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_test"), String::from("Do następnego razu!"))]
  );

  Ok(())
}

#[test]
fn a_declaration_speaks_for_a_language_xrf_has_no_code_page_for() -> XrfResult {
  let source: &str = "<?xml version=\"1.0\" encoding=\"windows-1250\"?><string_table><string id=\"st_test\"><text>Příště</text></string></string_table>";
  let path: PathBuf = write_encoded("xml_encoding/cze/st_dialogs.xml", source, new_windows1250_encoder())?;

  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_test"), String::from("Příště"))]
  );

  Ok(())
}

#[test]
fn a_gamedata_directory_supplies_the_encoding_when_nothing_else_does() -> XrfResult {
  let source: &str = "<string_table><string id=\"st_test\"><text>Привет</text></string></string_table>";
  let path: PathBuf = write_encoded("xml_encoding/rus/undeclared.xml", source, new_windows1251_encoder())?;

  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_test"), String::from("Привет"))]
  );

  Ok(())
}

#[test]
fn a_file_saying_nothing_of_its_language_is_read_as_the_unprefixed_fonts_draw_it() -> XrfResult {
  let source: &str = "<string_table><string id=\"st_test\"><text>Привіт</text></string></string_table>";
  let path: PathBuf = write_encoded("xml_encoding/undeclared.xml", source, new_windows1251_encoder())?;

  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_test"), String::from("Привіт"))]
  );

  Ok(())
}

#[test]
fn a_byte_order_mark_outranks_the_language() -> XrfResult {
  // gamedata-coc's st_items_weapons.xml is UTF-8 marked and declares windows-1251.
  let mut marked: Vec<u8> = vec![0xEF, 0xBB, 0xBF];

  marked.extend_from_slice(
    "<?xml version=\"1.0\" encoding=\"windows-1251\"?><string_table><string id=\"st_test\"><text>Привет</text></string></string_table>"
      .as_bytes(),
  );

  let path: PathBuf = GeneratedResources::write("xml_encoding/rus/marked.xml", marked)?;

  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_test"), String::from("Привет"))]
  );

  Ok(())
}

#[test]
fn utf16_is_refused_rather_than_read_as_a_code_page() -> XrfResult {
  let path: PathBuf = GeneratedResources::write("xml_encoding/utf16.xml", [0xFF, 0xFE, 0x3C, 0x00])?;
  let error = TranslationXmlReader::read(&path).unwrap_err();

  assert!(error.to_string().contains("UTF-16"));

  Ok(())
}
