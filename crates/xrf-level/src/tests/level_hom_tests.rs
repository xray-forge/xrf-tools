use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::hom::level_hom_file::LevelHomFile;
use crate::hom::level_hom_polygon::HomPolygon;
use crate::tests::fixtures::{chunk, triangle};

/// An occlusion mesh: a version chunk and a packed run of triangles.
fn mesh_bytes(flags: &[u32]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(LevelHomFile::VERSION_CHUNK_ID, &0u32.to_le_bytes())?;
  let mut geometry: Vec<u8> = Vec::new();

  for value in flags {
    geometry.extend(triangle(&value.to_le_bytes()));
  }

  bytes.extend(chunk(LevelHomFile::GEOMETRY_CHUNK_ID, &geometry)?);

  Ok(bytes)
}

#[test]
fn an_occlusion_mesh_reads_as_the_version_and_triangles_it_is() -> XrfResult {
  let bytes: Vec<u8> = mesh_bytes(&[7, 0])?;
  let mesh: LevelHomFile = LevelHomFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(mesh.version, LevelHomFile::CURRENT_VERSION);
  assert_eq!(mesh.polygons.len(), 2);
  assert_eq!(mesh.polygons[0].flags, 7);

  Ok(())
}

#[test]
fn an_occlusion_mesh_is_written_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = mesh_bytes(&[7, 0])?;
  let mesh: LevelHomFile = LevelHomFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  mesh.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn geometry_that_is_not_whole_triangles_is_refused() -> XrfResult {
  let mut bytes: Vec<u8> = chunk(LevelHomFile::VERSION_CHUNK_ID, &0u32.to_le_bytes())?;
  let mut geometry: Vec<u8> = triangle(&0u32.to_le_bytes());

  geometry.push(0);
  bytes.extend(chunk(LevelHomFile::GEOMETRY_CHUNK_ID, &geometry)?);

  let error: String = LevelHomFile::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("a partial triangle is refused")
    .to_string();

  assert!(error.contains("whole triangles"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn an_occluder_triangle_is_the_width_the_engine_packs_it_at() {
  assert_eq!(HomPolygon::SERIALIZED_SIZE, 40);
}
