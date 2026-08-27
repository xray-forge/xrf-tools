use xrf_test_utils::utils::write_generated_test_resource;

use super::roots;
use crate::project::source_read::read_source;

#[test]
fn reads_a_json_map_as_one_file_carrying_every_language() {
  let root: &str = "source_read/json";

  write_generated_test_resource(
    &format!("{root}/st_test.json"),
    r#"{"st_hello":{"eng":"Hello","ukr":"Pryvit"}}"#,
  )
  .expect("Expected a written test file");

  let descriptor = read_source(&roots(root), "").unwrap();

  assert_eq!(descriptor.languages, vec![String::from("eng"), String::from("ukr")]);

  let file = descriptor.files.get("st_test.json").expect("Expected the file");

  assert_eq!(file.entries.len(), 1);
  // Both languages are written back to the same JSON, so both point at it.
  assert_eq!(file.sources.len(), 2);
}

#[test]
fn opens_a_project_whose_json_is_broken_and_reports_it() {
  let root: &str = "source_read/broken_json";

  write_generated_test_resource(&format!("{root}/good.json"), r#"{"st_hello":{"eng":"Hello"}}"#)
    .expect("Expected a written test file");
  write_generated_test_resource(&format!("{root}/bad.json"), "{ not json").expect("Expected a written test file");

  let descriptor = read_source(&roots(root), "").unwrap();

  assert!(descriptor.files.contains_key("good.json"));
  assert!(
    descriptor
      .findings
      .iter()
      .any(|finding| finding.rule == "translations.unreadable")
  );
}

#[test]
fn reports_an_id_defined_in_two_files_instead_of_refusing_to_open() {
  let root: &str = "source_read/cross_file";

  write_generated_test_resource(&format!("{root}/first.json"), r#"{"st_same":{"eng":"first"}}"#)
    .expect("Expected a written test file");
  write_generated_test_resource(&format!("{root}/second.json"), r#"{"st_same":{"eng":"second"}}"#)
    .expect("Expected a written test file");

  let descriptor = read_source(&roots(root), "").unwrap();

  assert_eq!(descriptor.files.len(), 2);
  assert!(
    descriptor
      .findings
      .iter()
      .any(|finding| finding.rule == "translations.duplicate-across-files")
  );
}

#[test]
fn records_the_code_page_each_language_will_be_built_in() {
  let root: &str = "source_read/encodings";

  write_generated_test_resource(&format!("{root}/st_test.json"), r#"{"st_hello":{"eng":"a","ukr":"b"}}"#)
    .expect("Expected a written test file");

  let descriptor = read_source(&roots(root), "").unwrap();

  assert_eq!(
    descriptor.encodings.get("eng").map(String::as_str),
    Some("windows-1252")
  );
  assert_eq!(
    descriptor.encodings.get("ukr").map(String::as_str),
    Some("windows-1251")
  );
}
