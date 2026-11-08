use byteorder::ByteOrder;
use xrf_chunk::ChunkWriter;
use xrf_editable_object::{EditableObjectFile, EditableSurface};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Aabb, Vector3d};
use xrf_ogf::{
  OgfChildrenChunk, OgfCollisionFace, OgfCollisionIndicesChunk, OgfCollisionVerticesChunk, OgfDescriptionChunk, OgfFvf,
  OgfGeometry, OgfHeaderChunk, OgfModelType, OgfStaticVertex, OgfTextureChunk,
};

use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::ogf_export_chunks::OgfExportChunks;
use crate::object::compile::rigid::rigid_geometry::RigidGeometry;
use crate::object::compile::rigid::rigid_split::RigidSplit;

/// Writes a static object's OGF as `CExportObjectOGF::Export` does: one split alone as the visual itself
/// (`MT_NORMAL`), several as a hierarchy (`MT_HIERRARHY`) carrying the description and a child per split; then the
/// collision mesh of every split's vertices and faces, which no engine loader reads.
pub(crate) struct RigidOgfWriter;

impl RigidOgfWriter {
  /// The OGF's bytes.
  ///
  /// # Errors
  ///
  /// Returns an error for geometry without a face, which no visual draws, and where a chunk does not write, such as a
  /// description time past the 32-bit times it was authored with.
  //
  // todo: write a progressive object's sliding windows (`MT_PROGRESSIVE`, `OGF_SWIDATA` from `VIPM_Convert`); it is
  // written at full detail, as `MT_NORMAL`.
  pub(crate) fn write<T: ByteOrder>(
    object: &EditableObjectFile,
    geometry: &RigidGeometry,
    source: &ObjectCompileSource,
  ) -> XrfResult<Vec<u8>> {
    let mut file: ChunkWriter = ChunkWriter::new();

    match geometry.splits.as_slice() {
      [] => {
        return Err(XrfError::new_invalid_error(
          "A static object keeps no face to export once degenerate ones are dropped",
        ));
      }
      [split] => Self::write_part::<T>(&mut file, split)?,
      splits => {
        OgfExportChunks::write_chunk::<T, _>(
          &mut file,
          OgfHeaderChunk::CHUNK_ID,
          &OgfExportChunks::header(OgfModelType::Hierarchy, &Self::get_bounds(splits)),
        )?;
        OgfExportChunks::write_chunk::<T, _>(
          &mut file,
          OgfDescriptionChunk::CHUNK_ID,
          &OgfExportChunks::description(object, source),
        )?;

        let children: Vec<Vec<u8>> = splits
          .iter()
          .map(|split| {
            let mut child: ChunkWriter = ChunkWriter::new();

            Self::write_part::<T>(&mut child, split)?;
            child.flush_raw_into_buffer()
          })
          .collect::<XrfResult<_>>()?;

        OgfChildrenChunk::write_nested::<T>(&mut file, &children)?;
      }
    }

    Self::write_collision::<T>(&mut file, &geometry.splits)?;

    file.flush_raw_into_buffer()
  }

  /// A split's visual, `SSplit::SavePart`: its header, texture, vertices and indices.
  fn write_part<T: ByteOrder>(writer: &mut ChunkWriter, split: &RigidSplit) -> XrfResult {
    let format: OgfFvf = OgfFvf::from_raw(EditableSurface::POSITION_NORMAL_UV_FVF)
      .expect("the export's vertex format is a flexible vertex format");

    OgfExportChunks::write_chunk::<T, _>(
      writer,
      OgfHeaderChunk::CHUNK_ID,
      &OgfExportChunks::header(OgfModelType::Normal, &Self::get_bounds(std::slice::from_ref(split))),
    )?;
    OgfExportChunks::write_chunk::<T, _>(
      writer,
      OgfTextureChunk::CHUNK_ID,
      &OgfTextureChunk {
        texture_name: split.texture.clone(),
        shader_name: split.shader.clone(),
        trailing: Vec::new(),
      },
    )?;

    let vertices: Vec<OgfStaticVertex> = split
      .vertices
      .iter()
      .map(|vertex| OgfStaticVertex {
        position: vertex.position,
        normal: Some(vertex.normal),
        point_size: None,
        diffuse: None,
        specular: None,
        texture_coordinates: vec![vec![vertex.texture_coordinate.x, vertex.texture_coordinate.y]],
      })
      .collect();
    let indices: Vec<u16> = split.faces.iter().flatten().copied().collect();

    OgfGeometry::write_static::<T>(writer, &format, &vertices, &indices)
  }

  /// The collision mesh: every split's vertex positions in turn, and their faces, numbered past the splits before.
  fn write_collision<T: ByteOrder>(writer: &mut ChunkWriter, splits: &[RigidSplit]) -> XrfResult {
    let mut vertices: Vec<Vector3d> = Vec::new();
    let mut faces: Vec<OgfCollisionFace> = Vec::new();

    for split in splits {
      let offset: u32 = u32::try_from(vertices.len())
        .map_err(|_| XrfError::new_invalid_error("A static object holds more vertices than a u32 numbers"))?;

      faces.extend(split.faces.iter().map(|face| OgfCollisionFace {
        vertices: face.map(|index| offset + u32::from(index)),
      }));
      vertices.extend(split.vertices.iter().map(|vertex| vertex.position));
    }

    OgfExportChunks::write_chunk::<T, _>(
      writer,
      OgfCollisionVerticesChunk::CHUNK_ID,
      &OgfCollisionVerticesChunk { vertices },
    )?;
    OgfExportChunks::write_chunk::<T, _>(
      writer,
      OgfCollisionIndicesChunk::CHUNK_ID,
      &OgfCollisionIndicesChunk { faces },
    )
  }

  fn get_bounds(splits: &[RigidSplit]) -> Aabb {
    OgfExportChunks::get_bounds(
      splits
        .iter()
        .flat_map(|split| &split.vertices)
        .map(|vertex| &vertex.position),
    )
  }
}
