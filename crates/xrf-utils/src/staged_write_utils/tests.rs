use std::fs::{self, File};
use std::io::Result as IoResult;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::staged_write_utils::{write_file_staged, write_new_file_staged};

/// An empty directory of this case's own, since every assertion here is about what a destination holds.
fn get_test_directory(name: &str) -> IoResult<PathBuf> {
  let directory: PathBuf = build_absolute_generated_test_resource_path(&format!("staged_write_utils/{name}"));

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory)?;

  Ok(directory)
}

#[test]
fn creates_a_destination_that_does_not_exist_yet() -> IoResult<()> {
  let path: PathBuf = get_test_directory("creates")?.join("config.json");

  write_file_staged(&path, b"{}")?;

  assert_eq!(fs::read(&path)?, b"{}");

  Ok(())
}

#[test]
fn replaces_an_existing_destination() -> IoResult<()> {
  let path: PathBuf = get_test_directory("replaces")?.join("config.json");

  fs::write(&path, b"{\"previous\":true}")?;
  write_file_staged(&path, b"{\"current\":true}")?;

  assert_eq!(fs::read(&path)?, b"{\"current\":true}");

  Ok(())
}

#[test]
fn leaves_no_staging_files_behind() -> IoResult<()> {
  let directory: PathBuf = get_test_directory("leftovers")?;

  write_file_staged(&directory.join("config.json"), b"{}")?;

  let names: Vec<String> = fs::read_dir(&directory)?
    .filter_map(Result::ok)
    .map(|entry| entry.file_name().to_string_lossy().to_string())
    .collect();

  assert_eq!(names, vec![String::from("config.json")]);

  Ok(())
}

#[test]
fn replaces_a_destination_that_is_open_for_reading() -> IoResult<()> {
  let path: PathBuf = get_test_directory("open")?.join("config.json");

  fs::write(&path, b"original")?;

  // Windows refuses a plain rename onto a file another handle has open for writing; a shared read handle is what a
  // watcher or an editor holds, and publication must not depend on nobody looking.
  let reader: File = File::open(&path)?;

  write_file_staged(&path, b"replacement")?;

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

  assert!(write_file_staged(&unreachable, b"{}").is_err());
  assert!(!unreachable.exists(), "a refused write must not create the target");
  assert_eq!(fs::read(&path)?, b"{\"previous\":true}");

  Ok(())
}

#[test]
fn creates_the_directories_above_a_new_file() -> IoResult<()> {
  let path: PathBuf = get_test_directory("new")?.join("nested").join("config.json");

  write_new_file_staged(&path, b"{}")?;

  assert_eq!(fs::read(&path)?, b"{}");

  Ok(())
}

#[cfg(feature = "staging-faults")]
#[test]
fn keeps_the_previous_file_when_the_staged_write_fails() -> IoResult<()> {
  use crate::staged_write_utils::staging_faults::fail_next_staged_write;

  let directory: PathBuf = get_test_directory("injected")?;
  let path: PathBuf = directory.join("config.json");

  fs::write(&path, b"{\"previous\":true}")?;

  fail_next_staged_write();

  assert!(write_file_staged(&path, b"{\"current\":true}").is_err());
  assert_eq!(
    fs::read(&path)?,
    b"{\"previous\":true}",
    "the failed write published nothing"
  );

  let names: Vec<String> = fs::read_dir(&directory)?
    .filter_map(Result::ok)
    .map(|entry| entry.file_name().to_string_lossy().to_string())
    .collect();

  assert_eq!(
    names,
    vec![String::from("config.json")],
    "the staged file is cleaned up"
  );

  Ok(())
}
