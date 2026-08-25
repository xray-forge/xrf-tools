use std::fs;

use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;

use crate::staged_write::write_file_staged;

#[test]
fn replaces_an_existing_file_after_staging_succeeds() -> XrfResult {
  let path = write_generated_test_resource("staged_write/replaced.txt", "original")?;

  write_file_staged(&path, b"replacement")?;

  assert_eq!(fs::read(path)?, b"replacement");

  Ok(())
}

#[test]
fn leaves_no_staging_files_behind() -> XrfResult {
  // Its own directory, because the assertion scans one and the sibling test writes here too.
  let path = write_generated_test_resource("staged_write/leftovers/no_leftovers.txt", "original")?;

  write_file_staged(&path, b"replacement")?;

  let leftovers: Vec<String> = fs::read_dir(path.parent().expect("Expected a parent"))?
    .filter_map(Result::ok)
    .map(|entry| entry.file_name().to_string_lossy().into_owned())
    .filter(|name| name.contains("xrf-tmp") || name.contains("xrf-backup"))
    .collect();

  assert!(leftovers.is_empty(), "left behind: {leftovers:?}");

  Ok(())
}

#[test]
fn replaces_a_destination_that_is_open_for_reading() -> XrfResult {
  // What lets the replacement be one rename on every platform. Windows is the reason to pin it: a
  // reader holding the target used to be the case the Windows-only backup dance existed for, and the
  // dance cost a window where the file was absent. If `fs::rename` ever stops replacing here, this
  // fails loudly rather than the save quietly losing its atomicity.
  let path = write_generated_test_resource("staged_write/open_for_reading.txt", "original")?;
  let held: fs::File = fs::File::open(&path)?;

  write_file_staged(&path, b"replacement")?;

  drop(held);

  assert_eq!(fs::read(&path)?, b"replacement");

  Ok(())
}

#[test]
fn keeps_the_original_when_staging_fails() -> XrfResult {
  // A refusal must not consume the file it was going to replace. `write_file_staged` requires the
  // target to exist, so pointing it at a name nothing holds is the cheapest way to reach the error
  // path and prove it left the directory as it found it.
  let path = write_generated_test_resource("staged_write/refused/present.txt", "original")?;
  let directory = path.parent().expect("Expected a parent").to_path_buf();
  let missing = directory.join("absent.txt");

  assert!(write_file_staged(&missing, b"replacement").is_err());
  assert!(!missing.exists(), "a refused write must not create the target");
  assert_eq!(fs::read(&path)?, b"original", "the sibling it never named is untouched");

  let leftovers: Vec<String> = fs::read_dir(&directory)?
    .filter_map(Result::ok)
    .map(|entry| entry.file_name().to_string_lossy().into_owned())
    .filter(|name| name.contains("xrf-tmp"))
    .collect();

  assert!(leftovers.is_empty(), "left behind: {leftovers:?}");

  Ok(())
}
