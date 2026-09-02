//! Visuals in the shapes `ogf fix` meets, laid out the way `xrf-db`'s own residue fixtures are.
//!
//! Well-formed scaffolding comes from [`ChunkWriter`]; every defect is raw bytes, so the fixture cannot inherit a
//! mistake from the writer whose output the command is judged against.

use std::fs;
use std::path::PathBuf;

use xrf_chunk::ChunkWriter;
use xrf_db::{OgfBox, OgfHeaderChunk, OgfKinematicsChunk, OgfSphere, Vector3d, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

/// The four references shipped Anomaly zombied faces declare.
pub(super) const DECLARED_REFS: [&str; 4] = [
  "actors\\stalker_animation",
  "actors\\stalker_scripts_animation",
  "actors\\zombied_animation",
  "actors\\stalker_smart_cover_animation",
];

/// The fifth reference those faces carry without counting it, which the engine therefore never loads.
pub(super) const SPLIT_REF: &str = "actors\\stalker_scenario_animation";

/// Where the declared chunk size cuts that fifth reference, leaving `actors\st` inside the chunk.
const SPLIT_AT: usize = 9;

/// Bytes both fragments of the split reference add up to, terminator included.
pub(super) const SPLIT_SIZE: usize = 34;

fn raw_chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(8 + payload.len());

  bytes.extend_from_slice(&id.to_le_bytes());
  bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
  bytes.extend_from_slice(payload);

  bytes
}

fn header_bytes() -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_xr::<XRayByteOrder, _>(&OgfHeaderChunk {
    version: 4,
    model_type: 3,
    shader_id: 0,
    bounding_box: OgfBox {
      min: Vector3d::new(0.0, 0.0, 0.0),
      max: Vector3d::new(1.0, 1.0, 1.0),
    },
    bounding_sphere: OgfSphere {
      position: Vector3d::new(0.0, 0.0, 0.0),
      radius: 1.0,
    },
  })?;

  writer.flush_chunk_into_buffer::<XRayByteOrder>(OgfHeaderChunk::CHUNK_ID)
}

/// Count, then that many NUL-terminated paths.
fn kinematics_payload(refs: &[&str]) -> Vec<u8> {
  let mut payload: Vec<u8> = (refs.len() as u32).to_le_bytes().to_vec();

  for reference in refs {
    payload.extend_from_slice(reference.as_bytes());
    payload.push(0);
  }

  payload
}

pub(super) fn well_formed() -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = header_bytes()?;

  bytes.extend(raw_chunk(
    OgfKinematicsChunk::CHUNK_ID,
    &kinematics_payload(&DECLARED_REFS),
  ));

  Ok(bytes)
}

/// The Anomaly zombied face shape: a fifth reference split across the declared bounds of chunk 24.
pub(super) fn split_motion_ref() -> XrfResult<Vec<u8>> {
  let mut full: Vec<u8> = SPLIT_REF.as_bytes().to_vec();

  full.push(0);

  let mut payload: Vec<u8> = kinematics_payload(&DECLARED_REFS);

  payload.extend_from_slice(&full[..SPLIT_AT]);

  let mut bytes: Vec<u8> = header_bytes()?;

  bytes.extend(raw_chunk(OgfKinematicsChunk::CHUNK_ID, &payload));
  bytes.extend_from_slice(&full[SPLIT_AT..]);

  Ok(bytes)
}

/// Trailing bytes long enough to be a header and belonging to nothing, which the reader refuses.
pub(super) fn unexplained_residue() -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = well_formed()?;

  bytes.extend_from_slice(b"alker_scenario_animation\0");

  Ok(bytes)
}

/// An empty directory of this test's own.
pub(super) fn create_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("ogf-fix/{name}"));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  Ok(root)
}
