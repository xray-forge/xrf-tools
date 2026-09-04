//! That an include resolves against the directory of the file that wrote it, whatever the process working directory is.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::dialect::LtxStandardDialect;
use crate::ltx::Ltx;

#[test]
fn a_directory_is_taken_from_either_separator() {
  assert_eq!(Ltx::directory_of("configs\\system.ltx"), "configs");
  assert_eq!(Ltx::directory_of("configs/system.ltx"), "configs");
  assert_eq!(Ltx::directory_of("C:/x/y/system.ltx"), "C:/x/y");
  assert_eq!(Ltx::directory_of("C:\\x\\y\\system.ltx"), "C:\\x\\y");
  assert_eq!(Ltx::directory_of("/x/y/system.ltx"), "/x/y");

  // Mixed separators are what a Windows caller passing a forward-slashed root actually produces.
  assert_eq!(Ltx::directory_of("C:\\x/y\\system.ltx"), "C:\\x/y");
  assert_eq!(Ltx::directory_of("C:/x\\y/system.ltx"), "C:/x\\y");

  // A bare name is already relative to wherever it was found, so it has no directory of its own.
  assert_eq!(Ltx::directory_of("system.ltx"), "");
}

/// Writes a root that only includes `misc/nested.ltx`, and the nested file that declares the section.
fn write_including_tree(root: &Path) -> XrfResult<PathBuf> {
  fs::create_dir_all(root.join("misc"))?;
  fs::write(root.join("misc").join("nested.ltx"), "[nested]\nkey = value\n")?;

  let entry: PathBuf = root.join("system.ltx");

  fs::write(&entry, "#include \"misc\\nested.ltx\"\n")?;

  Ok(entry)
}

#[test]
fn a_filesystem_root_finds_what_it_includes_from_anywhere() -> XrfResult {
  let root: PathBuf = build_absolute_generated_test_resource_path("include_directories/from_anywhere");
  let entry: PathBuf = write_including_tree(&root)?;

  // Forward-slashed, which is how the CLI hands a path over, and never the working directory of this test process.
  let resolved: Ltx =
    Ltx::read_from_file_with_dialect(entry.to_string_lossy().replace('\\', "/"), &LtxStandardDialect)?;

  assert_eq!(
    resolved.get_from("nested", "key"),
    Some("value"),
    "an included section to survive a read by filesystem path"
  );

  fs::remove_dir_all(&root)?;

  Ok(())
}

#[test]
fn a_wildcard_include_resolves_in_the_same_directory() -> XrfResult {
  let root: PathBuf = build_absolute_generated_test_resource_path("include_directories/wildcard");

  fs::create_dir_all(root.join("items"))?;
  fs::write(root.join("items").join("w_first.ltx"), "[first]\na = 1\n")?;
  fs::write(root.join("items").join("w_second.ltx"), "[second]\nb = 2\n")?;

  let entry: PathBuf = root.join("system.ltx");

  fs::write(&entry, "#include \"items\\w_*.ltx\"\n")?;

  let resolved: Ltx =
    Ltx::read_from_file_with_dialect(entry.to_string_lossy().replace('\\', "/"), &LtxStandardDialect)?;

  assert_eq!(resolved.get_from("first", "a"), Some("1"));
  assert_eq!(resolved.get_from("second", "b"), Some("2"));

  fs::remove_dir_all(&root)?;

  Ok(())
}

#[test]
fn an_include_two_directories_deep_resolves_against_its_own_parent() -> XrfResult {
  let root: PathBuf = build_absolute_generated_test_resource_path("include_directories/nested");

  fs::create_dir_all(root.join("a").join("b"))?;
  fs::write(root.join("a").join("b").join("leaf.ltx"), "[leaf]\nkey = deep\n")?;
  // The middle file includes its own sibling, so it is the middle file's directory that has to be used, not the root's.
  fs::write(root.join("a").join("middle.ltx"), "#include \"b\\leaf.ltx\"\n")?;

  let entry: PathBuf = root.join("system.ltx");

  fs::write(&entry, "#include \"a\\middle.ltx\"\n")?;

  let resolved: Ltx =
    Ltx::read_from_file_with_dialect(entry.to_string_lossy().replace('\\', "/"), &LtxStandardDialect)?;

  assert_eq!(resolved.get_from("leaf", "key"), Some("deep"));

  fs::remove_dir_all(&root)?;

  Ok(())
}

#[test]
fn a_config_read_by_path_records_its_real_directory() -> XrfResult {
  let root: PathBuf = build_absolute_generated_test_resource_path("include_directories/recorded");
  let entry: PathBuf = write_including_tree(&root)?;
  let forward: String = entry.to_string_lossy().replace('\\', "/");

  let resolved: Ltx = Ltx::read_from_file_with_dialect(&forward, &LtxStandardDialect)?;

  // `pack-equipment` resolves the `~` gamedata marker by joining `..` onto this, so an empty directory sent it to the
  // wrong root rather than failing.
  assert_eq!(
    resolved
      .get_directory()
      .map(|it| it.to_string_lossy().replace('\\', "/")),
    Some(Ltx::directory_of(&forward).replace('\\', "/")),
    "the recorded directory to be the file's own parent"
  );
  assert_ne!(resolved.get_directory().map(PathBuf::as_path), Some(Path::new("")));

  fs::remove_dir_all(&root)?;

  Ok(())
}
