use xrf_test_utils::GeneratedResources;

use crate::project::describe::translation_gamedata_reader::TranslationGamedataReader;
use crate::project::tests::fixtures::{roots, table};
use crate::text::translation_variant::TranslationVariant;

#[test]
fn pivots_language_directories_into_one_entry_per_id() {
  let root: &str = "gamedata_read/pivot";

  GeneratedResources::write(&format!("{root}/eng/st_test.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  GeneratedResources::write(&format!("{root}/rus/st_test.xml"), table("st_hello", "Privet"))
    .expect("Expected a written test file");

  let descriptor = TranslationGamedataReader::read(&roots(root), "").unwrap();

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

  GeneratedResources::write(&format!("{root}/eng/st_test.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  GeneratedResources::write(&format!("{root}/map_desc/mp_zaton.xml"), table("st_map", "Zaton"))
    .expect("Expected a written test file");
  GeneratedResources::write(&format!("{root}/only_openxray/openxray.xml"), table("st_x", "x"))
    .expect("Expected a written test file");

  let descriptor = TranslationGamedataReader::read(&roots(root), "").unwrap();

  assert_eq!(descriptor.languages, vec![String::from("eng")]);
}

#[test]
fn keeps_a_language_the_enum_does_not_know_and_says_so() {
  let root: &str = "gamedata_read/unknown_language";

  GeneratedResources::write(&format!("{root}/eng/st_test.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  GeneratedResources::write(&format!("{root}/cze/st_test.xml"), table("st_hello", "Ahoj"))
    .expect("Expected a written test file");

  let descriptor = TranslationGamedataReader::read(&roots(root), "").unwrap();

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

  GeneratedResources::write(
    &format!("{root}/eng/st_test.xml"),
    "<string_table>\n\t<string id=\"st_dup\">\n\t\t<text>shadowed</text>\n\t</string>\n\t<string id=\"st_dup\">\n\t\t<text>winning</text>\n\t</string>\n</string_table>",
  ).expect("Expected a written test file");

  let descriptor = TranslationGamedataReader::read(&roots(root), "").unwrap();
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

  GeneratedResources::write(&format!("{root}/eng/st_good.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  GeneratedResources::write(&format!("{root}/eng/st_bad.xml"), "<string_table><string id=")
    .expect("Expected a written test file");

  let descriptor = TranslationGamedataReader::read(&roots(root), "").unwrap();

  assert!(descriptor.files.contains_key("st_good.xml"));
  assert!(
    descriptor
      .findings
      .iter()
      .any(|finding| finding.rule == "translations.unreadable")
  );
}

#[test]
fn records_the_code_page_each_language_is_drawn_in() {
  let root: &str = "gamedata_read/encodings";

  GeneratedResources::write(&format!("{root}/eng/st_test.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  // Declared as Clear Sky's Polish tables are, which says nothing about the code page the `_cent` fonts draw.
  GeneratedResources::write(
    &format!("{root}/pol/st_test.xml"),
    format!(
      "<?xml version=\"1.0\" encoding=\"windows-1251\"?>\n{}",
      table("st_hello", "Hello")
    ),
  )
  .expect("Expected a written test file");
  GeneratedResources::write(
    &format!("{root}/cze/st_test.xml"),
    format!(
      "<?xml version=\"1.0\" encoding=\"windows-1250\"?>\n{}",
      table("st_hello", "Ahoj")
    ),
  )
  .expect("Expected a written test file");

  let descriptor = TranslationGamedataReader::read(&roots(root), "").unwrap();

  assert_eq!(
    descriptor.encodings.get("eng").map(String::as_str),
    Some("windows-1251")
  );
  assert_eq!(
    descriptor.encodings.get("pol").map(String::as_str),
    Some("windows-1250")
  );
  // A language XRF does not build has only its own declaration to go by.
  assert_eq!(
    descriptor.encodings.get("cze").map(String::as_str),
    Some("windows-1250")
  );
}

#[test]
fn reports_a_table_nested_below_a_language_and_leaves_it_out() {
  let root: &str = "gamedata_read/nested";

  GeneratedResources::write(&format!("{root}/eng/st_test.xml"), table("st_hello", "Hello"))
    .expect("Expected a written test file");
  GeneratedResources::write(&format!("{root}/eng/extra/st_more.xml"), table("st_more", "More"))
    .expect("Expected a written test file");

  let descriptor = TranslationGamedataReader::read(&roots(root), "").unwrap();

  // The engine opens it as `eng\st_more.xml`, which is not there, so none of its strings reach the game.
  assert!(!descriptor.files.contains_key("st_more.xml"));
  assert!(descriptor.findings.iter().any(|finding| {
    finding.rule == "translations.nested" && finding.subject.as_deref() == Some(r"eng\extra\st_more.xml")
  }));
}
