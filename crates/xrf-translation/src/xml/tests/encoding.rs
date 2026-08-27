use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;
use xrf_utils::{encode_string_to_bytes, new_windows1251_encoder, new_windows1252_encoder};

use crate::xml::read::read_string_table;

fn write_encoded(relative_path: &str, source: &str, encoding: xrf_utils::XRayEncoding) -> XrfResult<PathBuf> {
  Ok(write_generated_test_resource(
    relative_path,
    encode_string_to_bytes(source, encoding)?,
  )?)
}

#[test]
fn reads_xml_using_its_declared_windows_1251_encoding() -> XrfResult {
  let relative_path: &str = "xml_encoding/declared.xml";
  let source: &str = "<?xml version=\"1.0\" encoding=\"windows-1251\"?><string_table><string id=\"st_test\"><text>Привіт</text></string></string_table>";

  let path: PathBuf = write_encoded(relative_path, source, new_windows1251_encoder())?;

  let entries = read_string_table(&path)?;

  assert_eq!(entries, vec![(String::from("st_test"), String::from("Привіт"))]);

  Ok(())
}

#[test]
fn declarationless_xml_uses_the_language_code_page_fallback() -> XrfResult {
  let relative_path: &str = "xml_encoding/declarationless.xml";
  let source: &str = "<string_table><string id=\"st_test\"><text>À bientôt</text></string></string_table>";

  let path: PathBuf = write_encoded(relative_path, source, new_windows1252_encoder())?;

  let entries = read_string_table(&path)?;

  assert_eq!(entries, vec![(String::from("st_test"), String::from("À bientôt"))]);

  Ok(())
}

#[test]
fn a_declaration_outranks_the_directory_language() -> XrfResult {
  let relative_path: &str = "xml_encoding/eng/mismatched.xml";
  let source: &str = "<?xml version=\"1.0\" encoding=\"windows-1251\"?><string_table><string id=\"st_test\"><text>Привет</text></string></string_table>";

  // The directory says English, which would mean 1252 and mangle every Cyrillic byte. What the file
  // itself declares is what it was written with, so that is what it is read with.
  let path: PathBuf = write_encoded(relative_path, source, new_windows1251_encoder())?;

  let entries = read_string_table(&path)?;

  assert_eq!(entries, vec![(String::from("st_test"), String::from("Привет"))]);

  Ok(())
}

#[test]
fn a_gamedata_directory_supplies_the_encoding_when_nothing_else_does() -> XrfResult {
  let relative_path: &str = "xml_encoding/rus/undeclared.xml";
  let source: &str = "<string_table><string id=\"st_test\"><text>Привет</text></string></string_table>";

  // No declaration, so the only statement of language is the directory it sits in.
  let path: PathBuf = write_encoded(relative_path, source, new_windows1251_encoder())?;

  let entries = read_string_table(&path)?;

  assert_eq!(entries, vec![(String::from("st_test"), String::from("Привет"))]);

  Ok(())
}

#[test]
fn utf16_is_refused_rather_than_read_as_a_code_page() -> XrfResult {
  let path: PathBuf = write_generated_test_resource("xml_encoding/utf16.xml", [0xFF, 0xFE, 0x3C, 0x00])?;
  let error = read_string_table(&path).unwrap_err();

  assert!(error.to_string().contains("UTF-16"));

  Ok(())
}
