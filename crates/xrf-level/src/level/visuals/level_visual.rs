use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;
use xrf_math::Matrix4x4;
use xrf_ogf::{
  OgfChildrenLinkChunk, OgfFastpathChunk, OgfGeometryContainerChunk, OgfHeaderChunk, OgfLodDefinitionChunk,
  OgfSwiContainerChunk, OgfSwiDataChunk, OgfTextureChunk, OgfTreeDefinitionChunk,
};

/// One visual of a compiled level, as `fsL_VISUALS` stores it.
#[derive(Debug)]
pub struct LevelVisual {
  pub header: OgfHeaderChunk,
  /// The shader and texture the visual is dressed with, absent when the visual takes them from the level shader
  /// table by `header.shader_id` instead.
  pub texture: Option<OgfTextureChunk>,
  /// Where the visual's geometry sits in the buffers of `level.geom`.
  pub geometry: Option<OgfGeometryContainerChunk>,
  /// The same for `level.geomX`, with a progressive visual's windows over it, present only for a visual the compiler
  /// gave a fast path.
  pub fastpath: Option<OgfFastpathChunk>,
  /// Visuals this one composes, by their index in the run, for a hierarchy visual.
  pub children: Vec<u32>,
  /// Where a tree stands and how it is lit, absent for every visual that is not one.
  pub tree: Option<OgfTreeDefinitionChunk>,
  /// The impostor a `MT_LOD` visual draws in place of the trees it composes, seen from far enough away, or why it could
  /// not be read: a drawing detail, which costs the visual that part rather than the level.
  pub lod: Option<XrfResult<OgfLodDefinitionChunk>>,
  /// The windows of a progressive static mesh, stored with the visual, or why they could not be read.
  pub swi: Option<XrfResult<OgfSwiDataChunk>>,
  /// Which of `level.geom`'s window tables a progressive tree draws from, or why that could not be read.
  pub swi_container: Option<XrfResult<OgfSwiContainerChunk>>,
}

impl LevelVisual {
  /// Reads one visual from a reader positioned at its chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the visual carries no header, or when a chunk it does carry does not end where its record
  /// says it should; its impostor and slide windows are kept as read or as the reason they were not instead.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Ok(Self {
      header: ChunkReader::find_required_by_id(&chunks, OgfHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      texture: match ChunkReader::find_optional_by_id(&chunks, OgfTextureChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr::<T, _>()?),
        None => None,
      },
      children: match ChunkReader::find_optional_by_id(&chunks, OgfChildrenLinkChunk::CHUNK_ID) {
        Some(mut chunk) => chunk.read_xr::<T, OgfChildrenLinkChunk>()?.children,
        None => Vec::new(),
      },
      geometry: match ChunkReader::find_optional_by_id(&chunks, OgfGeometryContainerChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr::<T, _>()?),
        None => None,
      },
      fastpath: match ChunkReader::find_optional_by_id(&chunks, OgfFastpathChunk::CHUNK_ID) {
        Some(mut chunk) => Some(OgfFastpathChunk::read_from_chunk::<T, _>(&mut chunk)?),
        None => None,
      },
      tree: match ChunkReader::find_optional_by_id(&chunks, OgfTreeDefinitionChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr::<T, _>()?),
        None => None,
      },
      lod: ChunkReader::find_optional_by_id(&chunks, OgfLodDefinitionChunk::CHUNK_ID)
        .map(|mut chunk| chunk.read_xr::<T, _>()),
      swi: ChunkReader::find_optional_by_id(&chunks, OgfSwiDataChunk::CHUNK_ID)
        .map(|mut chunk| chunk.read_xr::<T, _>()),
      swi_container: ChunkReader::find_optional_by_id(&chunks, OgfSwiContainerChunk::CHUNK_ID)
        .map(|mut chunk| chunk.read_xr::<T, _>()),
    })
  }

  /// Whether the visual draws from the level's shared buffers at all.
  pub const fn is_drawable(&self) -> bool {
    self.geometry.is_some()
  }

  /// The transform placing this visual in the level, for one stored in its own space.
  pub const fn get_placement(&self) -> Option<&Matrix4x4> {
    match &self.tree {
      Some(definition) => Some(&definition.transform),
      None => None,
    }
  }
}
