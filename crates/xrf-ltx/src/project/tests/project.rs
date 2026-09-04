//! Assembles a project from a directory and works out its entry points.
//!
//! Paths are **logical**, relative to the project's mount, since the project reads through a VFS whether its configs are
//! loose or archived.

use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayLogicalPath, XrayLookupScope, XrayVfs};

use crate::project::ltx_project::LtxProject;

fn create_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("project/{name}"));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  Ok(root)
}

#[test]
fn does_not_treat_wildcard_included_files_as_entries() -> XrfResult {
  let root: PathBuf = create_root("wildcard")?;
  let sections: PathBuf = root.join("sections");

  fs::create_dir_all(&sections)?;
  fs::write(root.join("root.ltx"), "#include \"sections\\section_*.ltx\"\n")?;
  fs::write(sections.join("section_first.ltx"), "[first]\n")?;
  fs::write(sections.join("section_second.ltx"), "[second]\n")?;

  let project: LtxProject = LtxProject::open_at_path(&root)?;

  assert_eq!(project.ltx_file_entries, vec![XrayLogicalPath::new("root.ltx")?]);
  assert_eq!(project.ltx_files.len(), 3, "every config is still listed");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn does_not_treat_a_named_included_file_as_an_entry() -> XrfResult {
  let root: PathBuf = create_root("named")?;

  fs::write(root.join("root.ltx"), "#include \"included.ltx\"\n")?;
  fs::write(root.join("included.ltx"), "[section]\n")?;

  let project: LtxProject = LtxProject::open_at_path(&root)?;

  assert_eq!(project.ltx_file_entries, vec![XrayLogicalPath::new("root.ltx")?]);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn does_not_treat_a_file_included_after_a_section_as_an_entry() -> XrfResult {
  // Anomaly places includes after sections, and the file they name inherits from the includer. Treated as an entry it would
  // be read standalone and reported as inheriting an unknown section.
  let root: PathBuf = create_root("trailing")?;

  fs::write(root.join("root.ltx"), "[base]\nvalue = 1\n\n#include \"child.ltx\"\n")?;
  fs::write(root.join("child.ltx"), "[child]:base\n")?;

  let project: LtxProject = LtxProject::open_at_path(&root)?;

  assert_eq!(project.ltx_file_entries, vec![XrayLogicalPath::new("root.ltx")?]);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn renders_a_loose_path_for_a_person_and_reads_it_through_the_project() -> XrfResult {
  // What keeps reports reading as they did before the project moved onto a VFS: a logical path is stored, an absolute one is
  // shown, and only the project knows how to open it.
  let root: PathBuf = create_root("paths")?;

  fs::write(root.join("root.ltx"), "[section]\nvalue = 1\n")?;

  let project: LtxProject = LtxProject::open_at_path(&root)?;
  let entry: &XrayLogicalPath = project.ltx_file_entries.first().expect("one entry");

  assert_eq!(entry, &XrayLogicalPath::new("root.ltx")?);
  assert_eq!(project.path_of(entry), root.join("root.ltx"));
  assert_eq!(project.physical_path_of(entry), Some(root.join("root.ltx")));
  assert_eq!(
    project.read_full(entry)?.get_from("section", "value"),
    Some("1"),
    "the project reads its own files"
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn places_config_names_in_whichever_scope_the_project_has() -> XrfResult {
  // The same check body reads a configs directory and a game root, so a config-relative name has to carry the scope's
  // prefix. A bare `system.ltx` resolves in the first project and nothing at all in the second.
  let root: PathBuf = create_root("scoped")?;
  let configs: PathBuf = root.join("configs");

  fs::create_dir_all(&configs)?;
  fs::write(configs.join("system.ltx"), "[section]\nvalue = 1\n")?;

  let at_configs: LtxProject = LtxProject::open_at_path(&configs)?;

  assert_eq!(at_configs.system_ltx_path()?, XrayLogicalPath::new("system.ltx")?);
  assert_eq!(
    at_configs.config_path("environment\\suns.ltx")?,
    XrayLogicalPath::new("environment\\suns.ltx")?
  );

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root)?;

  let at_game_root: LtxProject = LtxProject::open_at_scope_opt(
    &configs,
    vfs,
    XrayLookupScope::all().with_prefix("configs")?,
    Default::default(),
  )?;

  assert_eq!(
    at_game_root.system_ltx_path()?,
    XrayLogicalPath::new("configs\\system.ltx")?
  );
  assert_eq!(
    at_game_root.config_path("environment\\suns.ltx")?,
    XrayLogicalPath::new("configs\\environment\\suns.ltx")?
  );
  assert_eq!(
    at_game_root.system_ltx()?.get_from("section", "value"),
    Some("1"),
    "the scoped path is the one that reads"
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn an_empty_project_holds_nothing_and_still_answers() -> XrfResult {
  let project: LtxProject = LtxProject::empty(PathBuf::from("C:\\gamedata\\configs"));

  assert!(project.ltx_files.is_empty());
  assert!(project.ltx_file_entries.is_empty());
  assert!(
    project.physical_path_of(&XrayLogicalPath::new("system.ltx")?).is_none(),
    "nothing is mounted, so nothing resolves"
  );

  Ok(())
}

#[test]
fn an_unparseable_config_is_reported_per_entry_rather_than_ending_assembly() -> XrfResult {
  let root: PathBuf = create_root("unparseable")?;

  fs::write(
    root.join("readable.ltx"),
    "[section]
value = 1
",
  )?;
  fs::write(
    root.join("broken.ltx"),
    "[unterminated
",
  )?;

  // Assembly cannot read one of them, and answers anyway. One unreadable config used to end the whole open, which hid
  // every other file's findings behind it; see `issues/0116`.
  let project: LtxProject = LtxProject::open_at_path(&root)?;

  assert_eq!(project.ltx_files.len(), 2);
  assert!(
    project.ltx_file_entries.contains(&XrayLogicalPath::new("broken.ltx")?),
    "an unreadable config is an entry point, because the verifier has to reach it to say why"
  );

  // And the verifier is what reports it, as one finding among however many the readable files raise.
  let result = project.verify_entries()?;

  assert_eq!(result.errors.len(), 1);
  assert!(
    result.errors[0].to_string().contains("broken.ltx"),
    "{:?}",
    result.errors
  );

  fs::remove_dir_all(root)?;

  Ok(())
}
