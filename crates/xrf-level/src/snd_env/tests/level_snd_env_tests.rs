use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;

use crate::snd_env::level_snd_env_face::LevelSndEnvFace;
use crate::snd_env::level_snd_env_file::LevelSndEnvFile;
use crate::tests::fixtures::{chunk, floats, string};

/// A mesh of one triangle between `front` and `back`, over the names given, with a version-4 header.
fn file_bytes(names: &[&str], face: (u32, u16, u16)) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(
    LevelSndEnvFile::NAMES_CHUNK_ID,
    &names.iter().flat_map(|name| string(name)).collect::<Vec<u8>>(),
  )?;
  let mut geometry: Vec<u8> = [4u32, 3, 1].iter().flat_map(|value| value.to_le_bytes()).collect();

  geometry.extend(floats(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]));
  geometry.extend(floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0]));

  for index in [0u32, 1, face.0] {
    geometry.extend(index.to_le_bytes());
  }

  geometry.extend((u32::from(face.1) | (u32::from(face.2) << 16)).to_le_bytes());
  bytes.extend(chunk(LevelSndEnvFile::GEOMETRY_CHUNK_ID, &geometry)?);

  Ok(bytes)
}

#[test]
fn reads_the_environments_and_the_faces_between_them() -> XrfResult {
  let file: LevelSndEnvFile =
    LevelSndEnvFile::read_from_bytes::<XRayByteOrder>(file_bytes(&["open_space", "bathroom"], (2, 1, 0))?)?;

  assert_eq!(file.environments, ["open_space", "bathroom"]);
  assert_eq!(file.version, 4);
  assert_eq!(file.vertices.len(), 3);
  assert_eq!(
    file.faces,
    [LevelSndEnvFace {
      vertices: [0, 1, 2],
      front: 1,
      back: 0,
    }]
  );
  assert_eq!(file.get_front_environment(&file.faces[0]), Some("bathroom"));
  assert_eq!(file.get_back_environment(&file.faces[0]), Some("open_space"));

  Ok(())
}

#[test]
fn writes_the_mesh_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = file_bytes(&["open_space", "bathroom"], (2, 1, 0))?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelSndEnvFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn a_face_naming_an_environment_the_file_lacks_is_refused() -> XrfResult {
  let error: String = LevelSndEnvFile::read_from_bytes::<XRayByteOrder>(file_bytes(&["open_space"], (2, 0, 1))?)
    .expect_err("environment 1 of 1 is refused, as the engine asserts")
    .to_string();

  assert_eq!(
    error,
    "Invalid error: A sound environment face names environment 1 of 1"
  );

  Ok(())
}

#[test]
fn a_face_naming_a_vertex_the_mesh_lacks_is_refused() -> XrfResult {
  let error: String = LevelSndEnvFile::read_from_bytes::<XRayByteOrder>(file_bytes(&["open_space"], (3, 0, 0))?)
    .expect_err("vertex 3 of 3 is refused")
    .to_string();

  assert_eq!(error, "Invalid error: A sound environment face names vertex 3 of 3");

  Ok(())
}

#[test]
fn a_mesh_with_bytes_past_its_faces_is_refused() -> XrfResult {
  let mut bytes: Vec<u8> = chunk(LevelSndEnvFile::NAMES_CHUNK_ID, &string("open_space"))?;
  let mut geometry: Vec<u8> = [4u32, 0, 0].iter().flat_map(|value| value.to_le_bytes()).collect();

  geometry.extend(floats(&[0.0; 6]));
  geometry.push(0);
  bytes.extend(chunk(LevelSndEnvFile::GEOMETRY_CHUNK_ID, &geometry)?);

  assert!(LevelSndEnvFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}
