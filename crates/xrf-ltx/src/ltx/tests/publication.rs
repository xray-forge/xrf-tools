use std::fs;
use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_test_utils::{GeneratedResources, RefusedPublication};
use xrf_utils::StagingFaults;

use crate::Ltx;

/// A write that fails part way leaves the config a caller had, and nothing staged beside it.
#[test]
fn keeps_the_previous_file_when_publication_fails() -> XrfResult {
  let path: PathBuf = GeneratedResources::create_dir("ltx_write/publication")?.join("system.ltx");

  fs::write(&path, b"previous")?;
  StagingFaults::fail_next_write();

  assert!(Ltx::new().write_to_path(&path).is_err());
  RefusedPublication::assert_left_previous(&path, b"previous");

  Ok(())
}
