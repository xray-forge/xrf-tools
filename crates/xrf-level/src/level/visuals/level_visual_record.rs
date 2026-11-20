use byteorder::ByteOrder;
use xrf_chunk::{ChunkReadWrite, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ogf::{
  OgfChildrenLinkChunk, OgfFastpathChunk, OgfGeometryContainerChunk, OgfHeaderChunk, OgfLodDefinitionChunk,
  OgfSwiContainerChunk, OgfSwiDataChunk, OgfTextureChunk, OgfTreeDefinitionChunk,
};

use crate::level::visuals::level_visual::LevelVisual;

/// One visual of a compiled level held whole, every chunk it carries read, so it writes back as xrLC wrote it.
///
/// A visual is written in the order its kind's `Save` writes it (`xrLC/OGF_Face_Save.cpp`, `OGF_Face.cpp`):
/// - a static visual (`OGF::Save_Normal_PM`): geometry, its windows, its fast path, then the header;
/// - a tree (`OGF_Reference::Save`): geometry, placement, the header, then its window table;
/// - a hierarchy or LOD node (`OGF_Node::Save`, `OGF_LOD::Save`): the header, its children, then the impostor.
///
/// A texture chunk, which xrLC never writes, goes after the header.
#[derive(Debug)]
pub struct LevelVisualRecord {
  pub header: OgfHeaderChunk,
  pub texture: Option<OgfTextureChunk>,
  pub geometry: Option<OgfGeometryContainerChunk>,
  pub swi: Option<OgfSwiDataChunk>,
  pub fastpath: Option<OgfFastpathChunk>,
  pub tree: Option<OgfTreeDefinitionChunk>,
  pub swi_container: Option<OgfSwiContainerChunk>,
  pub children: Option<Vec<u32>>,
  pub lod: Option<OgfLodDefinitionChunk>,
}

impl LevelVisualRecord {
  /// A visual read whole; a children chunk is taken as present where it lists any.
  ///
  /// # Errors
  ///
  /// Returns an error for a chunk of it that was not read.
  pub fn of(visual: LevelVisual) -> XrfResult<Self> {
    let has_children: bool = !visual.children.is_empty();
    let unread =
      |what: &str, error: XrfError| XrfError::new_invalid_error(format!("A visual's {what} was not read: {error}"));

    Ok(Self {
      header: visual.header,
      texture: visual.texture,
      geometry: visual.geometry,
      swi: visual.swi.transpose().map_err(|error| unread("slide windows", error))?,
      fastpath: visual.fastpath,
      tree: visual.tree,
      swi_container: visual
        .swi_container
        .transpose()
        .map_err(|error| unread("slide window table", error))?,
      children: has_children.then_some(visual.children),
      lod: visual.lod.transpose().map_err(|error| unread("impostor", error))?,
    })
  }

  /// Writes the visual's chunks into `writer`, which the caller wraps in the visual's own chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer fails.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if self.tree.is_some() {
      Self::put::<T, _>(writer, OgfGeometryContainerChunk::CHUNK_ID, self.geometry.as_ref())?;
      Self::put::<T, _>(writer, OgfTreeDefinitionChunk::CHUNK_ID, self.tree.as_ref())?;
      Self::put::<T, _>(writer, OgfHeaderChunk::CHUNK_ID, Some(&self.header))?;
      Self::put::<T, _>(writer, OgfTextureChunk::CHUNK_ID, self.texture.as_ref())?;
      Self::put::<T, _>(writer, OgfSwiContainerChunk::CHUNK_ID, self.swi_container.as_ref())?;
    } else if self.geometry.is_some() {
      Self::put::<T, _>(writer, OgfGeometryContainerChunk::CHUNK_ID, self.geometry.as_ref())?;
      Self::put::<T, _>(writer, OgfSwiDataChunk::CHUNK_ID, self.swi.as_ref())?;

      if let Some(fastpath) = &self.fastpath {
        let mut chunk: ChunkWriter = ChunkWriter::new();

        fastpath.write::<T>(&mut chunk)?;
        chunk.flush_chunk_into::<T>(&mut writer.buffer, OgfFastpathChunk::CHUNK_ID)?;
      }

      Self::put::<T, _>(writer, OgfHeaderChunk::CHUNK_ID, Some(&self.header))?;
      Self::put::<T, _>(writer, OgfTextureChunk::CHUNK_ID, self.texture.as_ref())?;
    } else {
      Self::put::<T, _>(writer, OgfHeaderChunk::CHUNK_ID, Some(&self.header))?;
      Self::put::<T, _>(writer, OgfTextureChunk::CHUNK_ID, self.texture.as_ref())?;

      if let Some(children) = &self.children {
        Self::put::<T, _>(
          writer,
          OgfChildrenLinkChunk::CHUNK_ID,
          Some(&OgfChildrenLinkChunk {
            children: children.clone(),
          }),
        )?;
      }

      Self::put::<T, _>(writer, OgfLodDefinitionChunk::CHUNK_ID, self.lod.as_ref())?;
    }

    Ok(())
  }

  /// Writes `record` as chunk `id` where it is present.
  fn put<B: ByteOrder, R: ChunkReadWrite>(writer: &mut ChunkWriter, id: u32, record: Option<&R>) -> XrfResult {
    if let Some(record) = record {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      chunk.write_xr::<B, _>(record)?;
      chunk.flush_chunk_into::<B>(&mut writer.buffer, id)?;
    }

    Ok(())
  }
}
