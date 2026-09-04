//! The dialect chosen through a real project over a real directory.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxDialect, LtxProject, LtxProjectOptions, LtxStandardDialect};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::XrayLogicalPath;

use crate::dltx_dialect::DltxDialect;

/// Writes a base config and one patch file beside it.
fn patched_tree(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("dltx_project/{name}"));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  fs::write(
    root.join("system.ltx"),
    "[wpn_base]\ncost = 100\nammo_class = ammo_a, ammo_b\n\n[wpn_ak74]:wpn_base\ncost = 4000\n",
  )?;
  fs::write(
    root.join("mod_system_patch.ltx"),
    "![wpn_ak74]\ncost = 9999\n>ammo_class = ammo_c\n",
  )?;

  Ok(root)
}

fn open(root: &PathBuf, dialect: Arc<dyn LtxDialect>) -> XrfResult<LtxProject> {
  LtxProject::open_at_path_opt(root, LtxProjectOptions::default().with_dialect(dialect))
}

#[test]
fn standard_mode_refuses_the_patch_file_and_names_the_flag() -> XrfResult {
  let root: PathBuf = patched_tree("standard")?;
  let project: LtxProject = open(&root, Arc::new(LtxStandardDialect))?;

  // A patch file is an ordinary config to standard LTX, so it is an entry point, and resolving it fails.
  assert_eq!(project.ltx_file_entries.len(), 2);

  let error: String = project
    .read_full(&XrayLogicalPath::new("mod_system_patch.ltx")?)
    .expect_err("standard resolution to refuse a patch file")
    .to_string();

  assert!(error.contains("needs the dltx dialect"), "{error}");
  assert!(error.contains("--dltx"), "{error}");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn dltx_mode_treats_the_patch_file_as_an_attachment() -> XrfResult {
  let root: PathBuf = patched_tree("attachment")?;
  let project: LtxProject = open(&root, Arc::new(DltxDialect))?;

  // Not an entry point: it belongs to `system.ltx`, and verifying it alone would report its override as an orphan.
  assert_eq!(
    project.ltx_file_entries,
    vec![XrayLogicalPath::new("system.ltx")?],
    "the patch file is an attachment, not a config of its own"
  );
  assert_eq!(project.ltx_files.len(), 2, "though it is still a file in the project");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn dltx_mode_resolves_the_patched_values() -> XrfResult {
  let root: PathBuf = patched_tree("resolved")?;
  let project: LtxProject = open(&root, Arc::new(DltxDialect))?;
  let resolved: Arc<Ltx> = project.read_full(&XrayLogicalPath::new("system.ltx")?)?;

  assert_eq!(resolved.get_from("wpn_ak74", "cost"), Some("9999"), "the patch wins");
  assert_eq!(
    resolved.get_from("wpn_ak74", "ammo_class"),
    Some("ammo_a,ammo_b,ammo_c"),
    "inherited from the base, then appended to by the patch"
  );
  assert_eq!(
    resolved.get_from("wpn_base", "cost"),
    Some("100"),
    "the base is untouched"
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn the_same_tree_resolves_differently_under_each_dialect() -> XrfResult {
  let root: PathBuf = patched_tree("comparison")?;
  let entry: XrayLogicalPath = XrayLogicalPath::new("system.ltx")?;

  let standard: Arc<Ltx> = open(&root, Arc::new(LtxStandardDialect))?.read_full(&entry)?;
  let dltx: Arc<Ltx> = open(&root, Arc::new(DltxDialect))?.read_full(&entry)?;

  // The whole point of the opt-in: the same files, and a value that differs by which rules were asked for. Standard
  // LTX cannot see the patch file at all, because nothing includes it.
  assert_eq!(standard.get_from("wpn_ak74", "cost"), Some("4000"));
  assert_eq!(dltx.get_from("wpn_ak74", "cost"), Some("9999"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn dltx_mode_reports_a_dialect_name_a_caller_can_show() {
  assert_eq!(DltxDialect.get_name(), "dltx");
  assert_eq!(LtxStandardDialect.get_name(), "ltx");
}
