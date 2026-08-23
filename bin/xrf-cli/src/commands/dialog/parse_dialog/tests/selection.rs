//! Which files a sweep picks up.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_report::Status;
use xrf_vfs::{XrayMountMode, XrayWorldSpec};

use crate::commands::dialog::parse_dialog::dialog_sweep::{DialogSweep, DialogSweepResult};

const DIALOG: &str = r#"<game_dialogs><dialog id="d"><phrase_list><phrase id="0"><text>key</text></phrase></phrase_list></dialog></game_dialogs>"#;

/// A loose temp root declares no installation, so name the mode rather than letting Auto search upward.
fn world(root: &Path) -> XrayWorldSpec {
  XrayWorldSpec::root(root.display().to_string(), XrayMountMode::Directory)
}

fn create_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = std::env::temp_dir().join(format!("xrf-cli-parse-dialog-{name}-{}", std::process::id()));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  Ok(root)
}

#[test]
fn sweeps_only_dialog_named_xml_in_a_directory() -> XrfResult {
  let root: PathBuf = create_root("selection")?;

  fs::write(root.join("dialogs.xml"), DIALOG)?;
  fs::write(root.join("dialogs_zaton.xml"), DIALOG)?;
  // A gameplay directory holds these beside the dialogs, and neither is dialog data.
  fs::write(root.join("info_zaton.xml"), "<game_information_portions/>")?;
  fs::write(root.join("npc_profile.xml"), "<game_profile_list/>")?;
  fs::write(root.join("dialogs.ltx"), "[section]")?;

  let result: DialogSweepResult = DialogSweep::new(&world(&root), None).run()?;

  assert_eq!(result.census.files, 2);
  assert_eq!(result.census.dialogs, 2);
  assert!(result.report.checks().iter().all(|check| check.findings().is_empty()));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn walks_nested_directories() -> XrfResult {
  let root: PathBuf = create_root("nested")?;
  let nested: PathBuf = root.join("gameplay");

  fs::create_dir_all(&nested)?;
  fs::write(root.join("dialogs.xml"), DIALOG)?;
  fs::write(nested.join("dialogs_jupiter.xml"), DIALOG)?;

  let result: DialogSweepResult = DialogSweep::new(&world(&root), None).run()?;

  assert_eq!(result.census.files, 2);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn skips_rather_than_passes_a_directory_with_no_dialogs() -> XrfResult {
  let root: PathBuf = create_root("empty")?;

  fs::write(root.join("info_zaton.xml"), "<game_information_portions/>")?;

  let result: DialogSweepResult = DialogSweep::new(&world(&root), None).run()?;

  assert_eq!(result.census.files, 0);
  assert_eq!(result.census.dialogs, 0);
  // Nothing was read, so nothing was judged. Reporting this as a pass is how a mistyped path gets
  // wired into CI and silently checks nothing.
  assert_eq!(result.report.status(), Status::Skipped);
  assert!(result.report.checks().iter().all(|check| check.findings().is_empty()));

  fs::remove_dir_all(root)?;

  Ok(())
}
