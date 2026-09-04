//! How many times a project reads and parses its configs.

use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::XrayLogicalPath;

use crate::project::ltx_project::LtxProject;
use crate::project::ltx_read_counters::LtxReadCountersSnapshot;

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
fn opening_a_project_scans_every_config_for_includes_and_parses_none_of_them() -> XrfResult {
  let project: LtxProject = open_nested_project("open")?;
  let counters: LtxReadCountersSnapshot = project.get_read_counters();

  // Entry-point discovery needs every config's include list and none of their contents, so assembly is one read and one
  // include-only parse per config in the project.
  assert_eq!(counters.include_scans, 4);
  assert_eq!(counters.reads, 4);
  assert_eq!(counters.parses, 0);
  assert_eq!(counters.resolutions, 0);
  assert!(counters.bytes_read > 0);

  Ok(())
}

#[test]
fn resolving_one_entry_parses_every_config_in_its_include_tree() -> XrfResult {
  let project: LtxProject = open_nested_project("resolve_once")?;
  let entry: XrayLogicalPath = XrayLogicalPath::new("system.ltx")?;

  project.read_full(&entry)?;

  let counters: LtxReadCountersSnapshot = project.get_read_counters();

  // Four configs parsed on top of the four assembly scans: the entry point, both files it includes, and the one nested
  // inside the first.
  assert_eq!(counters.resolutions, 1);
  assert_eq!(counters.parses, 4);
  assert_eq!(counters.reads, 8);
  assert_eq!(counters.include_scans, 4);

  Ok(())
}

#[test]
fn resolving_the_same_entry_twice_reads_and_parses_everything_twice() -> XrfResult {
  let project: LtxProject = open_nested_project("resolve_twice")?;
  let entry: XrayLogicalPath = XrayLogicalPath::new("system.ltx")?;

  project.read_full(&entry)?;
  project.read_full(&entry)?;

  let counters: LtxReadCountersSnapshot = project.get_read_counters();

  // Nothing is retained between resolutions, which is what makes a gamedata sweep resolve `system.ltx` once per check
  // that wants it. Stage 4 of the plan makes this the same as resolving once; until then it is recorded, not accepted.
  assert_eq!(counters.resolutions, 2);
  assert_eq!(counters.parses, 8);
  assert_eq!(counters.reads, 12);

  Ok(())
}

#[test]
fn verifying_the_project_resolves_each_entry_point_again() -> XrfResult {
  let project: LtxProject = open_nested_project("verify")?;

  project.verify_entries()?;

  let counters: LtxReadCountersSnapshot = project.get_read_counters();

  // One entry point, so verification is one more resolution of the whole include tree on top of assembly.
  assert_eq!(counters.resolutions, 1);
  assert_eq!(counters.parses, 4);

  Ok(())
}

#[test]
fn a_read_outside_the_project_is_not_counted() -> XrfResult {
  let project: LtxProject = open_nested_project("outside")?;
  let before: LtxReadCountersSnapshot = project.get_read_counters();

  // The counters describe what the project did, so a caller reaching past it for a plain VFS read stays invisible.
  crate::Ltx::read_from_vfs(project.vfs(), project.scope(), "second.ltx")?;

  assert_eq!(project.get_read_counters(), before);

  Ok(())
}
