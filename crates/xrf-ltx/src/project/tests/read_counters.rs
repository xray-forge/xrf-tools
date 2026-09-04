//! How many times a project reads and parses its configs.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::XrayLogicalPath;

use crate::ltx::Ltx;
use crate::project::{LtxProject, LtxReadCountersSnapshot};

/// Writes a four-file tree: an entry point including two files, one of which includes a third.
fn open_nested_project(name: &str) -> XrfResult<LtxProject> {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("ltx_read_counters/{name}"));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  fs::write(
    root.join("system.ltx"),
    "#include \"first.ltx\"\n#include \"second.ltx\"\n[entry]\nvalue = 1\n",
  )?;
  fs::write(root.join("first.ltx"), "#include \"nested.ltx\"\n[first]\na = 1\n")?;
  fs::write(root.join("second.ltx"), "[second]\nb = 2\n")?;
  fs::write(root.join("nested.ltx"), "[nested]\nc = 3\n")?;

  LtxProject::open_at_path(&root)
}

#[test]
fn opening_a_project_reads_and_parses_every_config_exactly_once() -> XrfResult {
  let project: LtxProject = open_nested_project("open")?;
  let counters: LtxReadCountersSnapshot = project.get_read_counters();

  // Entry-point discovery needs every config's include list, which is now a read of the same document a resolution
  // will want rather than a separate cheaper scan of the same bytes.
  assert_eq!(counters.reads, 4);
  assert_eq!(counters.parses, 4);
  assert_eq!(counters.include_scans, 4);
  assert_eq!(counters.resolutions, 0);
  assert!(counters.bytes_read > 0);

  Ok(())
}

#[test]
fn resolving_an_entry_reuses_the_configs_assembly_already_read() -> XrfResult {
  let project: LtxProject = open_nested_project("resolve_once")?;
  let entry: XrayLogicalPath = XrayLogicalPath::new("system.ltx")?;

  project.read_full(&entry)?;

  let counters: LtxReadCountersSnapshot = project.get_read_counters();

  // Nothing was read or parsed a second time: the whole include tree was already held from assembly.
  assert_eq!(counters.reads, 4);
  assert_eq!(counters.parses, 4);
  assert_eq!(counters.resolutions, 1);

  Ok(())
}

#[test]
fn resolving_the_same_entry_again_answers_the_first_resolution() -> XrfResult {
  let project: LtxProject = open_nested_project("resolve_twice")?;
  let entry: XrayLogicalPath = XrayLogicalPath::new("system.ltx")?;

  let first: Arc<Ltx> = project.read_full(&entry)?;
  let second: Arc<Ltx> = project.read_full(&entry)?;

  let counters: LtxReadCountersSnapshot = project.get_read_counters();

  assert_eq!(
    counters.resolutions, 1,
    "one resolution however many times it is asked for"
  );
  assert_eq!(counters.reads, 4);
  assert_eq!(counters.parses, 4);

  // The same resolution, not an equal copy of it, which is what makes repeated asking free rather than cheap.
  assert!(Arc::ptr_eq(&first, &second));

  Ok(())
}

#[test]
fn every_caller_that_wants_the_root_config_shares_one_resolution() -> XrfResult {
  let project: LtxProject = open_nested_project("shared_root")?;

  let first: Arc<Ltx> = project.system_ltx()?;
  let second: Arc<Ltx> = project.system_ltx()?;
  let third: Arc<Ltx> = project.read_full(&project.system_ltx_path()?)?;

  // What a gamedata sweep depends on: four checks each ask for `system.ltx` and one resolution serves them all.
  assert_eq!(project.get_read_counters().resolutions, 1);
  assert!(Arc::ptr_eq(&first, &second));
  assert!(Arc::ptr_eq(&first, &third));

  Ok(())
}

#[test]
fn verifying_the_project_adds_one_resolution_per_entry_point() -> XrfResult {
  let project: LtxProject = open_nested_project("verify")?;

  project.verify_entries()?;

  let counters: LtxReadCountersSnapshot = project.get_read_counters();

  assert_eq!(counters.resolutions, 1, "one entry point");
  assert_eq!(counters.reads, 4, "nothing re-read for it");
  assert_eq!(counters.parses, 4);

  Ok(())
}

#[test]
fn verifying_twice_repeats_no_work_at_all() -> XrfResult {
  let project: LtxProject = open_nested_project("verify_twice")?;

  project.verify_entries()?;
  project.verify_entries()?;

  let counters: LtxReadCountersSnapshot = project.get_read_counters();

  // Two whole-project passes, which is what the gamedata `ltx` check does, and the second one reads nothing.
  assert_eq!(counters.resolutions, 1);
  assert_eq!(counters.reads, 4);
  assert_eq!(counters.parses, 4);

  Ok(())
}

#[test]
fn a_read_outside_the_project_is_not_counted() -> XrfResult {
  let project: LtxProject = open_nested_project("outside")?;
  let before: LtxReadCountersSnapshot = project.get_read_counters();

  // The counters describe what the project did, so a caller reaching past it for a plain VFS read stays invisible.
  Ltx::read_from_vfs(project.vfs(), project.scope(), "second.ltx")?;

  assert_eq!(project.get_read_counters(), before);

  Ok(())
}
