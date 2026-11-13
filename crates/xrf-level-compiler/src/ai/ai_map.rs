use std::path::Path;

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_level::{LevelAiFile, LevelAiNodes};

/// A whole `level.ai`: its header and every node.
#[derive(Clone, Debug, PartialEq)]
pub struct AiMap {
  pub file: LevelAiFile,
  pub nodes: LevelAiNodes,
}

impl AiMap {
  /// Reads a `level.ai` with its nodes.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or is not an AI map the engine loads.
  pub fn read_from_path(path: &Path) -> XrfResult<Self> {
    let (file, nodes) = LevelAiFile::read_with_nodes_from_path::<XRayByteOrder, _>(&path)?;

    Ok(Self { file, nodes })
  }

  /// The map's bytes, the header then the nodes.
  ///
  /// # Errors
  ///
  /// Returns an error when the nodes are not the ones the header counts.
  pub fn get_bytes(&self) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    self.file.write::<XRayByteOrder>(&self.nodes, &mut writer)?;

    Ok(writer.buffer)
  }
}
