use serde::Serialize;
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};

use crate::describe::chunks::archive_chunk_node::ArchiveChunkNode;
use crate::describe::source::ArchiveEntryContainer;
use crate::describe::source::ArchiveEntryReader;

/// The container a file is, for a file nothing here reads.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveChunksDescription {
  /// Chunks in the order the file frames them.
  pub chunks: Vec<ArchiveChunkNode>,
  /// Every chunk of the tree, the nested ones included.
  pub nodes: usize,
  /// How far the deepest branch descends.
  pub depth: usize,
  /// Bytes the file occupies unpacked, which the chunks account for in full.
  pub size: u64,
}

impl ArchiveChunksDescription {
  /// The container an opened entry is, or `None` for one that is not a container at all, which is an answer rather
  /// than a failure.
  pub fn of_entry(entry: &mut ArchiveEntryReader) -> Option<Self> {
    entry.read_container::<Self>().ok()
  }
}

impl ArchiveEntryContainer for ArchiveChunksDescription {
  fn read_container<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let size: u64 = reader.size;
    let chunks: Vec<ArchiveChunkNode> = ArchiveChunkNode::of_reader(reader);

    if chunks.is_empty() {
      return Err(XrfError::new_invalid_error("The entry is not a chunked container"));
    }

    Ok(Self {
      nodes: ArchiveChunkNode::count(&chunks),
      depth: ArchiveChunkNode::depth(&chunks),
      chunks,
      size,
    })
  }
}
