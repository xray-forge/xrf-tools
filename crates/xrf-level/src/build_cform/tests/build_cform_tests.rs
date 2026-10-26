use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::build_cform::build_cform_face::BuildCformFace;
use crate::build_cform::build_cform_file::BuildCformFile;
use crate::build_cform::build_cform_raycast_face::BuildCformRaycastFace;
use crate::tests::fixtures::{chunk, floats};

/// The geometry chunk's payload: a header counting one triangle over three vertices, then both.
fn geometry_payload(version: u32, corners: [u32; 3]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  for value in [version, 3, 1] {
    bytes.extend_from_slice(&value.to_le_bytes());
  }

  bytes.extend(floats(&[-1.0, -1.0, -1.0, 1.0, 1.0, 1.0]));
  bytes.extend(floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]));

  for corner in corners {
    bytes.extend_from_slice(&corner.to_le_bytes());
  }

  // The address the 32-bit compiler held its face at.
  bytes.extend_from_slice(&0x060A_D030u32.to_le_bytes());

  bytes
}

/// One raycast record, with the padding `#pragma pack(4)` puts around its fields.
fn raycast_payload() -> Vec<u8> {
  let mut bytes: Vec<u8> = 2u16.to_le_bytes().to_vec();

  bytes.extend_from_slice(&[0xAA, 0xBB]);
  bytes.extend_from_slice(&5u32.to_le_bytes());
  bytes.extend(floats(&[0.25, 0.5, 0.75, 0.5, 0.25, 1.0]));
  bytes.extend_from_slice(&0u16.to_le_bytes());
  bytes.extend_from_slice(&[0, 0]);

  bytes
}

fn form_bytes(version: u32, corners: [u32; 3]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(BuildCformFile::GEOMETRY_CHUNK_ID, &geometry_payload(version, corners))?;

  bytes.extend(chunk(BuildCformFile::RAYCAST_FACES_CHUNK_ID, &raycast_payload())?);

  Ok(bytes)
}

#[test]
fn a_raycast_record_is_the_width_the_compiler_blits_it_at() {
  assert_eq!(BuildCformRaycastFace::SERIALIZED_SIZE, 36);
  assert_eq!(raycast_payload().len() as u64, BuildCformRaycastFace::SERIALIZED_SIZE);
}

#[test]
fn a_build_cform_reads_its_geometry_and_one_raycast_record_per_face() -> XrfResult {
  let file: BuildCformFile = BuildCformFile::read_from_bytes::<XRayByteOrder>(form_bytes(4, [0, 1, 2])?)?;

  assert_eq!(file.version, BuildCformFile::CURRENT_VERSION);
  assert_eq!(file.aabb_max.y, 1.0);
  assert_eq!(file.vertices.len(), 3);
  assert_eq!(
    file.faces,
    vec![BuildCformFace {
      vertices: [0, 1, 2],
      face_handle: 0x060A_D030,
    }]
  );
  assert_eq!(file.raycast_faces.len(), 1);
  assert_eq!(file.raycast_faces[0].material, 2);
  assert_eq!(file.raycast_faces[0].material_padding, 0xBBAA);
  assert_eq!(file.raycast_faces[0].game_material, 5);
  assert_eq!(file.raycast_faces[0].texture_coordinates[1], (0.75, 0.5));

  Ok(())
}

#[test]
fn a_build_cform_is_written_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = form_bytes(4, [2, 0, 1])?;
  let file: BuildCformFile = BuildCformFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn a_build_cform_of_another_version_is_refused() -> XrfResult {
  // xrAI and the detail light compiler both assert `CFORM_CURRENT_VERSION`.
  let error: String = BuildCformFile::read_from_bytes::<XRayByteOrder>(form_bytes(3, [0, 1, 2])?)
    .expect_err("expect the version to be refused")
    .to_string();

  assert!(error.contains("version 3"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn a_build_cform_naming_a_missing_vertex_is_refused() -> XrfResult {
  let error: String = BuildCformFile::read_from_bytes::<XRayByteOrder>(form_bytes(4, [0, 1, 3])?)
    .expect_err("expect the corner past the vertices to be refused")
    .to_string();

  assert!(error.contains("names vertex"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn a_build_cform_whose_raycast_records_miss_a_face_is_refused() -> XrfResult {
  let mut bytes: Vec<u8> = chunk(BuildCformFile::GEOMETRY_CHUNK_ID, &geometry_payload(4, [0, 1, 2]))?;
  let mut records: Vec<u8> = raycast_payload();

  records.extend(raycast_payload());
  bytes.extend(chunk(BuildCformFile::RAYCAST_FACES_CHUNK_ID, &records)?);

  let error: String = BuildCformFile::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("expect the extra record to be refused")
    .to_string();

  assert!(
    error.contains("2 raycast records for 1 faces"),
    "Unexpected error: {error}"
  );

  Ok(())
}
