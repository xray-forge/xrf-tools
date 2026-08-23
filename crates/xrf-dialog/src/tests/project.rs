use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;

use crate::project::descriptor::DialogProjectDescriptor;
use crate::project::dialog_project::DialogProject;
use crate::project::layout::detect_mode;
use crate::project::mode::DialogProjectMode;
use crate::project::roots::{DialogProjectOverrides, DialogProjectRoots};

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

#[test]
fn resolves_both_roots_under_configs_in_gamedata_mode() {
  let root: &Path = Path::new("C:");
  let roots: DialogProjectRoots =
    DialogProjectRoots::resolve(root, DialogProjectMode::Gamedata, &DialogProjectOverrides::default());

  assert_eq!(roots.get_dialogs(), root.join("configs").join("gameplay"));
  assert_eq!(roots.get_translations(), root.join("configs").join("text"));
}

#[test]
fn resolves_translations_as_a_sibling_in_source_mode() {
  // The difference that matters: one path cannot address both trees here.
  let root: &Path = Path::new("C:");
  let roots: DialogProjectRoots =
    DialogProjectRoots::resolve(root, DialogProjectMode::Source, &DialogProjectOverrides::default());

  assert_eq!(roots.get_dialogs(), root.join("configs").join("gameplay"));
  assert_eq!(roots.get_translations(), root.join("translations"));
}

#[test]
fn takes_a_relative_override_against_the_root() {
  let root: &Path = Path::new("C:");
  let roots: DialogProjectRoots = DialogProjectRoots::resolve(
    root,
    DialogProjectMode::Gamedata,
    &DialogProjectOverrides {
      dialogs: Some(PathBuf::from("custom").join("talks")),
      translations: None,
    },
  );

  assert_eq!(roots.get_dialogs(), root.join("custom").join("talks"));
  assert_eq!(roots.get_translations(), root.join("configs").join("text"));
}

#[test]
fn takes_an_absolute_override_as_it_stands() {
  // A tree keeping its dialogs outside the project entirely still opens.
  let elsewhere: PathBuf = std::env::temp_dir().join("elsewhere");
  let roots: DialogProjectRoots = DialogProjectRoots::resolve(
    Path::new("C:"),
    DialogProjectMode::Gamedata,
    &DialogProjectOverrides {
      dialogs: Some(elsewhere.clone()),
      translations: None,
    },
  );

  assert_eq!(roots.get_dialogs(), elsewhere);
}

#[test]
fn detects_source_from_a_translations_directory_holding_json() -> XrfResult {
  let root: PathBuf = create_source("detect-source")?;

  assert_eq!(detect_mode(&root), DialogProjectMode::Source);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn detects_gamedata_for_anything_else() -> XrfResult {
  let gamedata: PathBuf = create_gamedata("detect-gamedata")?;
  let empty: PathBuf = create_root("detect-empty")?;

  assert_eq!(detect_mode(&gamedata), DialogProjectMode::Gamedata);
  // A root the heuristic cannot make sense of reads as the mode the tooling targets.
  assert_eq!(detect_mode(&empty), DialogProjectMode::Gamedata);
  assert_eq!(detect_mode(Path::new("nowhere-at-all")), DialogProjectMode::Gamedata);

  fs::remove_dir_all(gamedata)?;
  fs::remove_dir_all(empty)?;

  Ok(())
}

#[test]
fn opens_a_gamedata_tree_and_indexes_it() -> XrfResult {
  let root: PathBuf = create_gamedata("open-gamedata")?;
  let project: DialogProject =
    DialogProject::open(&root, DialogProjectMode::Gamedata, &DialogProjectOverrides::default())?;

  assert_eq!(project.get_files().len(), 1);
  assert_eq!(project.sum_dialogs(), 1);
  assert_eq!(project.get_files()[0].get_relative_path(), "dialogs.xml");
  assert!(project.get_findings().is_empty());

  let descriptor: DialogProjectDescriptor = project.describe();

  assert_eq!(descriptor.mode, DialogProjectMode::Gamedata);
  assert!(descriptor.dialogs_root.ends_with("configs/gameplay"));
  assert!(descriptor.translations_root.ends_with("configs/text"));

  let file = descriptor.files.get("dialogs.xml").expect("the file should be indexed");

  assert_eq!(file.encoding, "windows-1251");
  assert_eq!(file.dialogs.len(), 1);
  assert_eq!(file.dialogs[0].id, "d");
  assert_eq!(file.dialogs[0].phrases, 1);
  assert_eq!(file.dialogs[0].priority, Some(-5));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn opens_the_same_dialogs_in_source_mode_with_a_different_text_root() -> XrfResult {
  let root: PathBuf = create_source("open-source")?;
  let project: DialogProject =
    DialogProject::open(&root, DialogProjectMode::Source, &DialogProjectOverrides::default())?;

  let descriptor: DialogProjectDescriptor = project.describe();

  assert_eq!(descriptor.mode, DialogProjectMode::Source);
  assert!(descriptor.dialogs_root.ends_with("configs/gameplay"));
  assert!(descriptor.translations_root.ends_with("/translations"));
  assert_eq!(project.sum_dialogs(), 1);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn refuses_a_root_whose_dialogs_directory_is_absent() -> XrfResult {
  let root: PathBuf = create_root("no-dialogs-root")?;

  let error = DialogProject::open(&root, DialogProjectMode::Gamedata, &DialogProjectOverrides::default())
    .expect_err("an absent dialogs root should be refused");

  assert!(error.to_string().contains("Dialogs root does not exist"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn refuses_a_dialogs_directory_holding_none() -> XrfResult {
  // Answering with an empty project would hide that the caller named the wrong place.
  let root: PathBuf = create_root("no-dialog-files")?;
  let gameplay: PathBuf = root.join("configs").join("gameplay");

  fs::create_dir_all(&gameplay)?;
  fs::write(gameplay.join("info_zaton.xml"), "<game_information_portions/>")?;

  let error = DialogProject::open(&root, DialogProjectMode::Gamedata, &DialogProjectOverrides::default())
    .expect_err("a dialogs root with no dialogs should be refused");

  assert!(error.to_string().contains("No dialog files under"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn opens_despite_one_unreadable_file_and_reports_it() -> XrfResult {
  let root: PathBuf = create_gamedata("partial")?;
  let gameplay: PathBuf = root.join("configs").join("gameplay");

  fs::write(gameplay.join("dialogs_broken.xml"), "<game_dialogs><dialog id=\"d\">")?;

  let project: DialogProject =
    DialogProject::open(&root, DialogProjectMode::Gamedata, &DialogProjectOverrides::default())?;

  // The readable file still opened, which is the point: an editor that refuses the tree over one bad
  // file cannot reach the file you opened it to fix.
  assert_eq!(project.get_files().len(), 1);
  assert_eq!(project.get_findings().len(), 1);
  assert_eq!(project.get_findings()[0].rule, "dialog.unreadable");
  assert_eq!(project.get_findings()[0].subject.as_deref(), Some("dialogs_broken.xml"));

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

  let project: DialogProject =
    DialogProject::open(&root, DialogProjectMode::Gamedata, &DialogProjectOverrides::default())?;

  assert_eq!(project.get_findings().len(), 1);
  assert_eq!(project.get_findings()[0].rule, "dialog.schema");
  assert!(project.get_findings()[0].message.contains("go_back"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn keys_nested_files_by_their_path_under_the_dialogs_root() -> XrfResult {
  let root: PathBuf = create_gamedata("nested")?;
  let nested: PathBuf = root.join("configs").join("gameplay").join("extra");

  fs::create_dir_all(&nested)?;
  fs::write(nested.join("dialogs_extra.xml"), DIALOG)?;

  let project: DialogProject =
    DialogProject::open(&root, DialogProjectMode::Gamedata, &DialogProjectOverrides::default())?;
  let descriptor: DialogProjectDescriptor = project.describe();

  assert!(descriptor.files.contains_key("dialogs.xml"));
  assert!(descriptor.files.contains_key("extra/dialogs_extra.xml"));
  assert!(project.find_file("extra/dialogs_extra.xml").is_some());

  fs::remove_dir_all(root)?;

  Ok(())
}
