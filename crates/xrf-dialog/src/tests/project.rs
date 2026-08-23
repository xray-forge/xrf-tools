use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_vfs::XrayMountMode;

use crate::project::descriptor::DialogProjectDescriptor;
use crate::project::dialog_project::DialogProject;
use crate::project::layout::detect_mode;
use crate::project::mode::DialogProjectMode;
use crate::project::options::DialogProjectOptions;

const DIALOG: &str = r#"<game_dialogs><dialog id="d" priority="-5"><phrase_list><phrase id="0"><text>key</text></phrase></phrase_list></dialog></game_dialogs>"#;

fn create_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = std::env::temp_dir().join(format!("xrf-dialog-project-{name}-{}", std::process::id()));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  Ok(root)
}

/// Lay out a gamedata-shaped tree: dialogs and text both under `configs`.
fn create_gamedata(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = create_root(name)?;
  let gameplay: PathBuf = root.join("configs").join("gameplay");
  let text: PathBuf = root.join("configs").join("text").join("rus");

  fs::create_dir_all(&gameplay)?;
  fs::create_dir_all(&text)?;
  fs::write(gameplay.join("dialogs.xml"), DIALOG)?;
  fs::write(text.join("st_dialogs.xml"), "<string_table/>")?;

  Ok(root)
}

/// Lay out a source-shaped tree: `configs/gameplay` and `translations` as siblings.
fn create_source(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = create_root(name)?;
  let gameplay: PathBuf = root.join("configs").join("gameplay");
  let translations: PathBuf = root.join("translations");

  fs::create_dir_all(&gameplay)?;
  fs::create_dir_all(&translations)?;
  fs::write(gameplay.join("dialogs.xml"), DIALOG)?;
  fs::write(translations.join("st_dialogs.json"), "{}")?;

  Ok(root)
}

fn open(root: &PathBuf, mode: DialogProjectMode) -> XrfResult<DialogProject> {
  DialogProject::open_with_mode(XrayMountMode::Directory, &DialogProjectOptions::new(root, mode))
}

#[test]
fn defaults_both_prefixes_from_the_mode() {
  let gamedata: DialogProjectOptions = DialogProjectOptions::new("root", DialogProjectMode::Gamedata);
  let source: DialogProjectOptions = DialogProjectOptions::new("root", DialogProjectMode::Source);

  // The one thing the layouts agree on.
  assert_eq!(gamedata.get_dialogs_prefix(), r"configs\gameplay");
  assert_eq!(source.get_dialogs_prefix(), r"configs\gameplay");

  // And the one they do not.
  assert_eq!(gamedata.get_translations_prefix(), r"configs\text");
  assert_eq!(source.get_translations_prefix(), "translations");
}

#[test]
fn takes_a_prefix_override() {
  // A mod keeping its dialogs somewhere the layout does not predict.
  let options: DialogProjectOptions = DialogProjectOptions {
    dialogs_prefix: Some(String::from(r"custom\talks")),
    translations_prefix: Some(String::from("strings")),
    ..DialogProjectOptions::new("root", DialogProjectMode::Gamedata)
  };

  assert_eq!(options.get_dialogs_prefix(), r"custom\talks");
  assert_eq!(options.get_translations_prefix(), "strings");
}

#[test]
fn recognises_a_dialog_logical_path_by_its_file_name() {
  assert!(DialogProject::is_dialog_logical_path(r"configs\gameplay\dialogs.xml"));
  assert!(DialogProject::is_dialog_logical_path(
    r"configs\gameplay\dialogs_zaton.xml"
  ));
  assert!(DialogProject::is_dialog_logical_path("dialogs.xml"));

  // Everything a gameplay directory holds beside the dialogs.
  assert!(!DialogProject::is_dialog_logical_path(
    r"configs\gameplay\info_zaton.xml"
  ));
  assert!(!DialogProject::is_dialog_logical_path(
    r"configs\gameplay\npc_profile.xml"
  ));
  assert!(!DialogProject::is_dialog_logical_path(r"configs\gameplay\dialogs.ltx"));
  // Not a directory whose name merely starts the same way.
  assert!(!DialogProject::is_dialog_logical_path(r"dialogs\something.ltx"));
}

#[test]
fn detects_source_from_a_translations_prefix_holding_json() -> XrfResult {
  let root: PathBuf = create_source("detect-source")?;

  assert_eq!(detect_mode(XrayMountMode::Directory, &root)?, DialogProjectMode::Source);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn detects_gamedata_for_anything_else() -> XrfResult {
  let gamedata: PathBuf = create_gamedata("detect-gamedata")?;
  let empty: PathBuf = create_root("detect-empty")?;

  assert_eq!(
    detect_mode(XrayMountMode::Directory, &gamedata)?,
    DialogProjectMode::Gamedata
  );
  // A world the heuristic cannot make sense of reads as the mode the tooling targets.
  assert_eq!(
    detect_mode(XrayMountMode::Directory, &empty)?,
    DialogProjectMode::Gamedata
  );

  fs::remove_dir_all(gamedata)?;
  fs::remove_dir_all(empty)?;

  Ok(())
}

#[test]
fn opens_a_gamedata_tree_and_indexes_it_by_logical_path() -> XrfResult {
  let root: PathBuf = create_gamedata("open-gamedata")?;
  let project: DialogProject = open(&root, DialogProjectMode::Gamedata)?;

  assert_eq!(project.get_files().len(), 1);
  assert_eq!(project.sum_dialogs(), 1);
  assert_eq!(
    project.get_files()[0].get_logical_path(),
    r"configs\gameplay\dialogs.xml"
  );
  assert!(project.get_findings().is_empty());

  let descriptor: DialogProjectDescriptor = project.describe();

  assert_eq!(descriptor.mode, DialogProjectMode::Gamedata);
  assert_eq!(descriptor.dialogs_prefix, r"configs\gameplay");
  assert_eq!(descriptor.translations_prefix, r"configs\text");

  let file = descriptor
    .files
    .get(r"configs\gameplay\dialogs.xml")
    .expect("the file should be indexed by its logical path");

  assert_eq!(file.encoding, "windows-1251");
  assert_eq!(file.dialogs.len(), 1);
  assert_eq!(file.dialogs[0].id, "d");
  assert_eq!(file.dialogs[0].phrases, 1);
  assert_eq!(file.dialogs[0].priority, Some(-5));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn opens_the_same_dialogs_in_source_mode_with_a_different_text_prefix() -> XrfResult {
  let root: PathBuf = create_source("open-source")?;
  let descriptor: DialogProjectDescriptor = open(&root, DialogProjectMode::Source)?.describe();

  assert_eq!(descriptor.mode, DialogProjectMode::Source);
  assert_eq!(descriptor.dialogs_prefix, r"configs\gameplay");
  assert_eq!(descriptor.translations_prefix, "translations");
  assert!(descriptor.files.contains_key(r"configs\gameplay\dialogs.xml"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn reports_a_loose_tree_as_editable_with_a_physical_path_per_file() -> XrfResult {
  let root: PathBuf = create_gamedata("editable")?;
  let project: DialogProject = open(&root, DialogProjectMode::Gamedata)?;

  assert!(project.is_editable());
  assert!(project.get_files()[0].is_editable());
  assert!(project.get_files()[0].get_physical_path().is_some());
  assert!(project.describe().is_editable);
  assert!(
    project.describe().files[r"configs\gameplay\dialogs.xml"]
      .physical_path
      .is_some()
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn refuses_a_world_exposing_no_dialog_files() -> XrfResult {
  // Answering with an empty project would hide that the caller named the wrong place.
  let root: PathBuf = create_root("no-dialog-files")?;
  let gameplay: PathBuf = root.join("configs").join("gameplay");

  fs::create_dir_all(&gameplay)?;
  fs::write(gameplay.join("info_zaton.xml"), "<game_information_portions/>")?;

  // Mapped to unit so the assertion does not force `Debug` onto a project holding a mounted world.
  let error = open(&root, DialogProjectMode::Gamedata)
    .map(|_| ())
    .expect_err("a world with no dialogs should be refused");

  assert!(error.to_string().contains("No dialog files under"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn opens_despite_one_unreadable_file_and_reports_it() -> XrfResult {
  let root: PathBuf = create_gamedata("partial")?;
  let gameplay: PathBuf = root.join("configs").join("gameplay");

  fs::write(gameplay.join("dialogs_broken.xml"), "<game_dialogs><dialog id=\"d\">")?;

  let project: DialogProject = open(&root, DialogProjectMode::Gamedata)?;

  // The readable file still opened, which is the point: an editor that refuses the tree over one bad
  // file cannot reach the file you opened it to fix.
  assert_eq!(project.get_files().len(), 1);
  assert_eq!(project.get_findings().len(), 1);
  assert_eq!(project.get_findings()[0].rule, "dialog.unreadable");
  assert_eq!(
    project.get_findings()[0].subject.as_deref(),
    Some(r"configs\gameplay\dialogs_broken.xml")
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn reports_an_off_schema_element_as_a_project_finding() -> XrfResult {
  let root: PathBuf = create_root("schema")?;
  let gameplay: PathBuf = root.join("configs").join("gameplay");

  fs::create_dir_all(&gameplay)?;
  fs::write(
    gameplay.join("dialogs.xml"),
    r#"<game_dialogs><dialog id="d"><phrase_list><phrase id="0"><go_back>1</go_back></phrase></phrase_list></dialog></game_dialogs>"#,
  )?;

  let project: DialogProject = open(&root, DialogProjectMode::Gamedata)?;

  assert_eq!(project.get_findings().len(), 1);
  assert_eq!(project.get_findings()[0].rule, "dialog.schema");
  assert!(project.get_findings()[0].message.contains("go_back"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn finds_a_file_by_logical_path_regardless_of_case() -> XrfResult {
  // Logical paths are lower case by definition, but a caller echoing a user's typing is not.
  let root: PathBuf = create_gamedata("case")?;
  let project: DialogProject = open(&root, DialogProjectMode::Gamedata)?;

  assert!(project.find_file(r"configs\gameplay\dialogs.xml").is_some());
  assert!(project.find_file(r"CONFIGS\GAMEPLAY\DIALOGS.XML").is_some());
  assert!(project.find_file(r"configs\gameplay\nothing.xml").is_none());

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn keeps_nested_files_under_their_logical_path() -> XrfResult {
  let root: PathBuf = create_gamedata("nested")?;
  let nested: PathBuf = root.join("configs").join("gameplay").join("extra");

  fs::create_dir_all(&nested)?;
  fs::write(nested.join("dialogs_extra.xml"), DIALOG)?;

  let descriptor: DialogProjectDescriptor = open(&root, DialogProjectMode::Gamedata)?.describe();

  assert!(descriptor.files.contains_key(r"configs\gameplay\dialogs.xml"));
  assert!(
    descriptor
      .files
      .contains_key(r"configs\gameplay\extra\dialogs_extra.xml")
  );

  fs::remove_dir_all(root)?;

  Ok(())
}
