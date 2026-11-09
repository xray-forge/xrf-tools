use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::{ByteOrder, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkLayout, ChunkReadWrite, ChunkReader, ChunkSequence, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path, narrow_format_size};

use crate::editable_chunks::EditableChunks;
use crate::{
  EditableActorTransform, EditableBone, EditableBonePart, EditableDescription, EditableMesh, EditableMotion,
  EditableSurface,
};

/// An editable object, the `.object` file the SDK's Actor and Level editors open: meshes, the surfaces their faces
/// are made of, and for a dynamic object its skeleton. It is the source an OGF is exported from, and holds what the
/// export drops: shared points, vertex maps, smoothing groups and every surface's names.
///
/// Every chunk the SDK names is a field. How the file laid them out, and any chunk no field takes, is the body's,
/// each mesh's and each bone's [`ChunkLayout`], so a file reads and writes back byte for byte whichever editor
/// or exporter saved it, and an object built in code is written in the order `CEditableObject::Save` writes.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableObjectFile {
  /// `CEditableObject::m_objectFlags`: dynamic, HOM, multiple usage, sound occluder and the rest.
  pub flags: u32,
  /// Bytes the flags chunk holds: 4 as the editor writes it, 1 in an older object. `CEditableObject::Load` reads it
  /// with `r_chunk`, which copies however many the chunk holds into the low bytes.
  pub flags_size: u8,
  /// The script class the Level Editor gives the object, `None` where the chunk is absent.
  pub class_script: Option<String>,
  /// The LOD reference, `None` where the chunk is absent.
  pub lods: Option<String>,
  /// `None` where the object stores no mesh list at all.
  pub meshes: Option<Vec<EditableMesh>>,
  // todo: Read the older `EOBJ_CHUNK_SURFACES` and `_SURFACES2` layouts the SDK still loads, which stay extra chunks.
  pub surfaces: Option<Vec<EditableSurface>>,
  pub bones: Option<Vec<EditableBone>>,
  /// Skeleton motions stored in the object itself, `EOBJ_CHUNK_SMOTIONS`, kept as stored bytes so an older layout
  /// writes back as it was; [`Self::get_embedded_motions`] reads them.
  pub embedded_motions: Option<Vec<u8>>,
  /// The `.skls` motion files the object plays, by path.
  pub motion_references: Option<Vec<String>>,
  pub bone_parts: Option<Vec<EditableBonePart>>,
  pub actor_transform: Option<EditableActorTransform>,
  pub description: Option<EditableDescription>,
  /// How the body's chunks were laid out in its file.
  pub layout: ChunkLayout,
}

impl EditableObjectFile {
  /// `EOBJ_CURRENT_VERSION`, the only version `CEditableObject::Load` accepts.
  pub const VERSION: u16 = 0x0010;

  /// `EOBJ_CHUNK_OBJECT_BODY`, the one chunk at the root of the file that holds every other.
  pub const BODY_CHUNK_ID: u32 = 0x7777;

  pub const VERSION_CHUNK_ID: u32 = 0x0900;
  pub const FLAGS_CHUNK_ID: u32 = 0x0903;
  pub const SURFACES_CHUNK_ID: u32 = 0x0907;
  pub const MESHES_CHUNK_ID: u32 = 0x0910;
  pub const CLASS_SCRIPT_CHUNK_ID: u32 = 0x0912;
  pub const EMBEDDED_MOTIONS_CHUNK_ID: u32 = 0x0916;
  pub const ACTOR_TRANSFORM_CHUNK_ID: u32 = 0x0920;
  pub const BONES_CHUNK_ID: u32 = 0x0921;
  pub const DESCRIPTION_CHUNK_ID: u32 = 0x0922;
  pub const BONE_PARTS_CHUNK_ID: u32 = 0x0923;
  pub const LODS_CHUNK_ID: u32 = 0x0925;
  pub const MOTION_REFERENCES_CHUNK_ID: u32 = 0x0926;

  /// `CEditableObject::eoDynamic`: the object is a skinned or animated model rather than level geometry.
  pub const DYNAMIC_FLAG: u32 = 1 << 0;

  /// `CEditableObject::eoProgressive`: the export builds sliding-window detail levels of the mesh.
  pub const PROGRESSIVE_FLAG: u32 = 1 << 1;

  /// Bytes the flags chunk holds as the current editor writes it.
  pub const FLAGS_SIZE: u8 = 4;

  /// The skeleton motions the object stores itself, none where it stores no chunk of them.
  ///
  /// # Errors
  ///
  /// Returns an error where the stored motions are not ones [`EditableMotion::read_list`] reads.
  pub fn get_embedded_motions<T: ByteOrder>(&self) -> XrfResult<Vec<EditableMotion>> {
    self
      .embedded_motions
      .as_deref()
      .map_or_else(|| Ok(Vec::new()), EditableMotion::read_list::<T>)
  }

  /// Reads an editable object from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not an editable object [`Self::read_from_chunk`] reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Editable object was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads an editable object from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not an editable object [`Self::read_from_chunk`] reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads an editable object from bytes already in hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not an editable object [`Self::read_from_chunk`] reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads the file's root, which must be the body chunk alone: the SDK reads nothing else, so anything beside it
  /// would be dropped on the way back.
  ///
  /// # Errors
  ///
  /// Returns an error when the root holds anything but one body chunk, or the body is not one
  /// [`Self::read_from_body_chunk`] reads.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    match chunks.as_mut_slice() {
      [body] if body.id == Self::BODY_CHUNK_ID => Self::read_from_body_chunk::<T, D>(body),
      _ => Err(XrfError::new_parsing_error(format!(
        "Editable object root holds chunks {:?}, expected the body chunk {} alone",
        chunks.iter().map(|chunk| chunk.id).collect::<Vec<u32>>(),
        Self::BODY_CHUNK_ID
      ))),
    }
  }

  /// Reads the body chunk's children, as `CEditableObject::Load` finds them.
  ///
  /// # Errors
  ///
  /// Returns an error where the SDK refuses or asserts: a version other than [`Self::VERSION`], or no flags chunk; and
  /// where a chunk holds more or less than its fields, a mesh or bone list is not numbered from zero, a count runs
  /// past its chunk, or a mesh or bone fails to read.
  pub fn read_from_body_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut version: Option<u16> = None;
    let mut flags: Option<u32> = None;
    let mut flags_size: u8 = Self::FLAGS_SIZE;
    let mut class_script: Option<String> = None;
    let mut lods: Option<String> = None;
    let mut meshes: Option<Vec<EditableMesh>> = None;
    let mut surfaces: Option<Vec<EditableSurface>> = None;
    let mut bones: Option<Vec<EditableBone>> = None;
    let mut embedded_motions: Option<Vec<u8>> = None;
    let mut motion_references: Option<Vec<String>> = None;
    let mut bone_parts: Option<Vec<EditableBonePart>> = None;
    let mut actor_transform: Option<EditableActorTransform> = None;
    let mut description: Option<EditableDescription> = None;

    let layout: ChunkLayout = ChunkLayout::read(reader, |chunk| {
      match chunk.id {
        Self::VERSION_CHUNK_ID if version.is_none() => version = Some(chunk.read_u16_payload::<T>()?),
        Self::FLAGS_CHUNK_ID if flags.is_none() => {
          flags_size = Self::check_flags_size(chunk.read_bytes_remain())?;
          flags = Some(
            u32::try_from(T::read_uint(&chunk.read_remaining()?, flags_size.into()))
              .map_err(|_| XrfError::new_parsing_error("Object flags do not fit 32 bits"))?,
          );
        }
        Self::CLASS_SCRIPT_CHUNK_ID if class_script.is_none() => {
          class_script = Some(chunk.read_w1251_string_payload()?);
        }
        Self::LODS_CHUNK_ID if lods.is_none() => lods = Some(chunk.read_w1251_string_payload()?),
        Self::MESHES_CHUNK_ID if meshes.is_none() => {
          meshes = Some(EditableChunks::read_numbered(chunk, "mesh", |chunk| {
            EditableMesh::read::<T, D>(chunk)
          })?);
        }
        Self::SURFACES_CHUNK_ID if surfaces.is_none() => {
          surfaces = Some(EditableChunks::read_whole(chunk, "surfaces", |chunk| {
            EditableChunks::read_counted::<T, D, _>(chunk, EditableSurface::MIN_SERIALIZED_SIZE, "surfaces", |chunk| {
              chunk.read_xr::<T, _>()
            })
          })?);
        }
        Self::BONES_CHUNK_ID if bones.is_none() => {
          bones = Some(EditableChunks::read_numbered(chunk, "bone", |chunk| {
            EditableBone::read::<T, D>(chunk)
          })?);
        }
        Self::EMBEDDED_MOTIONS_CHUNK_ID if embedded_motions.is_none() => {
          embedded_motions = Some(chunk.read_remaining()?);
        }
        Self::MOTION_REFERENCES_CHUNK_ID if motion_references.is_none() => {
          motion_references = Some(EditableChunks::read_whole(chunk, "motion references", |chunk| {
            EditableChunks::read_counted::<T, D, _>(chunk, 1, "motion references", |chunk| chunk.read_w1251_string())
          })?);
        }
        Self::BONE_PARTS_CHUNK_ID if bone_parts.is_none() => {
          bone_parts = Some(EditableChunks::read_whole(chunk, "bone parts", |chunk| {
            EditableChunks::read_counted::<T, D, _>(
              chunk,
              EditableBonePart::MIN_SERIALIZED_SIZE,
              "bone parts",
              |chunk| chunk.read_xr::<T, _>(),
            )
          })?);
        }
        Self::ACTOR_TRANSFORM_CHUNK_ID if actor_transform.is_none() => {
          actor_transform = Some(EditableChunks::read_whole(chunk, "actor transform", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        Self::DESCRIPTION_CHUNK_ID if description.is_none() => {
          description = Some(EditableChunks::read_whole(chunk, "description", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        _ => return Ok(false),
      }

      Ok(true)
    })?;

    EditableChunks::require_version(
      EditableChunks::require(version, "object version")?,
      Self::VERSION,
      "object",
    )?;

    Ok(Self {
      flags: EditableChunks::require(flags, "object flags")?,
      flags_size,
      class_script,
      lods,
      meshes,
      surfaces,
      bones,
      embedded_motions,
      motion_references,
      bone_parts,
      actor_transform,
      description,
      layout,
    })
  }

  /// Writes the object to a path, creating the directories leading to it.
  ///
  /// # Errors
  ///
  /// Returns an error when the object cannot be written as [`Self::write_to_bytes`] says, or the file cannot be.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    Ok(StagedFile::write_creating_directories(
      path.as_ref(),
      &self.write_to_bytes::<T>()?,
    )?)
  }

  /// The object as the bytes of a file: the body chunk alone, as `CEditableObject::Save` frames it.
  ///
  /// # Errors
  ///
  /// Returns an error when a list is longer than its count holds, a mesh's lists disagree, or a string cannot be
  /// encoded.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    self.write_body::<T>(&mut writer)?;

    writer.flush_chunk_into_buffer::<T>(Self::BODY_CHUNK_ID)
  }

  /// Writes the body's children as [`Self::layout`] lays them out, and any it does not list in the order
  /// `CEditableObject::Save` writes them.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_bytes`] does.
  pub fn write_body<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut sequence: ChunkSequence = ChunkSequence::new();

    sequence.push(Self::VERSION_CHUNK_ID, |writer| {
      Ok(writer.write_u16::<T>(Self::VERSION)?)
    })?;

    if let Some(class_script) = &self.class_script {
      sequence.push(Self::CLASS_SCRIPT_CHUNK_ID, |writer| {
        writer.write_w1251_string(class_script)?;

        Ok(())
      })?;
    }

    if let Some(lods) = &self.lods {
      sequence.push(Self::LODS_CHUNK_ID, |writer| {
        writer.write_w1251_string(lods)?;

        Ok(())
      })?;
    }

    sequence.push(Self::FLAGS_CHUNK_ID, |writer| {
      let size: usize = Self::check_flags_size(self.flags_size.into())?.into();

      if size < 4 && self.flags >> (size * 8) != 0 {
        return Err(XrfError::new_invalid_error(format!(
          "Object flags {:#x} do not fit the {size} bytes the flags chunk holds",
          self.flags
        )));
      }

      let mut bytes: [u8; 4] = [0; 4];

      T::write_uint(&mut bytes, self.flags.into(), size);

      Ok(writer.write_all(&bytes[..size])?)
    })?;

    if let Some(meshes) = &self.meshes {
      sequence.push(Self::MESHES_CHUNK_ID, |writer| {
        EditableChunks::write_numbered::<T, _>(writer, meshes, |mesh, writer| mesh.write::<T>(writer))
      })?;
    }

    if let Some(surfaces) = &self.surfaces {
      sequence.push(Self::SURFACES_CHUNK_ID, |writer| {
        EditableChunks::write_counted::<T, _>(writer, surfaces, "Surfaces")
      })?;
    }

    if let Some(bones) = &self.bones {
      sequence.push(Self::BONES_CHUNK_ID, |writer| {
        EditableChunks::write_numbered::<T, _>(writer, bones, |bone, writer| bone.write::<T>(writer))
      })?;
    }

    if let Some(motions) = &self.embedded_motions {
      sequence.push(Self::EMBEDDED_MOTIONS_CHUNK_ID, |writer| Ok(writer.write_all(motions)?))?;
    }

    if let Some(references) = &self.motion_references {
      sequence.push(Self::MOTION_REFERENCES_CHUNK_ID, |writer| {
        writer.write_u32::<T>(narrow_format_size(references.len(), "Motion references")?)?;

        for reference in references {
          writer.write_w1251_string(reference)?;
        }

        Ok(())
      })?;
    }

    if let Some(parts) = &self.bone_parts {
      sequence.push(Self::BONE_PARTS_CHUNK_ID, |writer| {
        EditableChunks::write_counted::<T, _>(writer, parts, "Bone parts")
      })?;
    }

    if let Some(transform) = &self.actor_transform {
      sequence.push(Self::ACTOR_TRANSFORM_CHUNK_ID, |writer| transform.write::<T>(writer))?;
    }

    if let Some(description) = &self.description {
      sequence.push(Self::DESCRIPTION_CHUNK_ID, |writer| description.write::<T>(writer))?;
    }

    self.layout.write::<T>(writer, sequence)
  }

  /// Whether the object is a dynamic model, which the Actor Editor exports to an OGF of its own, rather than level
  /// geometry a level build merges.
  pub const fn is_dynamic(&self) -> bool {
    self.flags & Self::DYNAMIC_FLAG != 0
  }

  /// Refuses a flags chunk `r_chunk` could not read into the flags: empty, or wider than them.
  fn check_flags_size(size: u64) -> XrfResult<u8> {
    match u8::try_from(size) {
      Ok(size @ 1..=Self::FLAGS_SIZE) => Ok(size),
      _ => Err(XrfError::new_parsing_error(format!(
        "Unexpected object flags chunk of {size} bytes, the editor reads 1 to {}",
        Self::FLAGS_SIZE
      ))),
    }
  }
}
