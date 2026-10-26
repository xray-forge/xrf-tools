use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;
use xrf_skeleton::{SkeletonMotionParametersChunk, SkeletonMotionsChunk};

use crate::chunks::level::ogf_children_link_chunk::OgfChildrenLinkChunk;
use crate::chunks::level::ogf_fastpath_chunk::OgfFastpathChunk;
use crate::chunks::level::ogf_geometry_container_chunk::OgfGeometryContainerChunk;
use crate::chunks::level::ogf_lod_definition_chunk::OgfLodDefinitionChunk;
use crate::chunks::level::ogf_swi_container_chunk::OgfSwiContainerChunk;
use crate::chunks::level::ogf_tree_definition_chunk::OgfTreeDefinitionChunk;
use crate::chunks::mesh::ogf_children_chunk::OgfChildrenChunk;
use crate::chunks::mesh::ogf_collision_indices_chunk::OgfCollisionIndicesChunk;
use crate::chunks::mesh::ogf_collision_vertices_chunk::OgfCollisionVerticesChunk;
use crate::chunks::mesh::ogf_swi_data_chunk::OgfSwiDataChunk;
use crate::chunks::skeleton::ogf_bones_chunk::OgfBonesChunk;
use crate::chunks::skeleton::ogf_ik_data_chunk::OgfIkDataChunk;
use crate::chunks::skeleton::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::chunks::skeleton::ogf_lods_chunk::OgfLodsChunk;
use crate::chunks::skeleton::ogf_user_data_chunk::OgfUserDataChunk;
use crate::chunks::visual::ogf_description_chunk::OgfDescriptionChunk;
use crate::chunks::visual::ogf_header_chunk::OgfHeaderChunk;
use crate::chunks::visual::ogf_texture_chunk::OgfTextureChunk;
use crate::data::geometry::ogf_geometry::OgfGeometry;
use crate::residue::OgfResidue;
use crate::survey::ogf_chunk_entry::OgfChunkEntry;
use crate::survey::ogf_chunk_survey::OgfChunkSurvey;

/// Walks the chunk tree of an ogf file without interpreting payloads.
pub struct OgfChunksProcessor {}

impl OgfChunksProcessor {
  /// Chunk ids [`crate::OgfFile`] currently reads.
  pub const KNOWN_CHUNK_IDS: &'static [u32] = &[
    OgfHeaderChunk::CHUNK_ID,
    OgfTextureChunk::CHUNK_ID,
    OgfGeometry::VERTICES_CHUNK_ID,
    OgfGeometry::INDICES_CHUNK_ID,
    OgfChildrenChunk::CHUNK_ID,
    OgfBonesChunk::CHUNK_ID,
    OgfDescriptionChunk::CHUNK_ID,
    OgfKinematicsChunk::CHUNK_ID,
    OgfKinematicsChunk::CHUNK_ID_OLD,
    OgfUserDataChunk::CHUNK_ID,
    OgfLodsChunk::CHUNK_ID,
    OgfIkDataChunk::CHUNK_ID,
    OgfSwiDataChunk::CHUNK_ID,
    SkeletonMotionsChunk::CHUNK_ID,
    SkeletonMotionParametersChunk::CHUNK_ID,
    OgfChildrenLinkChunk::CHUNK_ID,
    OgfLodDefinitionChunk::CHUNK_ID,
    OgfTreeDefinitionChunk::CHUNK_ID,
    OgfSwiContainerChunk::CHUNK_ID,
    OgfGeometryContainerChunk::CHUNK_ID,
    OgfFastpathChunk::CHUNK_ID,
    OgfCollisionVerticesChunk::CHUNK_ID,
    OgfCollisionIndicesChunk::CHUNK_ID,
  ];

  /// The chunk tree of the visual `reader` opens, read from its start.
  ///
  /// Rewinds `reader` first, so a caller that has just read the visual through [`crate::OgfFile::read_from_chunk`]
  /// surveys the same bytes without reading or copying them again.
  pub fn collect_chunks<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<OgfChunkSurvey> {
    reader.reset_pos()?;

    let (mut chunks, residue) = OgfResidue::read_root_chunks::<T, _>(reader)?;
    let mut entries: Vec<OgfChunkEntry> = Vec::new();

    Self::walk(&mut chunks, 0, &mut entries)?;

    Ok(OgfChunkSurvey { entries, residue })
  }

  /// Chunk ids present in the visual `reader` opens that the reader does not understand, deduplicated and sorted.
  pub fn find_unknown_chunk_ids<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<u32>> {
    let mut unknown: Vec<u32> = Self::collect_chunks::<T, D>(reader)?
      .entries
      .into_iter()
      .map(|it| it.id)
      .filter(|id| !Self::KNOWN_CHUNK_IDS.contains(id))
      .collect();

    unknown.sort_unstable();
    unknown.dedup();

    Ok(unknown)
  }

  /// Record every chunk, descending through the children container into each nested object.
  fn walk<D: ChunkDataSource>(
    chunks: &mut [ChunkReader<D>],
    depth: usize,
    entries: &mut Vec<OgfChunkEntry>,
  ) -> XrfResult {
    for chunk in chunks {
      entries.push(OgfChunkEntry {
        id: chunk.id,
        depth,
        size: chunk.read_bytes_len(),
      });

      if chunk.id != OgfChildrenChunk::CHUNK_ID {
        continue;
      }

      chunk.reset_pos()?;

      for mut slot in chunk.read_children()? {
        Self::walk(&mut slot.read_children()?, depth + 1, entries)?;
      }
    }

    Ok(())
  }
}
