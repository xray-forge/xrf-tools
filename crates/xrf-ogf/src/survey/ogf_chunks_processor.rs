use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, InMemoryChunkDataSource};
use xrf_error::XrfResult;
use xrf_skeleton::{SkeletonMotionParametersChunk, SkeletonMotionsChunk};

use crate::chunks::ogf_bones_chunk::OgfBonesChunk;
use crate::chunks::ogf_children_chunk::OgfChildrenChunk;
use crate::chunks::ogf_description_chunk::OgfDescriptionChunk;
use crate::chunks::ogf_header_chunk::OgfHeaderChunk;
use crate::chunks::ogf_ik_data_chunk::OgfIkDataChunk;
use crate::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::chunks::ogf_lods_chunk::OgfLodsChunk;
use crate::chunks::ogf_swi_data_chunk::OgfSwiDataChunk;
use crate::chunks::ogf_texture_chunk::OgfTextureChunk;
use crate::chunks::ogf_user_data_chunk::OgfUserDataChunk;
use crate::data::ogf_geometry::OgfGeometry;
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
  ];

  /// The chunk tree of the visual `bytes` hold, for a caller that also reads the visual from the same bytes.
  pub fn collect_chunks_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<OgfChunkSurvey> {
    let (mut chunks, residue) =
      OgfResidue::read_root_chunks::<T, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_vec(bytes)?)?;
    let mut entries: Vec<OgfChunkEntry> = Vec::new();

    Self::walk(&mut chunks, 0, &mut entries)?;

    Ok(OgfChunkSurvey { entries, residue })
  }

  /// Chunk ids present in the visual `bytes` hold that the reader does not understand, deduplicated and sorted.
  pub fn find_unknown_chunk_ids_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Vec<u32>> {
    let mut unknown: Vec<u32> = Self::collect_chunks_from_bytes::<T>(bytes)?
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
