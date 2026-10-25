use std::fs;
use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_test_utils::{GeneratedResources, RefusedPublication};
use xrf_utils::StagingFaults;

use crate::thm_file::ThmFile;

/// A write that fails part way leaves the descriptor a caller had, and nothing staged beside it.
#[test]
fn keeps_the_previous_file_when_publication_fails() -> XrfResult {
  let path: PathBuf = GeneratedResources::create_dir("thm_file/publication")?.join("texture.thm");

  fs::write(&path, b"previous")?;
  StagingFaults::fail_next_write();

  assert!(ThmFile::default().write_to_path::<XRayByteOrder, _>(&path).is_err());
  RefusedPublication::assert_left_previous(&path, b"previous");

  Ok(())
}
