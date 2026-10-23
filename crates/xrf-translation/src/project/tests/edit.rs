use std::collections::HashMap;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use xrf_error::{XrfError, XrfResult};
use xrf_test_utils::utils::write_generated_test_resource;
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayLogicalPath, XrayLookupScope, XraySearch, XrayVfs};

use crate::edit::TranslationEdit;
use crate::json::read::read_json;
use crate::project::descriptor::{TranslationFile, TranslationProjectDescriptor, TranslationSource};
use crate::project::edit::{apply_edits, build_save_transaction, find_unwritable_character, render_edits};
use crate::types::TranslationVariant;
use crate::xml::read::read_string_table;

fn descriptor(encodings: &[(&str, &str)]) -> TranslationProjectDescriptor {
  TranslationProjectDescriptor {
    encodings: encodings
      .iter()
      .map(|(language, encoding)| (String::from(*language), String::from(*encoding)))
      .collect::<IndexMap<String, String>>(),
    ..Default::default()
  }
}

fn set(id: &str, text: &str) -> TranslationEdit {
  TranslationEdit::Set {
    id: String::from(id),
    value: TranslationVariant::String(String::from(text)),
  }
}

#[test]
fn routes_a_json_source_to_the_json_writer() -> XrfResult {
  let path = write_generated_test_resource("project_edit/dispatch.json", r#"{"st_a":{"eng":"A"}}"#)?;

  apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    read_json(&path)?["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("B")))
  );

  Ok(())
}

#[test]
fn routes_an_xml_source_to_the_splice_writer() -> XrfResult {
  let path = write_generated_test_resource(
    "project_edit/dispatch.xml",
    "<string_table><string id=\"st_a\"><text>A</text></string></string_table>",
  )?;

  apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    read_string_table(&path)?,
    vec![(String::from("st_a"), String::from("B"))]
  );

  Ok(())
}

/// The dispatch compared `Path::extension` against a lower-case spelling, so a source the author named in upper case
/// was reported as a file this cannot write - while every reader in the crate opened it without complaint.
#[test]
fn routes_a_source_whatever_case_its_name_was_typed_in() -> XrfResult {
  let path: PathBuf = write_generated_test_resource("project_edit/AUTHORED_CASE.JSON", r#"{"st_a":{"eng":"A"}}"#)?;

  apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    read_json(&path)?["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("B")))
  );

  let path: PathBuf = write_generated_test_resource(
    "project_edit/AUTHORED_CASE.XML",
    "<string_table><string id=\"st_a\"><text>A</text></string></string_table>",
  )?;

  apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    read_string_table(&path)?,
    vec![(String::from("st_a"), String::from("B"))]
  );

  Ok(())
}

/// And a name that merely ends with the letters is still not a translation, which is the splitter's rule.
#[test]
fn refuses_a_name_that_merely_ends_with_a_supported_extension() {
  let error = apply_edits(Path::new("translations/notesxml"), "eng", &[]).unwrap_err();

  assert!(error.to_string().contains("not a file this can write"));
}

#[test]
fn refuses_a_file_it_has_no_writer_for() {
  let error = apply_edits(Path::new("translations/notes.txt"), "eng", &[]).unwrap_err();

  assert!(error.to_string().contains("not a file this can write"));
}

#[test]
fn accepts_text_the_target_code_page_can_hold() -> XrfResult {
  let project = descriptor(&[("rus", "windows-1251")]);

  assert_eq!(find_unwritable_character(&project, "rus", "Привет")?, None);

  Ok(())
}

#[test]
fn names_the_character_a_target_cannot_hold() -> XrfResult {
  let project = descriptor(&[("fra", "windows-1252")]);
  let reported = find_unwritable_character(&project, "fra", "Привет")?.expect("Expected a refusal");

  assert!(reported.contains("U+041F"));
  assert!(reported.contains("fra"));

  Ok(())
}

#[test]
fn a_language_with_no_recorded_encoding_is_not_second_guessed() -> XrfResult {
  let project = descriptor(&[]);

  // Nothing was read for it, so there is no code page to judge against and nothing to report.
  assert_eq!(find_unwritable_character(&project, "jpn", "日本語")?, None);

  Ok(())
}

#[test]
fn renders_a_json_source_edited_as_apply_edits_writes_it() -> XrfResult {
  let existing: &str = "{\r\n  \"st_a\": {\r\n    \"eng\": \"A\"\r\n  }\r\n}\r\n";
  let path = write_generated_test_resource("project_edit/render.json", existing)?;

  let rendered: Vec<u8> = render_edits(
    &XrayLogicalPath::new(r"translations\render.json")?,
    existing.as_bytes(),
    "eng",
    &[set("st_a", "B")],
  )?;

  apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    rendered,
    std::fs::read(&path)?,
    "a commit writes what an in-place save did"
  );
  assert!(
    String::from_utf8_lossy(&rendered).contains("\r\n"),
    "the file's line endings stay"
  );

  Ok(())
}

#[test]
fn renders_a_string_table_edited_leaving_every_untouched_byte() -> XrfResult {
  let existing: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"windows-1251\"?>\n",
    "<string_table>\n",
    "  <string id=\"st_a\">\n    <text>A</text>\n  </string>\n",
    "  <!-- kept -->\n",
    "</string_table>\n"
  );

  let rendered: String = String::from_utf8(render_edits(
    &XrayLogicalPath::new(r"configs\text\eng\st_render.xml")?,
    existing.as_bytes(),
    "eng",
    &[set("st_a", "B")],
  )?)
  .expect("the table stays ASCII");

  assert_eq!(rendered, existing.replace("<text>A</text>", "<text>B</text>"));

  Ok(())
}

/// A search over one loose tree holding a two-language JSON source, with the source each language reads.
fn json_project(name: &str) -> XrfResult<(PathBuf, XraySearch, HashMap<String, TranslationSource>)> {
  let file: PathBuf = write_generated_test_resource(
    &format!("project_edit/{name}/translations/st_save.json"),
    r#"{"st_a":{"eng":"A","rus":"А"}}"#,
  )?;
  let root: &Path = file
    .parent()
    .and_then(Path::parent)
    .expect("the fixture sits two directories down");
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", root)?;

  let source: TranslationSource = TranslationSource::new(r"translations\st_save.json", None);

  Ok((
    file,
    XraySearch::new(vfs).with_step("project", XrayLookupScope::all()),
    HashMap::from([(String::from("eng"), source.clone()), (String::from("rus"), source)]),
  ))
}

#[test]
fn saves_every_language_of_one_json_source_in_one_transaction() -> XrfResult {
  let (file, search, sources) = json_project("save_languages")?;
  let edits: HashMap<String, Vec<TranslationEdit>> = HashMap::from([
    (String::from("eng"), vec![set("st_a", "B")]),
    (String::from("rus"), vec![set("st_a", "Б")]),
  ]);

  let transaction = build_save_transaction(&search, "st_save.json", &sources, &edits)?;

  // Nothing is written until the transaction commits.
  assert_eq!(
    read_json(&file)?["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("A")))
  );

  search.get_vfs().commit(&transaction)?;

  let saved = read_json(&file)?;

  assert_eq!(
    saved["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("B")))
  );
  assert_eq!(
    saved["st_a"]["rus"],
    Some(TranslationVariant::String(String::from("Б"))),
    "the second language applies over the first rather than replacing it"
  );

  Ok(())
}

#[test]
fn refuses_to_save_a_language_with_no_copy() -> XrfResult {
  let (_, search, sources) = json_project("save_missing")?;
  let edits: HashMap<String, Vec<TranslationEdit>> = HashMap::from([(String::from("ukr"), vec![set("st_a", "Б")])]);

  let error: XrfError = build_save_transaction(&search, "st_save.json", &sources, &edits).unwrap_err();

  assert!(error.to_string().contains("nothing on disk for 'ukr'"), "{error}");

  Ok(())
}

#[test]
fn refuses_to_render_a_file_it_cannot_write() -> XrfResult {
  let error = render_edits(&XrayLogicalPath::new(r"configs\system.ltx")?, b"", "eng", &[]).unwrap_err();

  assert!(error.to_string().contains(r"configs\system.ltx"), "{error}");

  Ok(())
}

#[test]
fn a_recorded_physical_path_is_a_display_form_and_not_a_write_address() {
  // What this pass fixed. The descriptor renders the host path portably, which is lossy in two ways:
  // a name that is not valid Unicode becomes replacement characters, and `\` — an ordinary filename
  // character on a Unix host — becomes a separator. Writing through this string would land elsewhere.
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
