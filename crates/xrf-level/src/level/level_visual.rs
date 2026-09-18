use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, find_optional_chunk_by_id, find_required_chunk_by_id};
use xrf_error::XrfResult;

use xrf_ogf::{OgfChildrenLinkChunk, OgfGeometryContainerChunk, OgfHeaderChunk, OgfTextureChunk};

/// One visual of a compiled level, as `fsL_VISUALS` stores it.
#[derive(Debug)]
pub struct LevelVisual {
  pub header: OgfHeaderChunk,
  /// The shader and texture the visual is dressed with, absent when the visual takes them from the level shader
  /// table by `header.shader_id` instead.
  pub texture: Option<OgfTextureChunk>,
  /// Where the visual's geometry sits in the buffers of `level.geom`.
  pub geometry: Option<OgfGeometryContainerChunk>,
  /// The same for `level.geomX`, present only for a visual the compiler gave a fast path.
  pub fastpath: Option<OgfGeometryContainerChunk>,
  /// Visuals this one composes, by their index in the run, for a hierarchy visual.
  pub children: Vec<u32>,
}

impl LevelVisual {
  /// Reads one visual from a reader positioned at its chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the visual carries no header, or when a chunk it does carry does not end where its record
  /// says it should.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Ok(Self {
      header: find_required_chunk_by_id(&chunks, OgfHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      texture: match find_optional_chunk_by_id(&chunks, OgfTextureChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr::<T, _>()?),
        None => None,
      },
      children: match find_optional_chunk_by_id(&chunks, OgfChildrenLinkChunk::CHUNK_ID) {
        Some(mut chunk) => chunk.read_xr::<T, OgfChildrenLinkChunk>()?.children,
        None => Vec::new(),
      },
      geometry: match find_optional_chunk_by_id(&chunks, OgfGeometryContainerChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr::<T, _>()?),
        None => None,
      },
      fastpath: Self::read_fastpath::<T, D>(&chunks)?,
    })
  }

  /// Whether the visual draws from the level's shared buffers at all.
  pub const fn is_drawable(&self) -> bool {
    self.geometry.is_some()
  }

  /// `OGF_FASTPATH` wraps a geometry container rather than being one, so its record is one level deeper than the
  /// ordinary container's (`xray-16/src/Layers/xrRender/FVisual.cpp`).
  fn read_fastpath<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
  ) -> XrfResult<Option<OgfGeometryContainerChunk>> {
    let Some(mut fastpath) = find_optional_chunk_by_id(chunks, OgfGeometryContainerChunk::FASTPATH_CHUNK_ID) else {
      return Ok(None);
    };

    let nested: Vec<ChunkReader<D>> = fastpath.read_children()?;

    match find_optional_chunk_by_id(&nested, OgfGeometryContainerChunk::CHUNK_ID) {
      Some(mut chunk) => Ok(Some(chunk.read_xr::<T, _>()?)),
      None => Ok(None),
    }
  }
}
