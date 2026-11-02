use std::collections::HashMap;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayLogicalPath, XrayLookupScope, XraySearch, XrayVfs};

use crate::json::translation_json_reader::TranslationJsonReader;
use crate::project::describe::translation_source::TranslationSource;
use crate::project::translation_editor::TranslationEditor;
use crate::text::translation_edit::TranslationEdit;
use crate::text::translation_variant::TranslationVariant;
use crate::xml::translation_xml_reader::TranslationXmlReader;

fn set(id: &str, text: &str) -> TranslationEdit {
  TranslationEdit::Set {
    id: String::from(id),
    value: TranslationVariant::String(String::from(text)),
  }
}

#[test]
fn routes_a_json_source_to_the_json_writer() -> XrfResult {
  let path = GeneratedResources::write("project_edit/dispatch.json", r#"{"st_a":{"eng":"A"}}"#)?;

  TranslationEditor::apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    TranslationJsonReader::read(&path)?["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("B")))
  );

  Ok(())
}

#[test]
fn routes_an_xml_source_to_the_splice_writer() -> XrfResult {
  let path = GeneratedResources::write(
    "project_edit/dispatch.xml",
    "<string_table><string id=\"st_a\"><text>A</text></string></string_table>",
  )?;

  TranslationEditor::apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_a"), String::from("B"))]
  );

  Ok(())
}

/// The dispatch compared `Path::extension` against a lower-case spelling, so a source the author named in upper case
/// was reported as a file this cannot write - while every reader in the crate opened it without complaint.
#[test]
fn routes_a_source_whatever_case_its_name_was_typed_in() -> XrfResult {
  let path: PathBuf = GeneratedResources::write("project_edit/AUTHORED_CASE.JSON", r#"{"st_a":{"eng":"A"}}"#)?;

  TranslationEditor::apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    TranslationJsonReader::read(&path)?["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("B")))
  );

  let path: PathBuf = GeneratedResources::write(
    "project_edit/AUTHORED_CASE.XML",
    "<string_table><string id=\"st_a\"><text>A</text></string></string_table>",
  )?;

  TranslationEditor::apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    TranslationXmlReader::read(&path)?,
    vec![(String::from("st_a"), String::from("B"))]
  );

  Ok(())
}

/// And a name that merely ends with the letters is still not a translation, which is the splitter's rule.
#[test]
fn refuses_a_name_that_merely_ends_with_a_supported_extension() {
  let error = TranslationEditor::apply_edits(Path::new("translations/notesxml"), "eng", &[]).unwrap_err();

  assert!(error.to_string().contains("not a file this can write"));
}

#[test]
fn refuses_a_file_it_has_no_writer_for() {
  let error = TranslationEditor::apply_edits(Path::new("translations/notes.txt"), "eng", &[]).unwrap_err();

  assert!(error.to_string().contains("not a file this can write"));
}

#[test]
fn renders_a_json_source_edited_as_apply_edits_writes_it() -> XrfResult {
  let existing: &str = "{\r\n  \"st_a\": {\r\n    \"eng\": \"A\"\r\n  }\r\n}\r\n";
  let path = GeneratedResources::write("project_edit/render.json", existing)?;

  let rendered: Vec<u8> = TranslationEditor::render_edits(
    &XrayLogicalPath::new(r"translations\render.json")?,
    existing.as_bytes(),
    "eng",
    &[set("st_a", "B")],
  )?;

  TranslationEditor::apply_edits(&path, "eng", &[set("st_a", "B")])?;

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

  let rendered: String = String::from_utf8(TranslationEditor::render_edits(
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
  let file: PathBuf = GeneratedResources::write(
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

  let transaction = TranslationEditor::build_save_transaction(&search, "st_save.json", &sources, &edits)?;

  // Nothing is written until the transaction commits.
  assert_eq!(
    TranslationJsonReader::read(&file)?["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("A")))
  );

  search.get_vfs().commit(&transaction)?;

  let saved = TranslationJsonReader::read(&file)?;

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

  let error: XrfError =
    TranslationEditor::build_save_transaction(&search, "st_save.json", &sources, &edits).unwrap_err();

  assert!(error.to_string().contains("nothing on disk for 'ukr'"), "{error}");

  Ok(())
}

#[test]
fn refuses_to_render_a_file_it_cannot_write() -> XrfResult {
  let error =
    TranslationEditor::render_edits(&XrayLogicalPath::new(r"configs\system.ltx")?, b"", "eng", &[]).unwrap_err();

  assert!(error.to_string().contains(r"configs\system.ltx"), "{error}");

  Ok(())
}
