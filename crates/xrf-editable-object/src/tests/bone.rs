use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_skeleton::{SkeletonBoneShapeKind, SkeletonJointKind};

use crate::EditableBone;
use crate::tests::fixtures;

fn read(children: &[u8]) -> XrfResult<EditableBone> {
  let mut reader: ChunkReader<InMemoryChunkDataSource> =
    ChunkReader::from_vec(fixtures::chunk(0, children))?.read_child_by_index(0)?;

  EditableBone::read::<XRayByteOrder, _>(&mut reader)
}

fn write(bone: &EditableBone) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  bone.write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

#[test]
fn a_bone_reads_its_place_pose_and_physics() -> XrfResult {
  let bone: EditableBone = read(&fixtures::bone_children(true, &[]))?;

  assert_eq!(bone.version, EditableBone::CURRENT_VERSION);
  assert_eq!(bone.definition.name, "root");
  assert_eq!(bone.definition.parent, "");
  assert_eq!(bone.definition.weight_map, "root");
  assert_eq!(bone.bind_pose.length, 1.0);
  assert_eq!(bone.data_name.as_deref(), Some("root"));
  assert_eq!(bone.game_material, "default_object");
  assert_eq!(bone.shape.kind, SkeletonBoneShapeKind::Sphere);
  assert_eq!(bone.joint.kind, SkeletonJointKind::Joint);
  assert_eq!(bone.joint.limits[1].min, -0.25);
  assert_eq!(bone.mass.as_ref().map(|mass| mass.mass), Some(10.0));

  Ok(())
}

#[test]
fn a_bone_is_written_back_byte_for_byte_with_its_definition_repeated() -> XrfResult {
  // `CBone::Save` writes the definition chunk twice; both come back, in place.
  for has_data_name in [true, false] {
    let original: Vec<u8> = fixtures::bone_children(has_data_name, &[]);

    assert_eq!(write(&read(&original)?)?, original, "data name {has_data_name}");
  }

  Ok(())
}

#[test]
fn a_bone_without_a_shape_is_refused_as_the_editor_asserts() {
  let mut children: Vec<u8> = Vec::new();

  for chunk in ChunkReader::from_vec(fixtures::bone_children(true, &[]))
    .and_then(|mut reader| reader.read_children())
    .expect("expect the fixture to read")
  {
    let mut chunk = chunk;

    if chunk.id != EditableBone::SHAPE_CHUNK_ID {
      let payload: Vec<u8> = chunk.read_remaining().expect("expect the payload");

      children.extend(fixtures::chunk(chunk.id, &payload));
    }
  }

  let error: String = read(&children).expect_err("expect no shape refused").to_string();

  assert!(error.contains("bone shape"), "Unexpected error: {error}");
}

#[test]
fn a_bone_version_the_editor_does_not_load_is_refused() {
  let mut children: Vec<u8> = fixtures::bone_children(true, &[]);

  children[8] = 3;

  let error: String = read(&children).expect_err("expect version 3 refused").to_string();

  assert!(error.contains("bone version 3"), "Unexpected error: {error}");
}
