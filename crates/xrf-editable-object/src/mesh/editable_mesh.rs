use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::to_format_size;

use crate::chunks::editable_chunk_sequence::EditableChunkSequence;
use crate::chunks::editable_chunks::EditableChunks;
use crate::{
  EditableExtraChunk, EditableFace, EditableMeshBounds, EditableSurfaceFaces, EditableVMap, EditableVMapReference,
};

/// One mesh of an editable object, `CEditableMesh` as `CEditableMesh::SaveMesh` stores it: points, triangles that
/// index them, the surfaces the triangles are made of, and the vertex maps their corners take UVs and weights from.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableMesh {
  pub name: String,
  pub bounds: EditableMeshBounds,
  /// `flVisible`, `flLocked` and `flSGMask`, the last saying how a mesh without smoothing groups is smoothed.
  pub flags: u8,
  /// `st_MeshOptions`, two reserved numbers the editor zeroes. `None` where the chunk is absent, which the SDK
  /// accepts.
  pub options: Option<[u32; 2]>,
  pub points: Vec<Vector3d>,
  pub faces: Vec<EditableFace>,
  /// One smoothing group mask per face, `None` where the mesh stores none.
  pub smoothing_groups: Option<Vec<u32>>,
  pub vmap_references: Vec<EditableVMapReference>,
  pub surface_faces: Vec<EditableSurfaceFaces>,
  /// `None` where the mesh stores no `EMESH_CHUNK_VMAPS_2`.
  // todo: Read the older `EMESH_CHUNK_VMAPS_0` and `_1` layouts the SDK still loads, which stay extra chunks here.
  pub vmaps: Option<Vec<EditableVMap>>,
  /// Chunks no field takes, see [`EditableExtraChunk`].
  pub extra: Vec<EditableExtraChunk>,
  /// The mesh's chunk ids in the order the file stored them, which the writer follows; empty for a mesh built in
  /// code.
  pub chunk_order: Vec<u32>,
}

impl EditableMesh {
  /// `EMESH_CURRENT_VERSION`, the only version `CEditableMesh::LoadMesh` accepts.
  pub const VERSION: u16 = 0x0011;

  pub const VERSION_CHUNK_ID: u32 = 0x1000;
  pub const NAME_CHUNK_ID: u32 = 0x1001;
  pub const FLAGS_CHUNK_ID: u32 = 0x1002;
  pub const BOUNDS_CHUNK_ID: u32 = 0x1004;
  pub const POINTS_CHUNK_ID: u32 = 0x1005;
  pub const FACES_CHUNK_ID: u32 = 0x1006;
  pub const VMAP_REFERENCES_CHUNK_ID: u32 = 0x1008;
  pub const SURFACE_FACES_CHUNK_ID: u32 = 0x1009;
  pub const OPTIONS_CHUNK_ID: u32 = 0x1010;
  pub const VMAPS_CHUNK_ID: u32 = 0x1012;
  pub const SMOOTHING_GROUPS_CHUNK_ID: u32 = 0x1013;

  /// `flSGMask`: a face with no smoothing group of its own is flat rather than smoothed with all its neighbours.
  pub const SMOOTHING_GROUP_MASK_FLAG: u8 = 1 << 2;

  /// Reads the children of one numbered mesh chunk, as `CEditableMesh::LoadMesh` finds them.
  ///
  /// # Errors
  ///
  /// Returns an error where the SDK fails or asserts: a version other than [`Self::VERSION`], or a required chunk
  /// missing; and where a chunk holds more or less than its fields, a smoothing group list does not match the faces,
  /// or a count runs past its chunk.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut version: Option<u16> = None;
    let mut name: Option<String> = None;
    let mut bounds: Option<EditableMeshBounds> = None;
    let mut flags: Option<u8> = None;
    let mut options: Option<[u32; 2]> = None;
    let mut points: Option<Vec<Vector3d>> = None;
    let mut faces: Option<Vec<EditableFace>> = None;
    let mut smoothing_groups: Option<Vec<u32>> = None;
    let mut vmap_references: Option<Vec<EditableVMapReference>> = None;
    let mut surface_faces: Option<Vec<EditableSurfaceFaces>> = None;
    let mut vmaps: Option<Vec<EditableVMap>> = None;
    let mut extra: Vec<EditableExtraChunk> = Vec::new();

    let children: Vec<ChunkReader<D>> = reader.read_children()?;
    let chunk_order: Vec<u32> = children.iter().map(|chunk| chunk.id).collect();

    for mut chunk in children {
      match chunk.id {
        Self::VERSION_CHUNK_ID if version.is_none() => {
          version = Some(chunk.read_u16_payload::<T>()?);
        }
        Self::NAME_CHUNK_ID if name.is_none() => {
          name = Some(chunk.read_w1251_string_payload()?);
        }
        Self::BOUNDS_CHUNK_ID if bounds.is_none() => {
          bounds = Some(EditableChunks::read_whole(&mut chunk, "mesh bounds", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        Self::FLAGS_CHUNK_ID if flags.is_none() => {
          flags = Some(chunk.read_u8_payload()?);
        }
        Self::OPTIONS_CHUNK_ID if options.is_none() => {
          options = Some(EditableChunks::read_whole(&mut chunk, "mesh options", |chunk| {
            Ok([chunk.read_u32::<T>()?, chunk.read_u32::<T>()?])
          })?);
        }
        Self::POINTS_CHUNK_ID if points.is_none() => {
          points = Some(EditableChunks::read_whole(&mut chunk, "mesh points", |chunk| {
            EditableChunks::read_counted::<T, D, _>(chunk, 12, "mesh points", |chunk| chunk.read_xr::<T, _>())
          })?);
        }
        Self::FACES_CHUNK_ID if faces.is_none() => {
          faces = Some(EditableChunks::read_whole(&mut chunk, "mesh faces", |chunk| {
            EditableChunks::read_counted::<T, D, _>(chunk, EditableFace::SIZE, "mesh faces", |chunk| {
              chunk.read_xr::<T, _>()
            })
          })?);
        }
        Self::SMOOTHING_GROUPS_CHUNK_ID if smoothing_groups.is_none() => {
          smoothing_groups = Some(Self::read_smoothing_groups::<T, D>(&mut chunk)?);
        }
        Self::VMAP_REFERENCES_CHUNK_ID if vmap_references.is_none() => {
          vmap_references = Some(EditableChunks::read_whole(
            &mut chunk,
            "vertex map references",
            |chunk| {
              EditableChunks::read_counted::<T, D, _>(
                chunk,
                EditableVMapReference::MIN_SERIALIZED_SIZE,
                "vertex map references",
                |chunk| chunk.read_xr::<T, _>(),
              )
            },
          )?);
        }
        Self::SURFACE_FACES_CHUNK_ID if surface_faces.is_none() => {
          surface_faces = Some(EditableChunks::read_whole(&mut chunk, "surface faces", |chunk| {
            let count: u16 = chunk.read_u16::<T>()?;
            let mut lists: Vec<EditableSurfaceFaces> = chunk.new_bounded_vec(
              count.into(),
              EditableSurfaceFaces::MIN_SERIALIZED_SIZE,
              "surface face lists",
            )?;

            for _ in 0..count {
              lists.push(chunk.read_xr::<T, _>()?);
            }

            Ok(lists)
          })?);
        }
        Self::VMAPS_CHUNK_ID if vmaps.is_none() => {
          vmaps = Some(EditableChunks::read_whole(&mut chunk, "vertex maps", |chunk| {
            EditableChunks::read_counted::<T, D, _>(chunk, EditableVMap::MIN_SERIALIZED_SIZE, "vertex maps", |chunk| {
              chunk.read_xr::<T, _>()
            })
          })?);
        }
        _ => EditableChunks::keep(&mut chunk, &mut extra)?,
      }
    }

    EditableChunks::require_version(EditableChunks::require(version, "mesh version")?, Self::VERSION, "mesh")?;

    let faces: Vec<EditableFace> = EditableChunks::require(faces, "mesh faces")?;

    if let Some(groups) = &smoothing_groups
      && groups.len() != faces.len()
    {
      return Err(XrfError::new_parsing_error(format!(
        "Mesh carries {} smoothing groups for {} faces, the editor stores one per face",
        groups.len(),
        faces.len()
      )));
    }

    Ok(Self {
      name: EditableChunks::require(name, "mesh name")?,
      bounds: EditableChunks::require(bounds, "mesh bounds")?,
      flags: EditableChunks::require(flags, "mesh flags")?,
      options,
      points: EditableChunks::require(points, "mesh points")?,
      faces,
      smoothing_groups,
      vmap_references: EditableChunks::require(vmap_references, "vertex map references")?,
      surface_faces: EditableChunks::require(surface_faces, "surface faces")?,
      vmaps,
      extra,
      chunk_order,
    })
  }

  /// Writes the mesh's children in [`Self::chunk_order`], and any it does not list in the order
  /// `CEditableMesh::SaveMesh` writes them, the extra chunks last.
  ///
  /// # Errors
  ///
  /// Returns an error when a list is longer than its count field holds, the smoothing groups do not match the faces,
  /// or a vertex map's lists disagree on its entry count.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if let Some(groups) = &self.smoothing_groups
      && groups.len() != self.faces.len()
    {
      return Err(XrfError::new_invalid_error(format!(
        "Mesh '{}' holds {} smoothing groups for {} faces, the editor stores one per face",
        self.name,
        groups.len(),
        self.faces.len()
      )));
    }

    let mut sequence: EditableChunkSequence = EditableChunkSequence::new();

    sequence.push(Self::VERSION_CHUNK_ID, |writer| {
      Ok(writer.write_u16::<T>(Self::VERSION)?)
    })?;
    sequence.push(Self::NAME_CHUNK_ID, |writer| {
      writer.write_w1251_string(&self.name)?;

      Ok(())
    })?;
    sequence.push(Self::BOUNDS_CHUNK_ID, |writer| self.bounds.write::<T>(writer))?;
    sequence.push(Self::FLAGS_CHUNK_ID, |writer| Ok(writer.write_u8(self.flags)?))?;

    if let Some(options) = self.options {
      sequence.push(Self::OPTIONS_CHUNK_ID, |writer| {
        writer.write_u32::<T>(options[0])?;
        writer.write_u32::<T>(options[1])?;

        Ok(())
      })?;
    }

    sequence.push(Self::POINTS_CHUNK_ID, |writer| {
      EditableChunks::write_counted::<T, _>(writer, &self.points, "Mesh points")
    })?;
    sequence.push(Self::FACES_CHUNK_ID, |writer| {
      EditableChunks::write_counted::<T, _>(writer, &self.faces, "Mesh faces")
    })?;

    if let Some(groups) = &self.smoothing_groups {
      sequence.push(Self::SMOOTHING_GROUPS_CHUNK_ID, |writer| {
        for group in groups {
          writer.write_u32::<T>(*group)?;
        }

        Ok(())
      })?;
    }

    sequence.push(Self::VMAP_REFERENCES_CHUNK_ID, |writer| {
      EditableChunks::write_counted::<T, _>(writer, &self.vmap_references, "Vertex map references")
    })?;
    sequence.push(Self::SURFACE_FACES_CHUNK_ID, |writer| {
      writer.write_u16::<T>(to_format_size(self.surface_faces.len(), "Surface face lists")?)?;

      for list in &self.surface_faces {
        list.write::<T>(writer)?;
      }

      Ok(())
    })?;

    if let Some(vmaps) = &self.vmaps {
      sequence.push(Self::VMAPS_CHUNK_ID, |writer| {
        EditableChunks::write_counted::<T, _>(writer, vmaps, "Vertex maps")
      })?;
    }

    sequence.push_extra(&self.extra);

    sequence.write_into::<T>(writer, &self.chunk_order)
  }

  /// Whether a face with no smoothing group of its own is drawn flat.
  pub const fn is_smoothing_group_masked(&self) -> bool {
    self.flags & Self::SMOOTHING_GROUP_MASK_FLAG != 0
  }

  /// Reads the smoothing group chunk, which stores no count: `LoadMesh` reads a mask per face straight off it.
  fn read_smoothing_groups<T: ByteOrder, D: ChunkDataSource>(chunk: &mut ChunkReader<D>) -> XrfResult<Vec<u32>> {
    let size: u64 = chunk.read_bytes_remain();

    if !size.is_multiple_of(4) {
      return Err(XrfError::new_parsing_error(format!(
        "Unexpected smoothing group chunk of {size} bytes, which does not divide into 4 byte masks"
      )));
    }

    let mut groups: Vec<u32> = chunk.new_bounded_vec(size / 4, 4, "smoothing groups")?;

    for _ in 0..size / 4 {
      groups.push(chunk.read_u32::<T>()?);
    }

    Ok(groups)
  }
}
