use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::ai::level_ai_header::LevelAiHeader;
use crate::ai::level_ai_nodes::LevelAiNodes;

/// Descriptor of the `level.ai` file used by xray game engine: the AI map, a header and the nodes it counts.
///
/// The plain reads stop at the header, which is what a check of the graph identity wants and what makes reading 56
/// bytes off a 34 MB file cheap. The `with_nodes` reads take the node run too, as [`LevelAiNodes`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAiFile {
  pub header: LevelAiHeader,
}

impl LevelAiFile {
  /// The file a compiled level's AI map is shipped as, in its directory, `LEVEL_GRAPH_NAME`.
  pub const FILE_NAME: &'static str = "level.ai";

  /// Read level AI-map file from provided path.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(Self::open(path.as_ref())?)
  }

  /// Read level AI-map file from file.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads the header from bytes already in hand, which is how an archived map arrives.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads the header from a chunk reader over any data source.
  ///
  /// The route an archived level file takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      header: reader.read_xr::<T, _>()?,
    })
  }

  /// Reads the header and every node it counts from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, its version is one the engine refuses, or the nodes are not
  /// exactly the rest of it.
  pub fn read_with_nodes_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<(Self, LevelAiNodes)> {
    Self::read_with_nodes_from_chunk::<T, _>(&mut ChunkReader::from_file(Self::open(path.as_ref())?)?)
  }

  /// Reads the header and every node it counts from bytes already in hand, which is how an archived map arrives.
  ///
  /// # Errors
  ///
  /// See [`Self::read_with_nodes_from_path`].
  pub fn read_with_nodes_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<(Self, LevelAiNodes)> {
    Self::read_with_nodes_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads the header and every node it counts from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// See [`Self::read_with_nodes_from_path`].
  pub fn read_with_nodes_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<(Self, LevelAiNodes)> {
    let file: Self = Self::read_from_chunk::<T, _>(reader)?;
    let nodes: LevelAiNodes = LevelAiNodes::read_from_chunk(reader, &file.header)?;

    Ok((file, nodes))
  }

  /// Writes the map back: the header, then the nodes.
  ///
  /// # Errors
  ///
  /// Returns an error when the nodes are not the count and version the header names, which would write a map the
  /// engine misreads.
  pub fn write<T: ByteOrder>(&self, nodes: &LevelAiNodes, writer: &mut ChunkWriter) -> XrfResult {
    if nodes.get_count() != self.header.count as usize || nodes.get_version() as u32 != self.header.version {
      return Err(XrfError::new_invalid_error(format!(
        "Level AI-map header names {} nodes of version {}, but {} nodes of version {} are written",
        self.header.count,
        self.header.version,
        nodes.get_count(),
        nodes.get_version() as u32
      )));
    }

    writer.write_xr::<T, _>(&self.header)?;
    nodes.write(writer)
  }

  fn open(path: &Path) -> XrfResult<File> {
    File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level AI-map file was not read: {}, error: {}",
        format_path(path),
        error
      ))
    })
  }
}
