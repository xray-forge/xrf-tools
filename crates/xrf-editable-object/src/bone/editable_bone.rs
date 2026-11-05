use byteorder::{ByteOrder, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ogf::OgfBoneShape;

use crate::chunks::editable_chunk_sequence::EditableChunkSequence;
use crate::chunks::editable_chunks::EditableChunks;
use crate::{
  EditableBoneBindPose, EditableBoneBreak, EditableBoneDefinition, EditableBoneJoint, EditableBoneMass,
  EditableExtraChunk,
};

/// One bone of an editable object's skeleton, `CBone` as `CBone::Save` stores it.
///
/// The editor writes `BONE_CHUNK_DEF` twice: its place in the skeleton first, then the name alone again ahead of the
/// physics data `CBone::SaveData` writes. `CBone::Load_1` reads only the first, so the second is
/// [`Self::data_name`], kept to write back as stored.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableBone {
  /// `BONE_VERSION`: 1 or 2, which the SDK both loads.
  pub version: u16,
  pub definition: EditableBoneDefinition,
  pub bind_pose: EditableBoneBindPose,
  /// The name the second definition chunk repeats, `None` where it is absent.
  pub data_name: Option<String>,
  /// The game material the bone's collision shape is made of.
  pub game_material: String,
  // todo: Move `OgfBoneShape`, `OgfJointLimit` and the shapes they hold into a crate the OGF and the editable object
  // both read, without the `Ogf` prefix, as `xrf-skeleton` holds the motions both OGF and OMF read.
  pub shape: OgfBoneShape,
  /// `SJointIKData::ik_flags`, `None` where the chunk is absent.
  pub joint_flags: Option<u32>,
  pub joint: EditableBoneJoint,
  pub joint_break: Option<EditableBoneBreak>,
  pub friction: Option<f32>,
  pub mass: Option<EditableBoneMass>,
  /// Chunks no field takes, see [`EditableExtraChunk`].
  pub extra: Vec<EditableExtraChunk>,
  /// The bone's chunk ids in the order the file stored them, which the writer follows; empty for a bone built in
  /// code.
  pub chunk_order: Vec<u32>,
}

impl EditableBone {
  /// The versions `CBone::Load_1` reads.
  pub const SUPPORTED_VERSIONS: [u16; 2] = [1, 2];

  /// `BONE_VERSION`, the one `CBone::Save` writes.
  pub const CURRENT_VERSION: u16 = 2;

  pub const VERSION_CHUNK_ID: u32 = 0x0001;
  pub const DEFINITION_CHUNK_ID: u32 = 0x0002;
  pub const BIND_POSE_CHUNK_ID: u32 = 0x0003;
  pub const MATERIAL_CHUNK_ID: u32 = 0x0004;
  pub const SHAPE_CHUNK_ID: u32 = 0x0005;
  pub const JOINT_CHUNK_ID: u32 = 0x0006;
  pub const MASS_CHUNK_ID: u32 = 0x0007;
  pub const FLAGS_CHUNK_ID: u32 = 0x0008;
  pub const BREAK_CHUNK_ID: u32 = 0x0009;
  pub const FRICTION_CHUNK_ID: u32 = 0x0010;

  /// Reads the children of one numbered bone chunk.
  ///
  /// # Errors
  ///
  /// Returns an error where the SDK asserts — a missing version, definition, bind pose, material, shape or joint
  /// chunk — for a version [`Self::SUPPORTED_VERSIONS`] does not hold, and where a chunk holds more or less than its
  /// fields.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut version: Option<u16> = None;
    let mut definition: Option<EditableBoneDefinition> = None;
    let mut bind_pose: Option<EditableBoneBindPose> = None;
    let mut data_name: Option<String> = None;
    let mut game_material: Option<String> = None;
    let mut shape: Option<OgfBoneShape> = None;
    let mut joint_flags: Option<u32> = None;
    let mut joint: Option<EditableBoneJoint> = None;
    let mut joint_break: Option<EditableBoneBreak> = None;
    let mut friction: Option<f32> = None;
    let mut mass: Option<EditableBoneMass> = None;
    let mut extra: Vec<EditableExtraChunk> = Vec::new();

    let children: Vec<ChunkReader<D>> = reader.read_children()?;
    let chunk_order: Vec<u32> = children.iter().map(|chunk| chunk.id).collect();

    for mut chunk in children {
      match chunk.id {
        Self::VERSION_CHUNK_ID if version.is_none() => {
          version = Some(chunk.read_u16_payload::<T>()?);
        }
        Self::DEFINITION_CHUNK_ID if definition.is_none() => {
          definition = Some(EditableChunks::read_whole(&mut chunk, "bone definition", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        Self::DEFINITION_CHUNK_ID if data_name.is_none() => {
          data_name = Some(chunk.read_w1251_string_payload()?);
        }
        Self::BIND_POSE_CHUNK_ID if bind_pose.is_none() => {
          bind_pose = Some(EditableChunks::read_whole(&mut chunk, "bone bind pose", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        Self::MATERIAL_CHUNK_ID if game_material.is_none() => {
          game_material = Some(chunk.read_w1251_string_payload()?);
        }
        Self::SHAPE_CHUNK_ID if shape.is_none() => {
          shape = Some(EditableChunks::read_whole(&mut chunk, "bone shape", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        Self::FLAGS_CHUNK_ID if joint_flags.is_none() => {
          joint_flags = Some(chunk.read_u32_payload::<T>()?);
        }
        Self::JOINT_CHUNK_ID if joint.is_none() => {
          joint = Some(EditableChunks::read_whole(&mut chunk, "bone joint", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        Self::BREAK_CHUNK_ID if joint_break.is_none() => {
          joint_break = Some(EditableChunks::read_whole(&mut chunk, "bone joint break", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        Self::FRICTION_CHUNK_ID if friction.is_none() => {
          friction = Some(chunk.read_f32_payload::<T>()?);
        }
        Self::MASS_CHUNK_ID if mass.is_none() => {
          mass = Some(EditableChunks::read_whole(&mut chunk, "bone mass", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        _ => EditableChunks::keep(&mut chunk, &mut extra)?,
      }
    }

    let version: u16 = EditableChunks::require(version, "bone version")?;

    if !Self::SUPPORTED_VERSIONS.contains(&version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected bone version {version}, the SDK loads only versions {:?}",
        Self::SUPPORTED_VERSIONS
      )));
    }

    Ok(Self {
      version,
      definition: EditableChunks::require(definition, "bone definition")?,
      bind_pose: EditableChunks::require(bind_pose, "bone bind pose")?,
      data_name,
      game_material: EditableChunks::require(game_material, "bone material")?,
      shape: EditableChunks::require(shape, "bone shape")?,
      joint_flags,
      joint: EditableChunks::require(joint, "bone joint")?,
      joint_break,
      friction,
      mass,
      extra,
      chunk_order,
    })
  }

  /// Writes the bone's children in [`Self::chunk_order`], and any it does not list in the order `CBone::Save` writes
  /// them, the extra chunks last.
  ///
  /// # Errors
  ///
  /// Returns an error when a string cannot be encoded.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut sequence: EditableChunkSequence = EditableChunkSequence::new();

    sequence.push(Self::VERSION_CHUNK_ID, |writer| {
      Ok(writer.write_u16::<T>(self.version)?)
    })?;
    sequence.push(Self::DEFINITION_CHUNK_ID, |writer| self.definition.write::<T>(writer))?;
    sequence.push(Self::BIND_POSE_CHUNK_ID, |writer| self.bind_pose.write::<T>(writer))?;

    if let Some(data_name) = &self.data_name {
      sequence.push(Self::DEFINITION_CHUNK_ID, |writer| {
        writer.write_w1251_string(data_name)?;

        Ok(())
      })?;
    }

    sequence.push(Self::MATERIAL_CHUNK_ID, |writer| {
      writer.write_w1251_string(&self.game_material)?;

      Ok(())
    })?;
    sequence.push(Self::SHAPE_CHUNK_ID, |writer| self.shape.write::<T>(writer))?;

    if let Some(flags) = self.joint_flags {
      sequence.push(Self::FLAGS_CHUNK_ID, |writer| Ok(writer.write_u32::<T>(flags)?))?;
    }

    sequence.push(Self::JOINT_CHUNK_ID, |writer| self.joint.write::<T>(writer))?;

    if let Some(joint_break) = &self.joint_break {
      sequence.push(Self::BREAK_CHUNK_ID, |writer| joint_break.write::<T>(writer))?;
    }

    if let Some(friction) = self.friction {
      sequence.push(Self::FRICTION_CHUNK_ID, |writer| Ok(writer.write_f32::<T>(friction)?))?;
    }

    if let Some(mass) = &self.mass {
      sequence.push(Self::MASS_CHUNK_ID, |writer| mass.write::<T>(writer))?;
    }

    sequence.push_extra(&self.extra);

    sequence.write_into::<T>(writer, &self.chunk_order)
  }
}
