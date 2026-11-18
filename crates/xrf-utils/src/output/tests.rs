use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::OutputFiles;

/// An empty directory of this case's own, holding `names`, each written empty.
fn holding(case: &str, names: &[&str]) -> XrfResult<PathBuf> {
  let directory: PathBuf = GeneratedResources::create_dir(&format!("output/{case}"))?;

  for name in names {
    let path: PathBuf = directory.join(name);

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }

    fs::write(path, b"")?;
  }

  Ok(directory)
}

fn names(paths: &[PathBuf], directory: &Path) -> Vec<String> {
  paths
    .iter()
    .map(|path| {
      path
        .strip_prefix(directory)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
    })
    .collect()
}

#[test]
fn lets_a_run_write_where_none_of_its_files_is_there() -> XrfResult {
  let directory: PathBuf = holding("free", &["readme.txt"])?;
  let outputs: OutputFiles = OutputFiles::new(&directory)
    .with_files(["level", "level.geom"])
    .with_family("lmap#");

  outputs.assert_free(false)?;

  assert!(outputs.find_existing()?.is_empty());
  assert!(
    OutputFiles::new(&directory.join("missing"))
      .with_file("level")
      .find_existing()?
      .is_empty(),
    "a missing directory holds nothing"
  );

  Ok(())
}

#[test]
fn refuses_unforced_a_file_there_in_any_case_and_names_it() -> XrfResult {
  let directory: PathBuf = holding("clash", &["Level.Geom", "terrain/Terrain_Lm.dds"])?;
  let outputs: OutputFiles = OutputFiles::new(&directory).with_files(["level.geom", "terrain\\terrain_lm.dds"]);
  let error: String = outputs
    .assert_free(false)
    .expect_err("a file of the run is there")
    .to_string();

  assert!(
    error.contains("writes (Level.Geom, terrain") && error.contains("Terrain_Lm.dds)"),
    "{error}"
  );
  assert_eq!(error.matches("clash").count(), 1, "{error}");
  assert_eq!(
    names(&outputs.find_existing()?, &directory),
    ["Level.Geom", "terrain/Terrain_Lm.dds"]
  );

  outputs.assert_free(true)?;

  Ok(())
}

#[test]
fn owns_the_one_file_or_directory_a_path_names() -> XrfResult {
  let directory: PathBuf = holding("one_path", &["All.spawn", "unpacked/header.bin"])?;

  assert_eq!(
    names(
      &OutputFiles::of_path(&directory.join("all.spawn")).find_existing()?,
      &directory
    ),
    ["All.spawn"]
  );
  assert_eq!(
    names(
      &OutputFiles::of_path(&directory.join("UNPACKED")).find_existing()?,
      &directory
    ),
    ["unpacked"]
  );
  assert!(
    OutputFiles::of_path(&directory.join("test.spawn"))
      .assert_free(false)
      .is_ok()
  );

  Ok(())
}

#[test]
fn counts_a_familys_members_as_the_runs_own() -> XrfResult {
  let directory: PathBuf = holding(
    "family",
    &[
      "LMAP#2_1.dds",
      "lmap#1_1.dds",
      "lmap.txt",
      "Meshes/BRKBL#1.ogf",
      "meshes.txt",
      "Test_Level/spawn.part",
    ],
  )?;
  let outputs: OutputFiles = OutputFiles::new(&directory)
    .with_family("lmap#")
    .with_family("meshes/brkbl#")
    .with_family("test_level/");

  assert_eq!(
    names(&outputs.find_existing()?, &directory),
    [
      "lmap#1_1.dds",
      "LMAP#2_1.dds",
      "Meshes/BRKBL#1.ogf",
      "Test_Level/spawn.part"
    ]
  );
  assert!(outputs.assert_free(false).is_err());

  Ok(())
}

#[test]
fn removes_only_its_own_files_it_did_not_write_this_time() -> XrfResult {
  let directory: PathBuf = holding("stale", &["level", "lmap#1_1.dds", "lmap#2_1.dds", "notes.txt"])?;
  let outputs: OutputFiles = OutputFiles::new(&directory).with_file("level").with_family("lmap#");
  // Written under another case than the file system holds it, as a write over an existing file on Windows keeps.
  let removed: Vec<PathBuf> = outputs.remove_stale(&[directory.join("LEVEL"), directory.join("lmap#1_1.dds")])?;

  assert_eq!(names(&removed, &directory), ["lmap#2_1.dds"]);
  assert!(directory.join("level").is_file());
  assert!(directory.join("notes.txt").is_file());

  Ok(())
}

/// A written path spelled from another base cannot be told apart from a stale file, so nothing is removed.
#[test]
fn refuses_to_remove_against_a_written_path_outside_its_directory() -> XrfResult {
  let directory: PathBuf = holding("elsewhere", &["level", "lmap#1_1.dds"])?;
  let outputs: OutputFiles = OutputFiles::new(&directory).with_family("lmap#");

  assert!(outputs.remove_stale(&[PathBuf::from("lmap#1_1.dds")]).is_err());
  assert!(directory.join("lmap#1_1.dds").is_file());

  Ok(())
}
