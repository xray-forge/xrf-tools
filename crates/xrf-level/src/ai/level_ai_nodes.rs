use std::io::Write;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::ai::level_ai_header::LevelAiHeader;
use crate::ai::level_ai_node::LevelAiNode;
use crate::ai::level_ai_version::LevelAiVersion;

/// Every node of a `level.ai`, kept packed as the file stores them and decoded one at a time.
///
/// A map holds up to one and a half million nodes (Jupiter's is 34 MB) and most questions about it are about a few of
/// them or a total, so the run is not unpacked into records. Keeping the bytes also keeps the light bits older
/// compilers left behind the links, which makes [`Self::write`] give back the file it read.
///
/// The decoders take the byte order because the bytes are held as the file stores them.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelAiNodes {
  version: LevelAiVersion,
  bytes: Vec<u8>,
}

impl LevelAiNodes {
  /// Reads the nodes a header counts, from just after the header, in the layout its version names.
  ///
  /// # Errors
  ///
  /// Returns an error when the version is one the engine refuses, or the payload is not exactly the nodes counted.
  pub fn read_from_chunk<D: ChunkDataSource>(reader: &mut ChunkReader<D>, header: &LevelAiHeader) -> XrfResult<Self> {
    let version: LevelAiVersion = LevelAiVersion::from_number(header.version)?;
    let size: u64 = u64::from(header.count) * version.get_node_size();

    if size != reader.read_bytes_remain() {
      return Err(XrfError::new_invalid_error(format!(
        "Level AI-map counts {} nodes of {} bytes, but {} bytes follow its header",
        header.count,
        version.get_node_size(),
        reader.read_bytes_remain()
      )));
    }

    Ok(Self {
      version,
      bytes: reader.read_bytes(size as usize)?,
    })
  }

  /// The layout the nodes are packed in.
  pub fn get_version(&self) -> LevelAiVersion {
    self.version
  }

  /// Nodes held.
  pub fn get_count(&self) -> usize {
    self.bytes.len() / self.version.get_node_size() as usize
  }

  /// One node, by index, or none past the end.
  pub fn get_node<T: ByteOrder>(&self, index: usize) -> Option<LevelAiNode> {
    let size: usize = self.version.get_node_size() as usize;
    let start: usize = index.checked_mul(size)?;

    self
      .bytes
      .get(start..start.checked_add(size)?)
      .map(|bytes| LevelAiNode::decode::<T>(bytes, &self.version.get_node_layout()))
  }

  /// Every node, in order.
  pub fn iter_nodes<T: ByteOrder>(&self) -> impl Iterator<Item = LevelAiNode> + '_ {
    self
      .bytes
      .chunks_exact(self.version.get_node_size() as usize)
      .map(|bytes| LevelAiNode::decode::<T>(bytes, &self.version.get_node_layout()))
  }

  /// Writes the nodes back as they were read.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    Ok(writer.write_all(&self.bytes)?)
  }
}
