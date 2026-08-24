//! Chooses which LTX files a set of paths means, and what cannot be formatted.

use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;

use crate::commands::ltx::format::ltx_format_selection::LtxFormatSelection;

fn create_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = std::env::temp_dir().join(format!("xrf-cli-format-ltx-{name}-{}", std::process::id()));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  Ok(root)
}

fn create_installation(name: &str, configs: &[&str]) -> XrfResult<PathBuf> {
  let root: PathBuf = create_root(name)?;

  fs::write(
    root.join("fsgame.ltx"),
    "$arch_dir$ = false | false | $fs_root$ | db\\\n$game_data$ = true | true | $fs_root$ | gamedata\\\n",
  )?;

  for config in configs {
    let path: PathBuf = root.join("gamedata").join(config.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("config parent"))?;
    fs::write(&path, "[a]\n")?;
  }

  Ok(root)
}

#[test]
fn collects_ltx_files_from_folder_recursively() -> XrfResult {
  let root: PathBuf = create_root("folder")?;
  let nested: PathBuf = root.join("nested");

  fs::create_dir_all(&nested)?;
  fs::write(root.join("first.ltx"), "[a]\n")?;
  fs::write(nested.join("second.ltx"), "[b]\n")?;
  fs::write(root.join("ignored.txt"), "text")?;

  let files: Vec<PathBuf> = LtxFormatSelection::select(&[&root])?.files;

  assert_eq!(files, vec![root.join("first.ltx"), nested.join("second.ltx")]);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn collects_explicitly_provided_files_regardless_of_extension() -> XrfResult {
  let root: PathBuf = create_root("explicit")?;
  let ltx: PathBuf = root.join("first.ltx");
  let ini: PathBuf = root.join("second.ini");

  fs::write(&ltx, "[a]\n")?;
  fs::write(&ini, "[b]\n")?;

  let files: Vec<PathBuf> = LtxFormatSelection::select(&[&ltx, &ini])?.files;

  assert_eq!(files, vec![ltx, ini]);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn de_duplicates_mixed_folder_and_file_paths() -> XrfResult {
  let root: PathBuf = create_root("mixed")?;
  let first: PathBuf = root.join("first.ltx");

  fs::write(&first, "[a]\n")?;
  fs::write(root.join("second.ltx"), "[b]\n")?;

  let files: Vec<PathBuf> = LtxFormatSelection::select(&[&root, &first])?.files;

  assert_eq!(files, vec![first, root.join("second.ltx")]);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn sorts_multiple_explicit_paths() -> XrfResult {
  let root: PathBuf = create_root("explicit-sorted")?;
  let first: PathBuf = root.join("first.ltx");
  let second: PathBuf = root.join("second.ltx");

  fs::write(&first, "[a]\n")?;
  fs::write(&second, "[b]\n")?;

  let files: Vec<PathBuf> = LtxFormatSelection::select(&[&second, &first])?.files;

  assert_eq!(files, vec![first, second]);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn fails_on_missing_path() -> XrfResult {
  let root: PathBuf = create_root("missing")?;
  let missing: PathBuf = root.join("absent.ltx");

  assert!(LtxFormatSelection::select(&[&missing]).is_err());

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn selects_an_installation_through_the_vfs_and_finds_its_loose_configs() -> XrfResult {
  let root: PathBuf = create_installation("installation", &["configs\\system.ltx", "configs\\weapons\\ak74.ltx"])?;

  let selection: LtxFormatSelection = LtxFormatSelection::select(&[&root])?;

  assert_eq!(selection.files.len(), 2);
  assert!(
    selection.declined.is_empty(),
    "nothing is archived in this installation"
  );
  assert!(
    selection
      .files
      .iter()
      .all(|file| file.starts_with(root.join("gamedata"))),
    "only loose gamedata files are selected"
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn ignores_non_ltx_entries_of_an_installation() -> XrfResult {
  let root: PathBuf = create_installation("installation-mixed", &["configs\\system.ltx"])?;

  fs::create_dir_all(root.join("gamedata/textures"))?;
  fs::write(root.join("gamedata/textures/wpn.dds"), [0u8; 4])?;

  let selection: LtxFormatSelection = LtxFormatSelection::select(&[&root])?;

  assert_eq!(selection.files.len(), 1);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn an_installation_with_no_configs_selects_nothing_rather_than_failing() -> XrfResult {
  let root: PathBuf = create_installation("installation-empty", &[])?;

  fs::create_dir_all(root.join("gamedata"))?;

  assert!(LtxFormatSelection::select(&[&root])?.is_empty());

  fs::remove_dir_all(root)?;

  Ok(())
}
