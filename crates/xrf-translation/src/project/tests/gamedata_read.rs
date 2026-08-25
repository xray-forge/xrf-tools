use xrf_test_utils::utils::write_generated_test_resource;

use super::{roots, table};
use crate::project::gamedata_read::read_gamedata;
use crate::types::TranslationVariant;

#[test]
fn pivots_language_directories_into_one_entry_per_id() {
  let root: &str = "gamedata_read/pivot";

  write_generated_test_resource(&format!("{root}/eng/st_test.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  write_generated_test_resource(&format!("{root}/rus/st_test.xml"), table("st_hello", "Privet"))
    .expect("Expected a written test file");

  let descriptor = read_gamedata(&roots(root), "").unwrap();

  assert_eq!(descriptor.languages, vec![String::from("eng"), String::from("rus")]);

  let file = descriptor.files.get("st_test.xml").expect("Expected the file");
  let entry = file.entries.get("st_hello").expect("Expected the entry");

  assert_eq!(
    entry.get("eng").unwrap().as_ref(),
    Some(&TranslationVariant::String(String::from("Hello")))
  );
  assert_eq!(
    entry.get("rus").unwrap().as_ref(),
    Some(&TranslationVariant::String(String::from("Privet")))
  );
  // Both copies are recorded, because an edit has to know which file to write to.
  assert_eq!(file.sources.len(), 2);
}

#[test]
fn skips_the_directories_the_engine_skips() {
  let root: &str = "gamedata_read/skips";

  write_generated_test_resource(&format!("{root}/eng/st_test.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  write_generated_test_resource(&format!("{root}/map_desc/mp_zaton.xml"), table("st_map", "Zaton"))
    .expect("Expected a written test file");
  write_generated_test_resource(&format!("{root}/only_openxray/openxray.xml"), table("st_x", "x"))
    .expect("Expected a written test file");

  let descriptor = read_gamedata(&roots(root), "").unwrap();

  assert_eq!(descriptor.languages, vec![String::from("eng")]);
}

#[test]
fn keeps_a_language_the_enum_does_not_know_and_says_so() {
  let root: &str = "gamedata_read/unknown_language";

  write_generated_test_resource(&format!("{root}/eng/st_test.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  write_generated_test_resource(&format!("{root}/cze/st_test.xml"), table("st_hello", "Ahoj"))
    .expect("Expected a written test file");

  let descriptor = read_gamedata(&roots(root), "").unwrap();

  assert!(descriptor.languages.contains(&String::from("cze")));
  assert!(
    descriptor
      .findings
      .iter()
      .any(|finding| finding.rule == "translations.unknown-language")
  );
}

#[test]
fn keeps_the_duplicate_the_game_uses_and_reports_the_rest() {
  let root: &str = "gamedata_read/duplicates";

  write_generated_test_resource(
    &format!("{root}/eng/st_test.xml"),
    "<string_table>\n\t<string id=\"st_dup\">\n\t\t<text>shadowed</text>\n\t</string>\n\t<string id=\"st_dup\">\n\t\t<text>winning</text>\n\t</string>\n</string_table>",
  ).expect("Expected a written test file");

  let descriptor = read_gamedata(&roots(root), "").unwrap();
  let entry = descriptor
    .files
    .get("st_test.xml")
    .and_then(|file| file.entries.get("st_dup"))
    .expect("Expected the entry");

  assert_eq!(
    entry.get("eng").unwrap().as_ref(),
    Some(&TranslationVariant::String(String::from("winning")))
  );
  assert!(
    descriptor
      .findings
      .iter()
      .any(|finding| finding.rule == "translations.duplicate")
  );
}

#[test]
fn opens_a_project_whose_file_cannot_be_parsed_and_reports_it() {
  let root: &str = "gamedata_read/unreadable";

  write_generated_test_resource(&format!("{root}/eng/st_good.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  write_generated_test_resource(&format!("{root}/eng/st_bad.xml"), "<string_table><string id=")
    .expect("Expected a written test file");

  let descriptor = read_gamedata(&roots(root), "").unwrap();

  assert!(descriptor.files.contains_key("st_good.xml"));
  assert!(
    descriptor
      .findings
      .iter()
      .any(|finding| finding.rule == "translations.unreadable")
  );
}

#[test]
fn records_the_encoding_each_language_declares() {
  let root: &str = "gamedata_read/encodings";

  write_generated_test_resource(&format!("{root}/eng/st_test.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");

  let descriptor = read_gamedata(&roots(root), "").unwrap();

  // Undeclared and named `eng`, so it resolves through the language rather than a declaration.
  assert_eq!(
    descriptor.encodings.get("eng").map(String::as_str),
    Some("windows-1252")
  );
}
