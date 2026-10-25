//! Assembles a project from a directory and works out its entry points.
//!
//! Paths are **logical**, relative to the project's mount, since the project reads through a VFS whether its configs are
//! loose or archived.

use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::utils::create_generated_test_resource_dir;
use xrf_vfs::{
  XrayInputKind, XrayInputRecord, XrayLogicalPath, XrayLookupScope, XrayMountMode, XrayRoots, XraySearch, XrayVfs,
};

use crate::finding::LtxFindingKind;
use crate::project::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};

fn create_root(name: &str) -> XrfResult<PathBuf> {
  Ok(create_generated_test_resource_dir(&format!("project/{name}"))?)
}

#[test]
fn a_format_hands_over_only_the_rewrites_and_commits_them_by_default() -> XrfResult {
  let root: PathBuf = create_root("format_rewrites")?;

  fs::write(root.join("unformatted.ltx"), "[a]\nkey=value\n")?;
  fs::write(root.join("formatted.ltx"), "[b]\r\nkey = value\r\n")?;

  let project: LtxProject = LtxProject::open_at_path(&root)?;
  let mut handed: Vec<(PathBuf, Vec<u8>)> = Vec::new();
  let result: LtxProjectFormatResult = project.format_all_files_with(LtxFormatOptions::default(), |rewrites| {
    handed = rewrites;

    Ok(())
  })?;

  assert_eq!(
    handed,
    vec![(root.join("unformatted.ltx"), b"[a]\r\nkey = value\r\n".to_vec())]
  );
  assert_eq!((result.total_files, result.invalid_files), (2, 1));
  assert_eq!(
    fs::read_to_string(root.join("unformatted.ltx"))?,
    "[a]\nkey=value\n",
    "handing over writes nothing"
  );

  project.format_all_files()?;

  assert_eq!(
    fs::read_to_string(root.join("unformatted.ltx"))?,
    "[a]\r\nkey = value\r\n"
  );
  assert_eq!(
    fs::read_to_string(root.join("formatted.ltx"))?,
    "[b]\r\nkey = value\r\n"
  );

  fs::remove_dir_all(root)?;

  Ok(())
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
    project.read_resolution(entry)?.ltx.get_from("section", "value"),
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

  let at_game_root: LtxProject = LtxProject::open_at_search_opt(
    &configs,
    &XraySearch::new(vfs).with_step("root", XrayLookupScope::all()),
    Some("configs"),
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
    at_game_root.system_ltx()?.ltx.get_from("section", "value"),
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

  let project: LtxProject = LtxProject::open_at_path(&root)?;

  assert_eq!(project.ltx_files.len(), 2);
  assert!(
    project.ltx_file_entries.contains(&XrayLogicalPath::new("broken.ltx")?),
    "an unreadable config is an entry point, because the verifier has to reach it to say why"
  );

  // And the verifier is what reports it, as one finding among however many the readable files raise.
  let result = project.verify_entries()?;

  assert_eq!(result.findings.len(), 1);
  assert_eq!(result.findings[0].kind, LtxFindingKind::Unreadable);
  assert_eq!(result.findings[0].entry, "broken.ltx", "{:?}", result.findings);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn a_lean_project_lists_nothing_and_still_resolves_system_ltx_with_its_includes_and_inheritance() -> XrfResult {
  let root: PathBuf = create_root("lean")?;
  let configs: PathBuf = root.join("configs");

  fs::create_dir_all(configs.join("zones"))?;
  fs::write(
    configs.join("system.ltx"),
    "#include \"zones\\zones.ltx\"\n[campfire]:zone_base\nidle_light = on\n",
  )?;
  fs::write(
    configs.join("zones").join("zones.ltx"),
    "[zone_base]\nidle_light_range = 8\nidle_light = off\n",
  )?;

  let search: XraySearch = XrayRoots::one(root.clone(), XrayMountMode::Directory)
    .to_search_plan()?
    .open()?;
  let project: LtxProject = LtxProject::open_lean_at_search_opt(&root, &search, Some("configs"), Default::default())?;
  let system = project.system_ltx()?;
  let campfire = system.ltx.section("campfire").expect("the campfire section to resolve");

  assert!(project.ltx_files.is_empty(), "nothing is listed up front");
  assert_eq!(campfire.get("idle_light"), Some("on"));
  assert_eq!(campfire.get("idle_light_range"), Some("8"));

  Ok(())
}

#[test]
fn a_project_over_a_recording_search_records_the_configs_it_resolves() -> XrfResult {
  let root: PathBuf = create_root("recorded")?;
  let configs: PathBuf = root.join("configs");

  fs::create_dir_all(configs.join("zones"))?;
  fs::write(
    configs.join("system.ltx"),
    "#include \"zones\\*.ltx\"\n[campfire]:zone_base\n",
  )?;
  fs::write(
    configs.join("zones").join("zones.ltx"),
    "[zone_base]\nidle_light = off\n",
  )?;
  fs::write(root.join("unrelated.ltx"), "[unrelated]\n")?;

  let record: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = XrayRoots::one(root.clone(), XrayMountMode::Directory)
    .to_search_plan()?
    .open()?
    .with_recording(&record);
  let project: LtxProject = LtxProject::open_lean_at_search_opt(&root, &search, Some("configs"), Default::default())?;

  project.system_ltx()?;

  let mut read: Vec<String> = record
    .list()
    .into_iter()
    .map(|(input, answer)| match input.get_kind() {
      XrayInputKind::File(path) => format!("{} {}", path.as_str(), answer.source.is_some()),
      XrayInputKind::Listing(directory) => {
        format!("listing {}", directory.as_ref().map_or("", XrayLogicalPath::as_str))
      }
    })
    .collect();

  read.sort();

  // The wildcard include is a listing of its directory, and nothing outside `configs` is read.
  assert_eq!(
    read,
    vec![
      "configs\\system.ltx true",
      "configs\\zones\\zones.ltx true",
      "listing configs\\zones",
    ]
  );

  Ok(())
}

#[test]
fn a_project_over_a_search_reads_what_its_search_reaches_within_the_prefix() -> XrfResult {
  let root: PathBuf = create_root("over_search")?;

  fs::create_dir_all(root.join("configs"))?;
  fs::write(root.join("configs").join("system.ltx"), "#include \"game.ltx\"\n")?;
  fs::write(root.join("configs").join("game.ltx"), "[game]\n")?;
  fs::write(root.join("unrelated.ltx"), "[unrelated]\n")?;

  let search: XraySearch = XrayRoots::one(root.clone(), XrayMountMode::Directory)
    .to_search_plan()?
    .open()?;
  let project: LtxProject = LtxProject::open_at_search_opt(&root, &search, Some("configs"), Default::default())?;

  assert_eq!(
    project.ltx_files,
    vec![
      XrayLogicalPath::new("configs\\game.ltx")?,
      XrayLogicalPath::new("configs\\system.ltx")?
    ]
  );
  assert_eq!(
    project.ltx_file_entries,
    vec![XrayLogicalPath::new("configs\\system.ltx")?]
  );
  assert_eq!(
    project.config_path("game.ltx")?,
    XrayLogicalPath::new("configs\\game.ltx")?
  );

  Ok(())
}
