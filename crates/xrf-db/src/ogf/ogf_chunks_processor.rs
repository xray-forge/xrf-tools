use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};

use crate::data::ogf::ogf_geometry::OgfGeometry;
use crate::ogf::chunks::ogf_bones_chunk::OgfBonesChunk;
use crate::ogf::chunks::ogf_children_chunk::OgfChildrenChunk;
use crate::ogf::chunks::ogf_description_chunk::OgfDescriptionChunk;
use crate::ogf::chunks::ogf_header_chunk::OgfHeaderChunk;
use crate::ogf::chunks::ogf_ik_data_chunk::OgfIkDataChunk;
use crate::ogf::chunks::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::ogf::chunks::ogf_lods_chunk::OgfLodsChunk;
use crate::ogf::chunks::ogf_swi_data_chunk::OgfSwiDataChunk;
use crate::ogf::chunks::ogf_texture_chunk::OgfTextureChunk;
use crate::ogf::chunks::ogf_user_data_chunk::OgfUserDataChunk;
use crate::omf::chunks::omf_motions_chunk::OmfMotionsChunk;
use crate::omf::chunks::omf_parameters_chunk::OmfParametersChunk;

/// One chunk found while walking an ogf file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfChunkEntry {
  pub id: u32,
  /// 0 for a chunk of the root object, 1 for a chunk of a direct child, and so on.
  pub depth: usize,
  pub size: u64,
}

/// Walks the chunk tree of an ogf file without interpreting payloads.
///
/// Separate from [`crate::OgfFile`] on purpose: the file type stays a plain description of the chunks
/// the reader understands, while surveying what a file *actually* contains is a distinct job. It exists
/// to answer whether parsing is complete, rather than assuming it is because nothing crashed.
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
    OmfMotionsChunk::CHUNK_ID,
    OmfParametersChunk::CHUNK_ID,
  ];

  pub fn collect_chunks_from_path<T: ByteOrder, P: AsRef<Path>>(path: P) -> XrfResult<Vec<OgfChunkEntry>> {
    Self::collect_chunks::<T>(File::open(path.as_ref()).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "OGF file was not read: {}, error: {}",
        path.as_ref().display(),
        error
      ))
    })?)
  }

  pub fn collect_chunks<T: ByteOrder>(file: File) -> XrfResult<Vec<OgfChunkEntry>> {
    let mut chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_file(file)?.read_children()?;
    let mut entries: Vec<OgfChunkEntry> = Vec::new();

    Self::walk(&mut chunks, 0, &mut entries)?;

    Ok(entries)
  }

  /// Chunk ids present in the file that the reader does not understand, deduplicated and sorted.
  pub fn find_unknown_chunk_ids<T: ByteOrder, P: AsRef<Path>>(path: P) -> XrfResult<Vec<u32>> {
    let mut unknown: Vec<u32> = Self::collect_chunks_from_path::<T, _>(path)?
      .into_iter()
      .map(|it| it.id)
      .filter(|id| !Self::KNOWN_CHUNK_IDS.contains(id))
      .collect();

    unknown.sort_unstable();
    unknown.dedup();

    Ok(unknown)
  }

  /// Record every chunk, descending through the children container into each nested object.
  ///
  /// The immediate children of a children container are array slots numbered from zero, not chunk
  /// types, so they are stepped through rather than recorded.
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
