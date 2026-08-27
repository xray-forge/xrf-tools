use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayAsset, XrayAssetContainer, XrayLogicalPath, require_writable_path};

use crate::edit::TranslationEdit;
use crate::json::read::read_json;
use crate::project::descriptor::{TranslationFile, TranslationProjectDescriptor, TranslationSource};
use crate::project::edit::{apply_edits, apply_edits_to_asset, find_unwritable_character};
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

/// A loose asset at a host path, as a mount would have resolved one.
fn loose_asset(logical_path: &str, root: &Path, relative_path: &str) -> XrfResult<XrayAsset> {
  Ok(XrayAsset::new(
    XrayLogicalPath::new(logical_path)?,
    XrayAssetContainer::Directory {
      root: root.to_path_buf(),
      relative_path: PathBuf::from(relative_path),
    },
  ))
}

#[test]
fn writes_through_the_asset_the_mount_resolved() -> XrfResult {
  let path = write_generated_test_resource("project_edit/source_loose.json", r#"{"st_a":{"eng":"A"}}"#)?;
  let root: &Path = path.parent().expect("the fixture sits in a directory");
  let asset: XrayAsset = loose_asset(r"translations\source_loose.json", root, "source_loose.json")?;

  apply_edits_to_asset(&asset, "eng", &[set("st_a", "B")])?;

  assert_eq!(
    read_json(&path)?["st_a"]["eng"],
    Some(TranslationVariant::String(String::from("B")))
  );

  Ok(())
}

#[test]
fn refuses_to_edit_an_asset_that_came_out_of_an_archive() -> XrfResult {
  // The guard is the absence of a physical path, not a test of which kind of mount won: there is no
  // file inside a volume to replace, so this is refused before anything is written rather than
  // failing partway through.
  let asset: XrayAsset = XrayAsset::new(
    XrayLogicalPath::new(r"configs\text\rus\st_dialogs.xml")?,
    XrayAssetContainer::Archive {
      path: PathBuf::from("db"),
    },
  );

  let error = apply_edits_to_asset(&asset, "rus", &[]).unwrap_err();

  assert!(error.to_string().contains("read out of an archive"));
  // Named, so a caller can say which file refused rather than reporting the save as generally failed.
  assert!(error.to_string().contains(r"configs\text\rus\st_dialogs.xml"));
  // The wording is `xrf-vfs`'s, shared with every other editing domain, so a surface can recognise
  // this refusal without knowing which crate raised it.
  assert_eq!(
    error.to_string(),
    require_writable_path(r"configs\text\rus\st_dialogs.xml", None)
      .unwrap_err()
      .to_string()
  );

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
