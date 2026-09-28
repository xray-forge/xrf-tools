use std::io::Write;

use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_test_utils::utils::{
  build_relative_test_sample_file_path, open_generated_test_resource_as_file, overwrite_generated_test_resource_as_file,
};

use crate::cform::level_cform_face::LevelCformFace;
use crate::cform::level_cform_file::{LevelCformFile, LevelCformHeader};

fn sample() -> LevelCformHeader {
  LevelCformHeader {
    version: 4,
    vertex_count: 420_690,
    face_count: 810_223,
    aabb_min: Vector3d::new(-600.0, -20.5, -615.0),
    aabb_max: Vector3d::new(600.0, 80.25, 585.0),
  }
}

#[test]
fn test_read_write() -> XrfResult {
  let filename: String = String::from("read_write.cform");
  let mut writer: ChunkWriter = ChunkWriter::new();
  let original: LevelCformHeader = sample();

  original.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.bytes_written() as u64, LevelCformHeader::SIZE);

  writer.flush_raw_into(&mut overwrite_generated_test_resource_as_file(
    &build_relative_test_sample_file_path(file!(), &filename),
  )?)?;

  let read: LevelCformFile = LevelCformFile::read_from_file::<XRayByteOrder>(open_generated_test_resource_as_file(
    &build_relative_test_sample_file_path(file!(), &filename),
  )?)?;

  assert_eq!(read.header, original);

  Ok(())
}

#[test]
fn reads_the_vertices_and_faces_the_header_counts() -> XrfResult {
  let mut writer: ChunkWriter = ChunkWriter::new();
  let header: LevelCformHeader = LevelCformHeader {
    version: 4,
    vertex_count: 3,
    face_count: 1,
    aabb_min: Vector3d::new(0.0, 0.0, 0.0),
    aabb_max: Vector3d::new(1.0, 0.0, 1.0),
  };

  header.write::<XRayByteOrder>(&mut writer)?;

  let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

  for value in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] {
    bytes.extend_from_slice(&value.to_le_bytes());
  }

  for index in [0_u32, 2, 1] {
    bytes.extend_from_slice(&index.to_le_bytes());
  }

  // Material 5, shadows suppressed, sector 7.
  bytes.extend_from_slice(&(5_u32 | (1 << 14) | (7 << 16)).to_le_bytes());

  let (file, geometry) = LevelCformFile::read_with_geometry_from_bytes::<XRayByteOrder>(bytes)?;
  let face: LevelCformFace = geometry.get_faces()[0];

  assert_eq!(file.header, header);
  assert_eq!(geometry.get_vertices()[1], Vector3d::new(1.0, 0.0, 0.0));
  assert_eq!(face.vertices, [0, 2, 1]);
  assert_eq!(face.material, 5);
  assert!(face.is_shadow_suppressed);
  assert!(!face.is_wallmark_suppressed);
  assert_eq!(face.sector, 7);
  assert_eq!(geometry.get_triangle(&face)[1], Vector3d::new(0.0, 0.0, 1.0));

  Ok(())
}

#[test]
fn a_face_naming_a_vertex_past_the_count_is_an_error() -> XrfResult {
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelCformHeader {
    version: 4,
    vertex_count: 1,
    face_count: 1,
    aabb_min: Vector3d::new(0.0, 0.0, 0.0),
    aabb_max: Vector3d::new(0.0, 0.0, 0.0),
  }
  .write::<XRayByteOrder>(&mut writer)?;

  let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

  bytes.extend_from_slice(&[0; 12]);

  for index in [0_u32, 0, 3, 0] {
    bytes.extend_from_slice(&index.to_le_bytes());
  }

  assert!(LevelCformFile::read_with_geometry_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}

/// A collision form of the given corners and faces, each face of material nought.
fn new_form(vertices: &[[f32; 3]], faces: &[[u32; 3]]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelCformHeader {
    version: 4,
    vertex_count: vertices.len() as u32,
    face_count: faces.len() as u32,
    aabb_min: Vector3d::new(0.0, 0.0, 0.0),
    aabb_max: Vector3d::new(0.0, 0.0, 0.0),
  }
  .write::<XRayByteOrder>(&mut writer)?;

  let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

  for value in vertices.iter().flatten() {
    bytes.extend_from_slice(&value.to_le_bytes());
  }

  for face in faces {
    for index in face.iter().copied().chain([0]) {
      bytes.extend_from_slice(&index.to_le_bytes());
    }
  }

  Ok(bytes)
}

// A floor at nought over the corner x + z < 10, a roof at ten over x + z < 5, and a wall along x = 20.
#[test]
fn tells_which_points_have_nothing_straight_above_them() -> XrfResult {
  let bytes: Vec<u8> = new_form(
    &[
      [0.0, 0.0, 0.0],
      [10.0, 0.0, 0.0],
      [0.0, 0.0, 10.0],
      [0.0, 10.0, 0.0],
      [5.0, 10.0, 0.0],
      [0.0, 10.0, 5.0],
      [20.0, 0.0, 0.0],
      [20.0, 10.0, 0.0],
      [20.0, 0.0, 10.0],
    ],
    &[[0, 1, 2], [3, 5, 4], [6, 7, 8]],
  )?;
  let points: [Vector3d<f32>; 4] = [
    Vector3d::new(2.0, 1.7, 2.0),
    Vector3d::new(2.0, 11.0, 2.0),
    Vector3d::new(4.0, 1.7, 4.0),
    Vector3d::new(20.0, 1.7, 5.0),
  ];

  // Under the roof; over it; over the floor alone; beside a wall seen edge-on from above.
  assert_eq!(
    LevelCformFile::read_open_above_from_bytes::<XRayByteOrder>(bytes, &points)?,
    vec![false, true, true, true]
  );

  Ok(())
}

#[test]
fn a_face_naming_a_vertex_past_the_count_is_an_error_when_testing_the_sky() -> XrfResult {
  let bytes: Vec<u8> = new_form(&[[0.0, 0.0, 0.0]], &[[0, 0, 3]])?;

  assert!(LevelCformFile::read_open_above_from_bytes::<XRayByteOrder>(bytes, &[Vector3d::new(0.0, 0.0, 0.0)]).is_err());

  Ok(())
}

#[test]
fn truncated_header_is_an_error_and_not_a_panic() -> XrfResult {
  let filename: String = String::from("truncated.cform");
  let mut writer: ChunkWriter = ChunkWriter::new();

  sample().write::<XRayByteOrder>(&mut writer)?;

  let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

  bytes.truncate(LevelCformHeader::SIZE as usize - 1);

  overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?
    .write_all(&bytes)?;

  assert!(
    LevelCformFile::read_from_file::<XRayByteOrder>(open_generated_test_resource_as_file(
      &build_relative_test_sample_file_path(file!(), &filename)
    )?)
    .is_err(),
    "Expected truncated collision form header to fail reading"
  );

  Ok(())
}
