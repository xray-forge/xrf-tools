use std::path::PathBuf;

use indexmap::IndexMap;
use xrf_error::XrfResult;
use xrf_utils::to_portable_path_string;

use crate::project::describe::translation_file::TranslationFile;
use crate::project::describe::translation_project_descriptor::TranslationProjectDescriptor;
use crate::project::describe::translation_source::TranslationSource;

fn descriptor(encodings: &[(&str, &str)]) -> TranslationProjectDescriptor {
  TranslationProjectDescriptor {
    encodings: encodings
      .iter()
      .map(|(language, encoding)| (String::from(*language), String::from(*encoding)))
      .collect::<IndexMap<String, String>>(),
    ..Default::default()
  }
}

#[test]
fn accepts_text_the_target_code_page_can_hold() -> XrfResult {
  let project = descriptor(&[("rus", "windows-1251")]);

  assert_eq!(project.find_unwritable("rus", "Привет")?, None);

  Ok(())
}

#[test]
fn names_the_character_a_target_cannot_hold() -> XrfResult {
  let project = descriptor(&[("fra", "windows-1252")]);
  let reported = project.find_unwritable("fra", "Привет")?.expect("Expected a refusal");

  assert!(reported.contains("U+041F"));
  assert!(reported.contains("fra"));

  Ok(())
}

#[test]
fn refuses_a_text_too_long_for_the_line_a_table_holds_it_on() -> XrfResult {
  let project = descriptor(&[("eng", "windows-1251")]);
  let reported = project
    .find_unwritable("eng", &"a".repeat(4078))?
    .expect("Expected a refusal");

  assert!(reported.contains("4095 bytes"), "{reported}");
  // Escaping counts: an ampersand takes five bytes on its line.
  assert!(project.find_unwritable("eng", &"a".repeat(4077))?.is_none());
  assert!(
    project
      .find_unwritable("eng", &format!("&{}", "a".repeat(4073)))?
      .is_some()
  );

  Ok(())
}

#[test]
fn a_language_with_no_recorded_encoding_is_not_second_guessed() -> XrfResult {
  let project = descriptor(&[]);

  // Nothing was read for it, so there is no code page to judge against and nothing to report.
  assert_eq!(project.find_unwritable("jpn", "日本語")?, None);

  Ok(())
}

#[test]
fn a_recorded_physical_path_is_a_display_form_and_not_a_write_address() {
  // The descriptor renders the host path portably, which is lossy in two ways: a name that is not valid Unicode
  // becomes replacement characters, and `\` — an ordinary filename character on a Unix host — becomes a separator.
  // Writing through this string would land elsewhere.
  let source: TranslationSource = TranslationSource::new(
    r"translations\st_a.json",
    Some(to_portable_path_string(PathBuf::from("weird").join("a\\b.json"))),
  );

  assert!(
    source.physical_path.as_deref().is_some_and(|path| path.contains('/')),
    "the recorded form is separator-normalized, so it cannot round-trip back to the original name"
  );
  // Editability is all this field decides. The address comes from the asset.
  assert!(source.is_editable());
}

#[test]
fn reports_a_project_as_editable_only_when_every_source_has_a_file() {
  let loose: TranslationSource = TranslationSource::new("a.xml", Some(String::from("C:/a.xml")));
  let archived: TranslationSource = TranslationSource::new("b.xml", None);

  assert!(loose.is_editable());
  assert!(!archived.is_editable());

  let mut project: TranslationProjectDescriptor = TranslationProjectDescriptor::default();

  // An empty project is not editable: there is nothing to write, and reporting it as ready to save
  // would let a surface offer a save that can do nothing.
  project.finalize_editable();
  assert!(!project.is_editable);

  project.files.insert(
    String::from("st_a.xml"),
    TranslationFile {
      sources: [(String::from("eng"), loose.clone())].into_iter().collect(),
      ..Default::default()
    },
  );
  project.finalize_editable();
  assert!(project.is_editable);

  // One archived language is enough to make the whole project unsavable, because editability is per
  // language and the rolled-up flag is what a surface disables the save button on.
  project.files["st_a.xml"].sources.insert(String::from("rus"), archived);
  project.finalize_editable();
  assert!(!project.is_editable);
}
