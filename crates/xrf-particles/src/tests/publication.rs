use std::fs;
use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_test_utils::utils::{assert_publication_left_previous, create_generated_test_resource_dir};
use xrf_utils::staging_faults::fail_next_staged_write;

use crate::chunks::particles_effects_chunk::ParticlesEffectsChunk;
use crate::chunks::particles_groups_chunk::ParticlesGroupsChunk;
use crate::chunks::particles_header_chunk::ParticlesHeaderChunk;
use crate::particles_file::ParticlesFile;

/// A write that fails part way leaves the library a caller had, and nothing staged beside it.
#[test]
fn keeps_the_previous_file_when_publication_fails() -> XrfResult {
  let path: PathBuf = create_generated_test_resource_dir("particles_file/publication")?.join("particles.xr");
  let library: ParticlesFile = ParticlesFile {
    header: ParticlesHeaderChunk { version: 1 },
    effects: ParticlesEffectsChunk { effects: Vec::new() },
    groups: ParticlesGroupsChunk { groups: Vec::new() },
  };

  fs::write(&path, b"previous")?;
  fail_next_staged_write();

  assert!(library.write_to_path::<XRayByteOrder, _>(&path).is_err());
  assert_publication_left_previous(&path, b"previous");

  Ok(())
}
