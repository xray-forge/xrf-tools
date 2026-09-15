use serde::Serialize;
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::chunks::archive_chunk_node::ArchiveChunkNode;

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
  /// The container an entry is, or `None` for one that is not a container at all.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read. Bytes that read fine and are not a container are an
  /// answer rather than a failure, which is why this is an `Option` inside a `Result`.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Option<Self>> {
    let bytes: Vec<u8> = source.read_bytes(name)?;
    let size: u64 = bytes.len() as u64;
    let chunks: Vec<ArchiveChunkNode> = ArchiveChunkNode::of_bytes(bytes);

    if chunks.is_empty() {
      return Ok(None);
    }

    Ok(Some(Self {
      nodes: ArchiveChunkNode::count(&chunks),
      depth: ArchiveChunkNode::depth(&chunks),
      chunks,
      size,
    }))
  }
}
