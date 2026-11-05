//! Editable objects built byte by byte, independent of the crate's writer, so a round trip proves the writer against
//! the layout `CEditableObject::Save` produces rather than against itself.

use crate::{EditableBone, EditableMesh, EditableObjectFile};

/// A chunk: id, payload size, payload.
pub(crate) fn chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(payload.len() + 8);

  bytes.extend_from_slice(&id.to_le_bytes());
  bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
  bytes.extend_from_slice(payload);

  bytes
}

/// A null-terminated string.
pub(crate) fn string(value: &str) -> Vec<u8> {
  let mut bytes: Vec<u8> = value.as_bytes().to_vec();

  bytes.push(0);

  bytes
}

/// Little-endian `u32`s, one after another.
pub(crate) fn words(values: &[u32]) -> Vec<u8> {
  values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

/// Little-endian `f32`s, one after another.
pub(crate) fn floats(values: &[f32]) -> Vec<u8> {
  values.iter().flat_map(|value| value.to_le_bytes()).collect()
}

/// Children numbered from zero, as the editor stores meshes and bones.
pub(crate) fn numbered(children: &[Vec<u8>]) -> Vec<u8> {
  children
    .iter()
    .enumerate()
    .flat_map(|(index, child)| chunk(index as u32, child))
    .collect()
}

/// One sentinel chunk nothing names, so a test can watch it survive.
pub(crate) const SENTINEL_ID: u32 = 0x5eed;

pub(crate) fn sentinel() -> Vec<u8> {
  chunk(SENTINEL_ID, &[0xde, 0xad, 0xbe, 0xef])
}

/// The vertex map chunk: one UV map over three entries, polygon-mapped where `is_polymap` asks.
pub(crate) fn vmaps_payload(is_polymap: bool) -> Vec<u8> {
  let mut payload: Vec<u8> = words(&[1]);

  payload.extend(string("Texture"));
  payload.extend([2, u8::from(is_polymap), 0]);
  payload.extend(words(&[3]));
  payload.extend(floats(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0]));
  payload.extend(words(&[0, 1, 2]));

  if is_polymap {
    payload.extend(words(&[0, 0, 0]));
  }

  payload
}

/// A mesh's children: one triangle over three points, with a smoothing group, a UV map and one surface.
pub(crate) fn mesh_children(vmaps: &[u8], tail: &[u8]) -> Vec<u8> {
  let mut faces: Vec<u8> = words(&[1]);

  faces.extend(words(&[0, 0, 1, 1, 2, 2]));

  let mut references: Vec<u8> = words(&[3]);

  for index in 0..3u32 {
    references.push(1);
    references.extend(words(&[0, index]));
  }

  let mut surface_faces: Vec<u8> = 1u16.to_le_bytes().to_vec();

  surface_faces.extend(string("default"));
  surface_faces.extend(words(&[1, 0]));

  let mut points: Vec<u8> = words(&[3]);

  points.extend(floats(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]));

  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend(chunk(EditableMesh::VERSION_CHUNK_ID, &0x0011u16.to_le_bytes()));
  bytes.extend(chunk(EditableMesh::NAME_CHUNK_ID, &string("triangle")));
  bytes.extend(chunk(
    EditableMesh::BOUNDS_CHUNK_ID,
    &floats(&[0.0, 0.0, 0.0, 1.0, 1.0, 0.0]),
  ));
  bytes.extend(chunk(EditableMesh::FLAGS_CHUNK_ID, &[1]));
  bytes.extend(chunk(EditableMesh::OPTIONS_CHUNK_ID, &words(&[0, 0])));
  bytes.extend(chunk(EditableMesh::POINTS_CHUNK_ID, &points));
  bytes.extend(chunk(EditableMesh::FACES_CHUNK_ID, &faces));
  bytes.extend(chunk(EditableMesh::SMOOTHING_GROUPS_CHUNK_ID, &words(&[1])));
  bytes.extend(chunk(EditableMesh::VMAP_REFERENCES_CHUNK_ID, &references));
  bytes.extend(chunk(EditableMesh::SURFACE_FACES_CHUNK_ID, &surface_faces));
  bytes.extend(chunk(EditableMesh::VMAPS_CHUNK_ID, vmaps));
  bytes.extend_from_slice(tail);

  bytes
}

/// A bone's children as `CBone::Save` writes them, the second definition included where `has_data_name` asks.
pub(crate) fn bone_children(has_data_name: bool, tail: &[u8]) -> Vec<u8> {
  let mut definition: Vec<u8> = string("root");

  definition.extend(string(""));
  definition.extend(string("root"));

  let mut bind_pose: Vec<u8> = floats(&[0.0, 0.5, 0.0, 0.0, 0.25, 0.0]);

  bind_pose.extend(floats(&[1.0]));

  let mut shape: Vec<u8> = vec![2, 0, 1, 0];

  shape.extend(floats(&[0.5; 15]));
  shape.extend(floats(&[0.0, 0.0, 0.0, 0.25]));
  shape.extend(floats(&[0.0; 8]));

  let mut joint: Vec<u8> = words(&[2]);

  joint.extend(floats(&[
    -0.5, 0.5, 1.0, 1.0, -0.25, 0.25, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
  ]));
  joint.extend(floats(&[1.0, 1.0]));

  let mut mass: Vec<u8> = floats(&[10.0]);

  mass.extend(floats(&[0.0, 0.1, 0.0]));

  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend(chunk(EditableBone::VERSION_CHUNK_ID, &2u16.to_le_bytes()));
  bytes.extend(chunk(EditableBone::DEFINITION_CHUNK_ID, &definition));
  bytes.extend(chunk(EditableBone::BIND_POSE_CHUNK_ID, &bind_pose));

  if has_data_name {
    bytes.extend(chunk(EditableBone::DEFINITION_CHUNK_ID, &string("root")));
  }

  bytes.extend(chunk(EditableBone::MATERIAL_CHUNK_ID, &string("default_object")));
  bytes.extend(chunk(EditableBone::SHAPE_CHUNK_ID, &shape));
  bytes.extend(chunk(EditableBone::FLAGS_CHUNK_ID, &words(&[0])));
  bytes.extend(chunk(EditableBone::JOINT_CHUNK_ID, &joint));
  bytes.extend(chunk(EditableBone::BREAK_CHUNK_ID, &floats(&[0.0, 0.0])));
  bytes.extend(chunk(EditableBone::FRICTION_CHUNK_ID, &floats(&[0.0])));
  bytes.extend(chunk(EditableBone::MASS_CHUNK_ID, &mass));
  bytes.extend_from_slice(tail);

  bytes
}

/// The surfaces chunk: one surface the mesh's face list names.
pub(crate) fn surfaces_payload() -> Vec<u8> {
  let mut payload: Vec<u8> = words(&[1]);

  for value in [
    "default",
    "models\\model",
    "default",
    "default_object",
    "ed\\ed_axis",
    "Texture",
  ] {
    payload.extend(string(value));
  }

  payload.extend(words(&[1, 0x112, 1]));

  payload
}

/// The body's children for a dynamic object with one mesh, one bone and every chunk `Save` writes for it.
pub(crate) fn body_children(mesh: &[u8], bone: &[u8], tail: &[u8]) -> Vec<u8> {
  let mut references: Vec<u8> = words(&[2]);

  references.extend(string("actors\\stalker"));
  references.extend(string("actors\\idle"));

  let mut parts: Vec<u8> = words(&[1]);

  parts.extend(string("default"));
  parts.extend(words(&[1]));
  parts.extend(string("root"));

  let mut description: Vec<u8> = string("gsc");

  description.extend(words(&[1_100_000_000]));
  description.extend(string("\\\\MAINPC\\gladi"));
  description.extend(words(&[1_200_000_000]));

  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend(chunk(EditableObjectFile::VERSION_CHUNK_ID, &0x0010u16.to_le_bytes()));
  bytes.extend(chunk(EditableObjectFile::CLASS_SCRIPT_CHUNK_ID, &string("")));
  bytes.extend(chunk(EditableObjectFile::LODS_CHUNK_ID, &string("")));
  bytes.extend(chunk(EditableObjectFile::FLAGS_CHUNK_ID, &words(&[1])));
  bytes.extend(chunk(EditableObjectFile::MESHES_CHUNK_ID, &numbered(&[mesh.to_vec()])));
  bytes.extend(chunk(EditableObjectFile::SURFACES_CHUNK_ID, &surfaces_payload()));
  bytes.extend(chunk(EditableObjectFile::BONES_CHUNK_ID, &numbered(&[bone.to_vec()])));
  bytes.extend(chunk(EditableObjectFile::MOTION_REFERENCES_CHUNK_ID, &references));
  bytes.extend(chunk(EditableObjectFile::BONE_PARTS_CHUNK_ID, &parts));
  bytes.extend(chunk(
    EditableObjectFile::ACTOR_TRANSFORM_CHUNK_ID,
    &floats(&[0.0, 0.0, 0.0, 0.0, 3.0, 0.0]),
  ));
  bytes.extend(chunk(EditableObjectFile::DESCRIPTION_CHUNK_ID, &description));
  bytes.extend_from_slice(tail);

  bytes
}

/// A whole file around a body.
pub(crate) fn file(body: &[u8]) -> Vec<u8> {
  chunk(EditableObjectFile::BODY_CHUNK_ID, body)
}

/// The dynamic object every test starts from.
pub(crate) fn dynamic_object() -> Vec<u8> {
  file(&body_children(
    &mesh_children(&vmaps_payload(false), &[]),
    &bone_children(true, &[]),
    &[],
  ))
}
