use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::som::level_som_file::LevelSomFile;
use crate::som::level_som_polygon::SomPolygon;
use crate::tests::fixtures::{chunk, floats, triangle};

/// A sound occlusion mesh: a version chunk and a packed run of triangles.
fn mesh_bytes(version: u32, triangles: &[(u32, f32)]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(LevelSomFile::VERSION_CHUNK_ID, &version.to_le_bytes())?;
  let mut geometry: Vec<u8> = Vec::new();

  for (two_sided, occlusion) in triangles {
    geometry.extend(triangle(&two_sided.to_le_bytes()));
    geometry.extend(floats(&[*occlusion]));
  }

  bytes.extend(chunk(LevelSomFile::GEOMETRY_CHUNK_ID, &geometry)?);

  Ok(bytes)
}

#[test]
fn a_sound_occluder_counts_a_two_sided_face_twice_the_way_the_loader_builds_it() -> XrfResult {
  let bytes: Vec<u8> = mesh_bytes(0, &[(1, 0.5), (0, 0.25)])?;
  let mesh: LevelSomFile = LevelSomFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(mesh.polygons.len(), 2);
  assert_eq!(mesh.get_two_sided_count(), 1);
  assert_eq!(mesh.get_faces_count(), 3);
  assert_eq!(mesh.polygons[0].occlusion, 0.5);

  Ok(())
}

#[test]
fn a_sound_occlusion_mesh_is_written_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = mesh_bytes(0, &[(1, 0.5), (0, 0.25)])?;
  let mesh: LevelSomFile = LevelSomFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;

  let mut writer: ChunkWriter = ChunkWriter::new();

  mesh.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn a_sound_occlusion_version_the_engine_refuses_is_refused_here_too() -> XrfResult {
  assert!(LevelSomFile::read_from_bytes::<XRayByteOrder>(mesh_bytes(1, &[])?).is_err());

  Ok(())
}

#[test]
fn a_sound_occluder_triangle_is_wider_than_a_visual_one_by_its_occlusion() {
  assert_eq!(SomPolygon::SERIALIZED_SIZE, 44);
}
