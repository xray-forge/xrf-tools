use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkLayout, ChunkReadWrite, ChunkReader, ChunkSequence, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Vector3d};
use xrf_utils::to_format_size;

use crate::editable_chunks::EditableChunks;
use crate::{EditableFace, EditableSurfaceFaces, EditableVMap, EditableVMapReference};

/// One mesh of an editable object, `CEditableMesh` as `CEditableMesh::SaveMesh` stores it: points, triangles that
/// index them, the surfaces the triangles are made of, and the vertex maps their corners take UVs and weights from.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableMesh {
  pub name: String,
  pub bounds: Aabb,
  /// `flVisible`, `flLocked` and `flSGMask`, the last saying how a mesh without smoothing groups is smoothed.
  pub flags: u8,
  /// `st_MeshOptions`, two reserved numbers the editor zeroes. `None` where the chunk is absent, which the SDK
  /// accepts.
  pub options: Option<[u32; 2]>,
  pub points: Vec<Vector3d>,
  /// Whether the points chunk also carries the faces around each point, as editors older than `SaveMesh` saved it.
  /// `LoadMesh` never reads them; the writer derives them from [`Self::faces`], see [`Self::get_point_faces`], so an
  /// edited mesh cannot write back lists its faces no longer give.
  pub has_point_faces: bool,
  pub faces: Vec<EditableFace>,
  /// One smoothing group mask per face, `None` where the mesh stores none.
  pub smoothing_groups: Option<Vec<u32>>,
  /// Each face's corner normals, in corner order, which `blender-xray` writes beside the smoothing groups and the SDK
  /// never reads (see [`Self::CORNER_NORMALS_CHUNK_ID`]). `None` where the mesh stores none.
  pub corner_normals: Option<Vec<[Vector3d; 3]>>,
  pub vmap_references: Vec<EditableVMapReference>,
  pub surface_faces: Vec<EditableSurfaceFaces>,
  /// `None` where the mesh stores no `EMESH_CHUNK_VMAPS_2`.
  // todo: Read the older `EMESH_CHUNK_VMAPS_0` and `_1` layouts the SDK still loads, which stay extra chunks here.
  pub vmaps: Option<Vec<EditableVMap>>,
  /// How the mesh's chunks were laid out in its file.
  pub layout: ChunkLayout,
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
  /// `blender-xray`'s per-corner normals (`Chunks.Mesh.NORMALS`): three per face, no count, which its importer applies
  /// as custom split normals. `CEditableMesh::LoadMesh` reads no such chunk, so the SDK keeps smoothing by the groups.
  pub const CORNER_NORMALS_CHUNK_ID: u32 = 0x1014;

  /// `flVisible`: the editor draws the mesh, as every mesh it saves for export is.
  pub const VISIBLE_FLAG: u8 = 1 << 0;

  /// `flSGMask`: a face with no smoothing group of its own is flat rather than smoothed with all its neighbours.
  pub const SMOOTHING_GROUP_MASK_FLAG: u8 = 1 << 2;

  /// The smoothing word of a flat face, every edge hard, which `LoadMesh` gives each face of a mesh storing no words.
  pub const FLAT_FACE_SMOOTHING: u32 = u32::MAX;

  /// The smoothing word's bit for a back face, which numbers its edges the other way round and smooths only with
  /// other back faces.
  pub const BACK_FACE_SMOOTHING_FLAG: u32 = 1 << 3;

  /// Reads the children of one numbered mesh chunk, as `CEditableMesh::LoadMesh` finds them.
  ///
  /// # Errors
  ///
  /// Returns an error where the SDK fails or asserts: a version other than [`Self::VERSION`], or a required chunk
  /// missing; and where a chunk holds more or less than its fields, smoothing groups or corner normals do not match the
  /// faces, stored point face lists are not the ones the faces give, or a count runs past its chunk.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut version: Option<u16> = None;
    let mut name: Option<String> = None;
    let mut bounds: Option<Aabb> = None;
    let mut flags: Option<u8> = None;
    let mut options: Option<[u32; 2]> = None;
    let mut points: Option<MeshPoints> = None;
    let mut faces: Option<Vec<EditableFace>> = None;
    let mut smoothing_groups: Option<Vec<u32>> = None;
    let mut corner_normals: Option<Vec<[Vector3d; 3]>> = None;
    let mut vmap_references: Option<Vec<EditableVMapReference>> = None;
    let mut surface_faces: Option<Vec<EditableSurfaceFaces>> = None;
    let mut vmaps: Option<Vec<EditableVMap>> = None;

    let layout: ChunkLayout = ChunkLayout::read(reader, |chunk| {
      match chunk.id {
        Self::VERSION_CHUNK_ID if version.is_none() => version = Some(chunk.read_u16_payload::<T>()?),
        Self::NAME_CHUNK_ID if name.is_none() => name = Some(chunk.read_w1251_string_payload()?),
        Self::BOUNDS_CHUNK_ID if bounds.is_none() => {
          bounds = Some(EditableChunks::read_whole(chunk, "mesh bounds", |chunk| {
            chunk.read_xr::<T, _>()
          })?);
        }
        Self::FLAGS_CHUNK_ID if flags.is_none() => flags = Some(chunk.read_u8_payload()?),
        Self::OPTIONS_CHUNK_ID if options.is_none() => {
          options = Some(EditableChunks::read_whole(chunk, "mesh options", |chunk| {
            Ok([chunk.read_u32::<T>()?, chunk.read_u32::<T>()?])
          })?);
        }
        Self::POINTS_CHUNK_ID if points.is_none() => {
          points = Some(EditableChunks::read_whole(chunk, "mesh points", |chunk| {
            MeshPoints::read::<T, D>(chunk)
          })?);
        }
        Self::FACES_CHUNK_ID if faces.is_none() => {
          faces = Some(EditableChunks::read_whole(chunk, "mesh faces", |chunk| {
            EditableChunks::read_counted::<T, D, _>(chunk, EditableFace::SIZE, "mesh faces", |chunk| {
              chunk.read_xr::<T, _>()
            })
          })?);
        }
        Self::SMOOTHING_GROUPS_CHUNK_ID if smoothing_groups.is_none() => {
          smoothing_groups = Some(Self::read_smoothing_groups::<T, D>(chunk)?);
        }
        Self::CORNER_NORMALS_CHUNK_ID if corner_normals.is_none() => {
          corner_normals = Some(Self::read_corner_normals::<T, D>(chunk)?);
        }
        Self::VMAP_REFERENCES_CHUNK_ID if vmap_references.is_none() => {
          vmap_references = Some(EditableChunks::read_whole(chunk, "vertex map references", |chunk| {
            EditableChunks::read_counted::<T, D, _>(
              chunk,
              EditableVMapReference::MIN_SERIALIZED_SIZE,
              "vertex map references",
              |chunk| chunk.read_xr::<T, _>(),
            )
          })?);
        }
        Self::SURFACE_FACES_CHUNK_ID if surface_faces.is_none() => {
          surface_faces = Some(EditableChunks::read_whole(chunk, "surface faces", |chunk| {
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
          vmaps = Some(EditableChunks::read_whole(chunk, "vertex maps", |chunk| {
            EditableChunks::read_counted::<T, D, _>(chunk, EditableVMap::MIN_SERIALIZED_SIZE, "vertex maps", |chunk| {
              chunk.read_xr::<T, _>()
            })
          })?);
        }
        _ => return Ok(false),
      }

      Ok(true)
    })?;

    EditableChunks::require_version(EditableChunks::require(version, "mesh version")?, Self::VERSION, "mesh")?;

    let points: MeshPoints = EditableChunks::require(points, "mesh points")?;
    let mesh: Self = Self {
      name: EditableChunks::require(name, "mesh name")?,
      bounds: EditableChunks::require(bounds, "mesh bounds")?,
      flags: EditableChunks::require(flags, "mesh flags")?,
      options,
      points: points.points,
      has_point_faces: points.point_faces.is_some(),
      faces: EditableChunks::require(faces, "mesh faces")?,
      smoothing_groups,
      corner_normals,
      vmap_references: EditableChunks::require(vmap_references, "vertex map references")?,
      surface_faces: EditableChunks::require(surface_faces, "surface faces")?,
      vmaps,
      layout,
    };

    mesh.assert_per_face_lists_match()?;

    if let Some(stored) = points.point_faces
      && stored != mesh.get_point_faces()?
    {
      return Err(XrfError::new_parsing_error(format!(
        "Mesh '{}' stores point face lists its faces do not give, which writing them back could not reproduce",
        mesh.name
      )));
    }

    Ok(mesh)
  }

  /// Writes the mesh's children as [`Self::layout`] lays them out, and any it does not list in the order
  /// `CEditableMesh::SaveMesh` writes them.
  ///
  /// # Errors
  ///
  /// Returns an error when a list is longer than its count field holds, smoothing groups or corner normals do not match
  /// the faces, a face names a point the mesh does not have while its point faces are written, or a vertex map's lists
  /// disagree on its entry count.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.assert_per_face_lists_match()?;

    let mut sequence: ChunkSequence = ChunkSequence::new();

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
      EditableChunks::write_counted::<T, _>(writer, &self.points, "Mesh points")?;

      if self.has_point_faces {
        MeshPoints::write_faces::<T>(writer, &self.get_point_faces()?)?;
      }

      Ok(())
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

    if let Some(normals) = &self.corner_normals {
      sequence.push(Self::CORNER_NORMALS_CHUNK_ID, |writer| {
        for normal in normals.iter().flatten() {
          normal.write::<T>(writer)?;
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

    self.layout.write::<T>(writer, sequence)
  }

  /// Whether a face with no smoothing group of its own is drawn flat.
  pub const fn is_smoothing_group_masked(&self) -> bool {
    self.flags & Self::SMOOTHING_GROUP_MASK_FLAG != 0
  }

  /// The bit of a face's smoothing word that marks `edge`, from corner `edge` to corner `(edge + 1) % 3`, hard
  /// (`face_smoth_flags.h`): a back face stores edge `e` at bit `(4 - e) % 3` (`convert_edge_index`).
  pub const fn get_hard_edge_flag(smoothing: u32, edge: usize) -> u32 {
    let bit: usize = if smoothing & Self::BACK_FACE_SMOOTHING_FLAG != 0 {
      (4 - edge) % 3
    } else {
      edge
    };

    1 << bit
  }

  /// The faces around each point, by index, in face order: a face whose corners share a point lists it once per
  /// corner, as the editor's adjacency does.
  ///
  /// # Errors
  ///
  /// Returns an error when a face names a point the mesh does not have.
  pub fn get_point_faces(&self) -> XrfResult<Vec<Vec<u32>>> {
    let mut point_faces: Vec<Vec<u32>> = vec![Vec::new(); self.points.len()];

    for (index, face) in self.faces.iter().enumerate() {
      let face_index: u32 = to_format_size(index, "Mesh faces")?;

      for corner in &face.corners {
        point_faces
          .get_mut(corner.point as usize)
          .ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Mesh '{}' face {index} names point {} of {}",
              self.name,
              corner.point,
              self.points.len()
            ))
          })?
          .push(face_index);
      }
    }

    Ok(point_faces)
  }

  /// Refuses smoothing groups or corner normals that are not one entry per face, which their readers take straight off
  /// the chunk per face.
  fn assert_per_face_lists_match(&self) -> XrfResult {
    let lists = [
      ("smoothing groups", self.smoothing_groups.as_ref().map(Vec::len)),
      ("corner normal triples", self.corner_normals.as_ref().map(Vec::len)),
    ];

    for (label, count) in lists {
      if let Some(count) = count
        && count != self.faces.len()
      {
        return Err(XrfError::new_invalid_error(format!(
          "Mesh '{}' holds {count} {label} for {} faces, one per face",
          self.name,
          self.faces.len()
        )));
      }
    }

    Ok(())
  }

  /// Reads the corner normals chunk, which stores no count: three normals per face, read until the chunk ends.
  fn read_corner_normals<T: ByteOrder, D: ChunkDataSource>(
    chunk: &mut ChunkReader<D>,
  ) -> XrfResult<Vec<[Vector3d; 3]>> {
    const TRIPLE_SIZE: u64 = 36;

    let size: u64 = chunk.read_bytes_remain();

    if !size.is_multiple_of(TRIPLE_SIZE) {
      return Err(XrfError::new_parsing_error(format!(
        "Unexpected corner normals chunk of {size} bytes, which does not divide into {TRIPLE_SIZE} byte triples"
      )));
    }

    let mut normals: Vec<[Vector3d; 3]> =
      chunk.new_bounded_vec(size / TRIPLE_SIZE, TRIPLE_SIZE, "corner normal triples")?;

    for _ in 0..size / TRIPLE_SIZE {
      normals.push([
        chunk.read_xr::<T, _>()?,
        chunk.read_xr::<T, _>()?,
        chunk.read_xr::<T, _>()?,
      ]);
    }

    Ok(normals)
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

/// What the points chunk holds: the points, then in an older editor's file the faces around each.
struct MeshPoints {
  points: Vec<Vector3d>,
  point_faces: Option<Vec<Vec<u32>>>,
}

impl MeshPoints {
  /// A point's three floats.
  const POINT_SIZE: u64 = 12;

  fn read<T: ByteOrder, D: ChunkDataSource>(chunk: &mut ChunkReader<D>) -> XrfResult<Self> {
    let points: Vec<Vector3d> =
      EditableChunks::read_counted::<T, D, _>(chunk, Self::POINT_SIZE, "mesh points", |chunk| chunk.read_xr::<T, _>())?;

    if !chunk.has_data() {
      return Ok(Self {
        points,
        point_faces: None,
      });
    }

    // One count byte and that many face indices per point.
    let mut point_faces: Vec<Vec<u32>> = chunk.new_bounded_vec(points.len() as u64, 1, "point face lists")?;

    for _ in 0..points.len() {
      let count: u8 = chunk.read_u8()?;
      let mut faces: Vec<u32> = chunk.new_bounded_vec(count.into(), 4, "point faces")?;

      for _ in 0..count {
        faces.push(chunk.read_u32::<T>()?);
      }

      point_faces.push(faces);
    }

    Ok(Self {
      points,
      point_faces: Some(point_faces),
    })
  }

  /// Writes a count byte and the face indices for each point.
  fn write_faces<T: ByteOrder>(writer: &mut ChunkWriter, point_faces: &[Vec<u32>]) -> XrfResult {
    for faces in point_faces {
      writer.write_u8(to_format_size(faces.len(), "Point faces")?)?;

      for face in faces {
        writer.write_u32::<T>(*face)?;
      }
    }

    Ok(())
  }
}
