use std::fs::{self, File};
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use xrf_test_utils::GeneratedResources;

use crate::StagedFile;

/// An empty directory of this case's own, since every assertion here is about what a destination holds.
fn get_test_directory(name: &str) -> IoResult<PathBuf> {
  GeneratedResources::create_dir(&format!("staging/{name}"))
}

/// The names a directory holds, for asserting nothing staged was left beside a target.
fn list_names(directory: &Path) -> IoResult<Vec<String>> {
  Ok(
    fs::read_dir(directory)?
      .filter_map(Result::ok)
      .map(|entry| entry.file_name().to_string_lossy().to_string())
      .collect(),
  )
}

#[test]
fn creates_a_destination_that_does_not_exist_yet() -> IoResult<()> {
  let path: PathBuf = get_test_directory("creates")?.join("config.json");

  StagedFile::write(&path, b"{}")?;

  assert_eq!(fs::read(&path)?, b"{}");

  Ok(())
}

#[test]
fn replaces_an_existing_destination() -> IoResult<()> {
  let path: PathBuf = get_test_directory("replaces")?.join("config.json");

  fs::write(&path, b"{\"previous\":true}")?;
  StagedFile::write(&path, b"{\"current\":true}")?;

  assert_eq!(fs::read(&path)?, b"{\"current\":true}");

  Ok(())
}

#[test]
fn leaves_no_staging_files_behind() -> IoResult<()> {
  let directory: PathBuf = get_test_directory("leftovers")?;

  StagedFile::write(&directory.join("config.json"), b"{}")?;

  assert_eq!(list_names(&directory)?, vec![String::from("config.json")]);

  Ok(())
}

#[test]
fn replaces_a_destination_that_is_open_for_reading() -> IoResult<()> {
  let path: PathBuf = get_test_directory("open")?.join("config.json");

  fs::write(&path, b"original")?;

  // Windows refuses a plain rename onto a file another handle has open for writing; a shared read handle is what a
  // watcher or an editor holds, and publication must not depend on nobody looking.
  let reader: File = File::open(&path)?;

  StagedFile::write(&path, b"replacement")?;

  drop(reader);

  assert_eq!(fs::read(&path)?, b"replacement");

  Ok(())
}

#[test]
fn keeps_the_previous_file_when_staging_is_refused() -> IoResult<()> {
  let directory: PathBuf = get_test_directory("refused")?;
  let path: PathBuf = directory.join("config.json");

  fs::write(&path, b"{\"previous\":true}")?;

  // A destination inside a directory that does not exist cannot be staged, and must not be created either.
  let unreachable: PathBuf = directory.join("missing").join("config.json");

  assert!(StagedFile::write(&unreachable, b"{}").is_err());
  assert!(!unreachable.exists(), "a refused write must not create the target");
  assert_eq!(fs::read(&path)?, b"{\"previous\":true}");

  Ok(())
}

#[test]
fn creates_the_directories_above_a_new_file() -> IoResult<()> {
  let path: PathBuf = get_test_directory("new")?.join("nested").join("config.json");

  StagedFile::write_creating_directories(&path, b"{}")?;

  assert_eq!(fs::read(&path)?, b"{}");

  Ok(())
}

#[test]
fn names_a_sibling_after_its_target_and_knows_it_again() -> IoResult<()> {
  let target: PathBuf = get_test_directory("siblings")?.join("config.json");
  let sibling: PathBuf = StagedFile::to_sibling(&target, "staged")?;
  let name = sibling.file_name().expect("a sibling has a name");

  assert_eq!(sibling.parent(), target.parent());
  assert!(name.to_string_lossy().starts_with(".config.json.xrf-staged-"));
  assert!(StagedFile::is_sibling(name));
  assert!(!StagedFile::is_sibling(
    target.file_name().expect("a target has a name")
  ));
  assert_ne!(
    sibling,
    StagedFile::to_sibling(&target, "staged")?,
    "two siblings never share a name"
  );

  Ok(())
}

#[cfg(feature = "staging-faults")]
#[test]
fn keeps_the_previous_file_when_the_staged_write_fails() -> IoResult<()> {
  use crate::StagingFaults;

  let directory: PathBuf = get_test_directory("injected")?;
  let path: PathBuf = directory.join("config.json");

  fs::write(&path, b"{\"previous\":true}")?;

  StagingFaults::fail_next_write();

  assert!(StagedFile::write(&path, b"{\"current\":true}").is_err());
  assert_eq!(
    fs::read(&path)?,
    b"{\"previous\":true}",
    "the failed write published nothing"
  );
  assert_eq!(
    list_names(&directory)?,
    vec![String::from("config.json")],
    "the staged file is cleaned up"
  );

  Ok(())
}
