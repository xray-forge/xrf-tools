use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::{format_path, to_format_size};

use crate::build_aimap::build_aimap_node::BuildAimapNode;
use crate::build_aimap::build_aimap_params::BuildAimapParams;

/// The AI map the level editor hands xrAI, `build.aimap` (`ESceneAIMapTool::Export`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAimapFile {
  /// `E_AIMAP_VERSION`, which decides how wide a link is.
  pub version: u16,
  /// The box around every node, which a node's height is packed across.
  pub bounds_min: Vector3d<f32>,
  pub bounds_max: Vector3d<f32>,
  pub params: BuildAimapParams,
  pub nodes: Vec<BuildAimapNode>,
}

impl BuildAimapFile {
  /// The file the level editor leaves its AI map as, in a level's directory.
  pub const FILE_NAME: &'static str = "build.aimap";

  /// The version the SDK writes, with three-byte links.
  pub const NARROW_LINKS_VERSION: u16 = 1;

  /// The version IX-Ray's editor writes unless asked for the old one, with four-byte links.
  pub const WIDE_LINKS_VERSION: u16 = 2;

  pub const VERSION_CHUNK_ID: u32 = 1;

  pub const BOX_CHUNK_ID: u32 = 2;

  pub const PARAMS_CHUNK_ID: u32 = 3;

  pub const NODES_CHUNK_ID: u32 = 4;

  /// Reads an editor AI map from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not an editor AI map this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Build AI map was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads an editor AI map from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not an editor AI map this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not an editor AI map this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk is absent, unknown or not wholly read, or the version is one xrAI does not load.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    if let Some(chunk) = chunks
      .iter()
      .find(|chunk| !(Self::VERSION_CHUNK_ID..=Self::NODES_CHUNK_ID).contains(&chunk.id))
    {
      return Err(XrfError::new_invalid_error(format!(
        "Build AI map carries chunk {}, which the editor never writes",
        chunk.id
      )));
    }

    let version: u16 = ChunkReader::find_required_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u16_payload::<T>()?;
    let has_wide_links: bool = Self::has_wide_links(version)?;

    let mut bounds: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::BOX_CHUNK_ID)?;
    let bounds_min: Vector3d<f32> = bounds.read_xr::<T, _>()?;
    let bounds_max: Vector3d<f32> = bounds.read_xr::<T, _>()?;

    bounds.assert_read("Expect all data to be read from build AI map box")?;

    let mut params_chunk: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::PARAMS_CHUNK_ID)?;
    let params: BuildAimapParams = params_chunk.read_xr::<T, _>()?;

    params_chunk.assert_read("Expect all data to be read from build AI map params")?;

    let mut nodes_chunk: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::NODES_CHUNK_ID)?;
    let count: u32 = nodes_chunk.read_u32::<T>()?;
    let mut nodes: Vec<BuildAimapNode> = nodes_chunk.new_bounded_vec(
      u64::from(count),
      BuildAimapNode::get_serialized_size(has_wide_links),
      "build AI map nodes",
    )?;

    for _ in 0..count {
      nodes.push(BuildAimapNode::read::<T, _>(&mut nodes_chunk, has_wide_links)?);
    }

    nodes_chunk.assert_read("Expect all data to be read from build AI map nodes")?;

    Ok(Self {
      version,
      bounds_min,
      bounds_max,
      params,
      nodes,
    })
  }

  /// Writes the map back in the layout the editor writes it in.
  ///
  /// # Errors
  ///
  /// Returns an error when the version is one this does not write, a link does not fit its version, or the writer
  /// refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let has_wide_links: bool = Self::has_wide_links(self.version)?;

    let mut version: ChunkWriter = ChunkWriter::new();

    version.write_u16::<T>(self.version)?;
    version.flush_chunk_into::<T>(&mut writer.buffer, Self::VERSION_CHUNK_ID)?;

    let mut bounds: ChunkWriter = ChunkWriter::new();

    bounds.write_xr::<T, _>(&self.bounds_min)?;
    bounds.write_xr::<T, _>(&self.bounds_max)?;
    bounds.flush_chunk_into::<T>(&mut writer.buffer, Self::BOX_CHUNK_ID)?;

    let mut params: ChunkWriter = ChunkWriter::new();

    params.write_xr::<T, _>(&self.params)?;
    params.flush_chunk_into::<T>(&mut writer.buffer, Self::PARAMS_CHUNK_ID)?;

    let mut nodes: ChunkWriter = ChunkWriter::new();

    nodes.write_u32::<T>(to_format_size(self.nodes.len(), "build AI map nodes")?)?;

    for node in &self.nodes {
      node.write::<T>(&mut nodes, has_wide_links)?;
    }

    nodes.flush_chunk_into::<T>(&mut writer.buffer, Self::NODES_CHUNK_ID)?;

    Ok(())
  }

  /// Where a node sits in the world, as the editor unpacks it (`ESceneAIMapTool::UnpackPosition`): its cell scaled by
  /// the patch size, and its height spread across the box.
  pub fn get_node_position(&self, node: &BuildAimapNode) -> Vector3d<f32> {
    Vector3d::new(
      f32::from(node.x) * self.params.patch_size,
      f32::from(node.y) / f32::from(u16::MAX) * (self.bounds_max.y - self.bounds_min.y) + self.bounds_min.y,
      f32::from(node.z) * self.params.patch_size,
    )
  }

  /// Whether a map of `version` stores four-byte links rather than three-byte ones.
  ///
  /// # Errors
  ///
  /// Returns an error for a version neither editor writes; xrAI asserts the version is at most its own.
  pub fn has_wide_links(version: u16) -> XrfResult<bool> {
    match version {
      Self::NARROW_LINKS_VERSION => Ok(false),
      Self::WIDE_LINKS_VERSION => Ok(true),
      unknown => Err(XrfError::new_invalid_error(format!(
        "Build AI map version {unknown} is not one an editor writes"
      ))),
    }
  }
}
