use std::fs::File;
use std::io::Read;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::format_path;

/// `hdrNODES` in c++ codebase, stored raw at the very start of the `level.ai` file.
///
/// Unlike most xray formats `level.ai` is not chunked - `CLevelGraph` opens the file and casts its
/// pointer straight onto this structure, so the header occupies the first 56 bytes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAiHeader {
  pub version: u32,
  pub count: u32,
  pub size: f32,
  pub size_y: f32,
  pub aabb_min: Vector3d<f32>,
  pub aabb_max: Vector3d<f32>,
  pub guid: Uuid,
}

impl LevelAiHeader {
  /// Byte size of the header as laid out by the engine.
  pub const SIZE: u64 = 56;

  /// `XRAI_VERSION_BORSHT_BIG`, the first version whose nodes pack their place into six bytes rather than five.
  pub const WIDE_POSITION_VERSION: u32 = 12;

  /// `EPS_L`, which the engine pads the grid's row length with (`xrCore/math_constants.h`).
  const ROW_EPSILON: f32 = 0.001;

  /// Nodes a row of the grid holds, which is how a node's packed place is unpacked (`CLevelGraph::Initialize`).
  pub fn get_row_length(&self) -> u32 {
    ((self.aabb_max.z - self.aabb_min.z) / self.size + Self::ROW_EPSILON + 1.5).floor() as u32
  }
}

impl ChunkReadWrite for LevelAiHeader {
  /// Read level AI-map header from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      version: reader.read_u32::<T>()?,
      count: reader.read_u32::<T>()?,
      size: reader.read_f32::<T>()?,
      size_y: reader.read_f32::<T>()?,
      aabb_min: reader.read_xr::<T, _>()?,
      aabb_max: reader.read_xr::<T, _>()?,
      guid: Uuid::from_u128(reader.read_u128::<T>()?),
    })
  }

  /// Write level AI-map header into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.version)?;
    writer.write_u32::<T>(self.count)?;
    writer.write_f32::<T>(self.size)?;
    writer.write_f32::<T>(self.size_y)?;
    writer.write_xr::<T, _>(&self.aabb_min)?;
    writer.write_xr::<T, _>(&self.aabb_max)?;
    writer.write_u128::<T>(self.guid.as_u128())?;

    Ok(())
  }
}

/// Descriptor of the `level.ai` file used by xray game engine.
///
/// Only the header is read. Node payload is not parsed - it is the single largest file in a level
/// bundle and nothing in it can be validated without reimplementing the node compressor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAiFile {
  pub header: LevelAiHeader,
  // todo: Content.
}

impl LevelAiFile {
  /// Read level AI-map file from provided path.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level AI-map file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  /// Read level AI-map file from file.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Finds the node standing nearest a point across the ground, the lower of two at one place, reading nothing of
  /// the nodes but their places: every node ends in it, `NodePosition4` or `NodePosition12` by version.
  ///
  /// # Errors
  ///
  /// Returns an error when the header cannot be read or the nodes do not divide into the count it declares.
  pub fn find_nearest_node<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    x: f32,
    z: f32,
  ) -> XrfResult<Option<Vector3d<f32>>> {
    let header: LevelAiHeader = reader.read_xr::<T, _>()?;
    let remaining: u64 = reader.read_bytes_remain();

    if header.count == 0 {
      return Ok(None);
    }

    let position_size: u64 = if header.version >= LevelAiHeader::WIDE_POSITION_VERSION {
      6
    } else {
      5
    };
    let stride: u64 = remaining / u64::from(header.count);

    if !remaining.is_multiple_of(u64::from(header.count)) || stride < position_size {
      return Err(XrfError::new_invalid_error(format!(
        "Level AI-map nodes are {remaining} bytes, which do not divide into {} nodes",
        header.count
      )));
    }

    let mut nodes: Vec<u8> = vec![0; remaining as usize];

    reader.read_exact(&mut nodes)?;

    let row_length: u32 = header.get_row_length().max(1);
    let mut nearest: Option<(f32, Vector3d<f32>)> = None;

    for node in nodes.chunks_exact(stride as usize) {
      let place: &[u8] = &node[node.len() - position_size as usize..];
      let (xz, y): (u32, u16) = if position_size == 6 {
        (T::read_u32(&place[..4]), T::read_u16(&place[4..]))
      } else {
        (T::read_u24(&place[..3]), T::read_u16(&place[3..]))
      };
      let position: Vector3d<f32> = Vector3d::new(
        (xz / row_length) as f32 * header.size + header.aabb_min.x,
        f32::from(y) / f32::from(u16::MAX) * header.size_y + header.aabb_min.y,
        (xz % row_length) as f32 * header.size + header.aabb_min.z,
      );
      let distance: f32 = (position.x - x).powi(2) + (position.z - z).powi(2);

      if nearest
        .as_ref()
        .is_none_or(|(best, at)| distance < *best || (distance == *best && position.y < at.y))
      {
        nearest = Some((distance, position));
      }
    }

    Ok(nearest.map(|(_, position)| position))
  }

  /// Reads the header from a chunk reader over any data source.
  ///
  /// The route an archived level file takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      header: reader.read_xr::<T, _>()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use uuid::uuid;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_file,
    overwrite_generated_test_resource_as_file,
  };

  use crate::ai::level_ai_file::{LevelAiFile, LevelAiHeader};

  fn sample() -> LevelAiHeader {
    LevelAiHeader {
      version: 10,
      count: 1_356_204,
      size: 0.7,
      size_y: 0.2,
      aabb_min: Vector3d::new(-600.0, -20.5, -615.0),
      aabb_max: Vector3d::new(600.0, 80.25, 585.0),
      guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.ai");
    let mut writer: ChunkWriter = ChunkWriter::new();
    let original: LevelAiHeader = sample();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written() as u64, LevelAiHeader::SIZE);

    writer.flush_raw_into(&mut overwrite_generated_test_resource_as_file(
      &build_relative_test_sample_file_path(file!(), &filename),
    )?)?;

    let read: LevelAiFile = LevelAiFile::read_from_file::<XRayByteOrder>(open_generated_test_resource_as_file(
      &build_relative_test_sample_file_path(file!(), &filename),
    )?)?;

    assert_eq!(read.header, original);

    Ok(())
  }

  /// A map of the given version over a 10 m grid from the origin, three metres tall, one node a place.
  fn new_map(version: u32, places: &[(u32, u16)]) -> XrfResult<Vec<u8>> {
    let header: LevelAiHeader = LevelAiHeader {
      aabb_max: Vector3d::new(90.0, 3.0, 90.0),
      aabb_min: Vector3d::new(0.0, 0.0, 0.0),
      count: places.len() as u32,
      guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
      size: 10.0,
      size_y: 3.0,
      version,
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    header.write::<XRayByteOrder>(&mut writer)?;

    let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

    for (xz, y) in places {
      // Links, covers and plane, which the search skips.
      bytes.extend([0u8; 18]);

      if version >= LevelAiHeader::WIDE_POSITION_VERSION {
        bytes.extend(xz.to_le_bytes());
      } else {
        bytes.extend(&xz.to_le_bytes()[..3]);
      }

      bytes.extend(y.to_le_bytes());
    }

    Ok(bytes)
  }

  #[test]
  fn finds_the_node_nearest_a_point_across_the_ground() -> XrfResult {
    // A row is ten nodes: 23 is the fourth row's fourth node, at x 20 and z 30.
    let bytes: Vec<u8> = new_map(10, &[(0, 0), (23, u16::MAX), (99, 0)])?;
    let nearest: Vector3d<f32> =
      LevelAiFile::find_nearest_node::<XRayByteOrder, _>(&mut ChunkReader::from_vec(bytes)?, 21.0, 29.0)?
        .expect("a node");

    assert_eq!(nearest, Vector3d::new(20.0, 3.0, 30.0));

    Ok(())
  }

  #[test]
  fn reads_the_wide_places_of_the_big_map_versions() -> XrfResult {
    let bytes: Vec<u8> = new_map(12, &[(0, 0), (99, 0)])?;
    let nearest: Vector3d<f32> =
      LevelAiFile::find_nearest_node::<XRayByteOrder, _>(&mut ChunkReader::from_vec(bytes)?, 88.0, 88.0)?
        .expect("a node");

    assert_eq!(nearest, Vector3d::new(90.0, 0.0, 90.0));

    Ok(())
  }

  #[test]
  fn takes_the_lower_of_two_nodes_at_one_place() -> XrfResult {
    let bytes: Vec<u8> = new_map(10, &[(0, u16::MAX), (0, 0)])?;
    let nearest: Vector3d<f32> =
      LevelAiFile::find_nearest_node::<XRayByteOrder, _>(&mut ChunkReader::from_vec(bytes)?, 0.0, 0.0)?
        .expect("a node");

    assert_eq!(nearest.y, 0.0);

    Ok(())
  }

  #[test]
  fn nodes_that_do_not_divide_into_their_count_are_an_error() -> XrfResult {
    let mut bytes: Vec<u8> = new_map(10, &[(0, 0), (1, 0)])?;

    bytes.pop();

    assert!(LevelAiFile::find_nearest_node::<XRayByteOrder, _>(&mut ChunkReader::from_vec(bytes)?, 0.0, 0.0).is_err());

    Ok(())
  }

  #[test]
  fn truncated_header_is_an_error_and_not_a_panic() -> XrfResult {
    let filename: String = String::from("truncated.ai");
    let mut writer: ChunkWriter = ChunkWriter::new();

    sample().write::<XRayByteOrder>(&mut writer)?;

    let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

    bytes.truncate(LevelAiHeader::SIZE as usize - 1);

    overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?
      .write_all(&bytes)?;

    assert!(
      LevelAiFile::read_from_file::<XRayByteOrder>(open_generated_test_resource_as_file(
        &build_relative_test_sample_file_path(file!(), &filename)
      )?)
      .is_err(),
      "Expected truncated AI-map header to fail reading"
    );

    Ok(())
  }
}
