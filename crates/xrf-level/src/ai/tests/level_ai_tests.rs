use uuid::uuid;
use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::ai::level_ai_file::LevelAiFile;
use crate::ai::level_ai_header::LevelAiHeader;
use crate::ai::level_ai_node::LevelAiNode;
use crate::ai::level_ai_nodes::LevelAiNodes;
use crate::ai::level_ai_version::LevelAiVersion;

/// A header of `count` nodes in `version`, over a 10 by 6 metre box of 0.7 metre cells.
fn header(version: LevelAiVersion, count: u32) -> LevelAiHeader {
  LevelAiHeader {
    version: version as u32,
    count,
    size: 0.7,
    size_y: 20.0,
    aabb_min: Vector3d::new(-5.0, -10.0, -3.0),
    aabb_max: Vector3d::new(5.0, 10.0, 3.0),
    guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
  }
}

/// One node packed as `version` lays it out: the links as one little-endian bit run, then cover, plane and cell.
fn node_bytes(version: LevelAiVersion, node: &LevelAiNode) -> Vec<u8> {
  let (link_bits, link_bytes): (u32, usize) = match version {
    LevelAiVersion::Soc | LevelAiVersion::Priquel | LevelAiVersion::CsCop => (23, 12),
    LevelAiVersion::Borsht | LevelAiVersion::BorshtBig => (25, 13),
    LevelAiVersion::Skyloader => (26, 13),
  };
  let links: u128 = node.links.iter().enumerate().fold(0, |packed, (index, link)| {
    packed | (u128::from(*link) << (index as u32 * link_bits))
  });
  let cover = |cover: [u8; 4]| -> [u8; 2] {
    cover
      .iter()
      .enumerate()
      .fold(0u16, |packed, (index, value)| {
        packed | (u16::from(*value) << (index * 4))
      })
      .to_le_bytes()
  };
  let mut bytes: Vec<u8> = links.to_le_bytes()[..link_bytes].to_vec();

  bytes.extend(cover(node.high_cover));

  if !matches!(version, LevelAiVersion::Soc | LevelAiVersion::Priquel) {
    bytes.extend(cover(node.low_cover));
  }

  bytes.extend(node.plane.to_le_bytes());

  if matches!(version, LevelAiVersion::BorshtBig | LevelAiVersion::Skyloader) {
    bytes.extend(node.packed_xz.to_le_bytes());
  } else {
    bytes.extend(&node.packed_xz.to_le_bytes()[..3]);
  }

  bytes.extend(node.packed_y.to_le_bytes());

  bytes
}

/// A whole map: the header, then each node packed in its version.
fn map_bytes(header: &LevelAiHeader, nodes: &[LevelAiNode]) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  header.write::<XRayByteOrder>(&mut writer)?;

  let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

  for node in nodes {
    bytes.extend(node_bytes(LevelAiVersion::from_number(header.version)?, node));
  }

  Ok(bytes)
}

/// A node with every field away from zero, its links filling the version's widest link.
fn sample_node(widest_link: u32) -> LevelAiNode {
  LevelAiNode {
    links: [1, widest_link, 0, 7],
    high_cover: [1, 15, 0, 8],
    low_cover: [3, 2, 14, 9],
    plane: 126,
    packed_xz: 3 * 9 + 4,
    packed_y: 32768,
  }
}

#[test]
fn reads_and_writes_back_the_header() -> XrfResult {
  let original: LevelAiHeader = header(LevelAiVersion::CsCop, 1_356_204);
  let mut writer: ChunkWriter = ChunkWriter::new();

  original.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.bytes_written() as u64, LevelAiHeader::SIZE);
  assert_eq!(
    LevelAiFile::read_from_bytes::<XRayByteOrder>(writer.flush_raw_into_buffer()?)?.header,
    original
  );

  Ok(())
}

#[test]
fn a_truncated_header_is_an_error_and_not_a_panic() -> XrfResult {
  let mut writer: ChunkWriter = ChunkWriter::new();

  header(LevelAiVersion::CsCop, 0).write::<XRayByteOrder>(&mut writer)?;

  let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

  bytes.truncate(LevelAiHeader::SIZE as usize - 1);

  assert!(LevelAiFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}

#[test]
fn decodes_a_node_of_every_version() -> XrfResult {
  for (version, widest_link) in [
    (LevelAiVersion::Soc, (1 << 23) - 1),
    (LevelAiVersion::Priquel, (1 << 23) - 1),
    (LevelAiVersion::CsCop, (1 << 23) - 1),
    (LevelAiVersion::Borsht, (1 << 25) - 1),
    (LevelAiVersion::BorshtBig, (1 << 25) - 1),
    (LevelAiVersion::Skyloader, (1 << 26) - 1),
  ] {
    let expected = |widest_link: u32| -> LevelAiNode {
      let mut node: LevelAiNode = sample_node(widest_link);

      if matches!(version, LevelAiVersion::Soc | LevelAiVersion::Priquel) {
        // One cover set, which the engine hands out as both.
        node.low_cover = node.high_cover;
      }

      node
    };
    let bytes: Vec<u8> = map_bytes(&header(version, 2), &[expected(widest_link), expected(0)])?;
    let (file, nodes): (LevelAiFile, LevelAiNodes) = LevelAiFile::read_with_nodes_from_bytes::<XRayByteOrder>(bytes)?;

    assert_eq!(file.header.count, 2);
    assert_eq!(nodes.get_version(), version);
    assert_eq!(nodes.get_count(), 2);
    assert_eq!(
      nodes.get_node::<XRayByteOrder>(0),
      Some(expected(widest_link)),
      "{version:?}"
    );
    assert_eq!(nodes.get_node::<XRayByteOrder>(1), Some(expected(0)), "{version:?}");
    assert_eq!(nodes.get_node::<XRayByteOrder>(2), None);
    assert_eq!(nodes.iter_nodes::<XRayByteOrder>().count(), 2);
  }

  Ok(())
}

#[test]
fn a_node_sits_at_its_cell_centre_and_packed_height() {
  let header: LevelAiHeader = header(LevelAiVersion::CsCop, 1);
  let node: LevelAiNode = sample_node(0);

  // 6 metres across z in 0.7 metre cells is floor(8.57 + 1.501) = 10 cells a row, and 10 metres across x is 15 rows.
  assert_eq!(header.get_row_length(), 10);
  assert_eq!(header.get_column_length(), 15);

  let position: Vector3d = header.get_node_position(&node);

  // Cell 31 is row 3, column 1: x = 3 * 0.7 - 5, z = 1 * 0.7 - 3, and the height is halfway up the 20 metre extent.
  assert!((position.x - (3.0 * 0.7 - 5.0)).abs() < 1e-5, "{position}");
  assert!(
    (position.y - (32768.0 / 65535.0 * 20.0 - 10.0)).abs() < 1e-5,
    "{position}"
  );
  assert!((position.z - (0.7 - 3.0)).abs() < 1e-5, "{position}");
  assert!(
    node.get_normal().distance(&Vector3d::new(0.0, 1.0, 0.0)) < 1e-6,
    "{}",
    node.get_normal()
  );
}

#[test]
fn a_link_past_the_node_count_is_no_neighbour() {
  assert_eq!(sample_node(5).get_neighbours(6).collect::<Vec<u32>>(), [1, 5, 0]);
}

#[test]
fn writes_the_map_back_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = map_bytes(&header(LevelAiVersion::CsCop, 2), &[sample_node(9), sample_node(0)])?;
  let (file, nodes): (LevelAiFile, LevelAiNodes) =
    LevelAiFile::read_with_nodes_from_bytes::<XRayByteOrder>(bytes.clone())?;
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&nodes, &mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}

#[test]
fn nodes_that_are_not_the_ones_the_header_counts_are_refused() -> XrfResult {
  let mut bytes: Vec<u8> = map_bytes(&header(LevelAiVersion::CsCop, 2), &[sample_node(0)])?;
  let error: String = LevelAiFile::read_with_nodes_from_bytes::<XRayByteOrder>(bytes.clone())
    .expect_err("a missing node is refused")
    .to_string();

  assert_eq!(
    error,
    "Invalid error: Level AI-map counts 2 nodes of 23 bytes, but 23 bytes follow its header"
  );

  bytes.extend(node_bytes(LevelAiVersion::CsCop, &sample_node(0)));
  bytes.push(0);

  assert!(LevelAiFile::read_with_nodes_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}

#[test]
fn a_version_the_engine_refuses_is_refused() -> XrfResult {
  let mut header: LevelAiHeader = header(LevelAiVersion::CsCop, 0);

  header.version = 7;

  let error: String = LevelAiFile::read_with_nodes_from_bytes::<XRayByteOrder>(map_bytes(&header, &[])?)
    .expect_err("version 7 is not loaded")
    .to_string();

  assert!(error.contains("version 7"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn writing_nodes_the_header_does_not_count_is_refused() -> XrfResult {
  let (file, nodes): (LevelAiFile, LevelAiNodes) = LevelAiFile::read_with_nodes_from_bytes::<XRayByteOrder>(
    map_bytes(&header(LevelAiVersion::CsCop, 1), &[sample_node(0)])?,
  )?;
  let mut other: LevelAiFile = file.clone();

  other.header.count = 2;

  assert!(other.write::<XRayByteOrder>(&nodes, &mut ChunkWriter::new()).is_err());

  Ok(())
}
