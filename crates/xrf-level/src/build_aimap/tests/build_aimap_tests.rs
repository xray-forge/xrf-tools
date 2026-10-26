use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::build_aimap::build_aimap_file::BuildAimapFile;
use crate::build_aimap::build_aimap_node::BuildAimapNode;
use crate::tests::fixtures::{chunk, floats};

/// One node with `link_size`-byte links: the first neighbour is node 1 and the rest are the version's invalid link.
fn node_bytes(link_size: usize, x: i16, y: u16, z: i16) -> Vec<u8> {
  let mut bytes: Vec<u8> = 1u32.to_le_bytes()[..link_size].to_vec();

  for _ in 0..3 {
    bytes.extend_from_slice(&u32::MAX.to_le_bytes()[..link_size]);
  }

  bytes.extend_from_slice(&126u16.to_le_bytes());
  bytes.extend_from_slice(&x.to_le_bytes());
  bytes.extend_from_slice(&y.to_le_bytes());
  bytes.extend_from_slice(&z.to_le_bytes());

  bytes
}

/// A map of two nodes in the given version, laid out as the editor exports one.
fn map_bytes(version: u16, link_size: usize) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = chunk(BuildAimapFile::VERSION_CHUNK_ID, &version.to_le_bytes())?;

  bytes.extend(chunk(
    BuildAimapFile::BOX_CHUNK_ID,
    &floats(&[-14.7, 1.0, -14.7, 14.7, 3.0, 14.7]),
  )?);
  bytes.extend(chunk(BuildAimapFile::PARAMS_CHUNK_ID, &floats(&[0.7, 1.0, 1.5, 4.0]))?);

  let mut nodes: Vec<u8> = 2u32.to_le_bytes().to_vec();

  nodes.extend(node_bytes(link_size, -21, 0, 21));
  nodes.extend(node_bytes(link_size, 3, u16::MAX, -4));
  bytes.extend(chunk(BuildAimapFile::NODES_CHUNK_ID, &nodes)?);

  Ok(bytes)
}

#[test]
fn a_node_is_twenty_bytes_in_the_sdk_version_and_twenty_four_in_ix_ray_s() {
  assert_eq!(BuildAimapNode::get_serialized_size(false), 20);
  assert_eq!(BuildAimapNode::get_serialized_size(true), 24);
  assert_eq!(
    node_bytes(3, 0, 0, 0).len() as u64,
    BuildAimapNode::get_serialized_size(false)
  );
}

#[test]
fn an_sdk_map_reads_three_byte_links_and_its_invalid_link_as_their_widest_value() -> XrfResult {
  let file: BuildAimapFile = BuildAimapFile::read_from_bytes::<XRayByteOrder>(map_bytes(1, 3)?)?;

  assert_eq!(file.version, BuildAimapFile::NARROW_LINKS_VERSION);
  assert_eq!(file.params.patch_size, 0.7);
  assert_eq!(file.nodes.len(), 2);
  assert_eq!(
    file.nodes[0].links,
    [
      1,
      BuildAimapNode::NARROW_LINK_MASK,
      BuildAimapNode::NARROW_LINK_MASK,
      BuildAimapNode::NARROW_LINK_MASK
    ]
  );
  assert_eq!((file.nodes[0].x, file.nodes[0].y, file.nodes[0].z), (-21, 0, 21));

  Ok(())
}

#[test]
fn an_ix_ray_map_reads_four_byte_links() -> XrfResult {
  let file: BuildAimapFile = BuildAimapFile::read_from_bytes::<XRayByteOrder>(map_bytes(2, 4)?)?;

  assert_eq!(file.nodes[1].links, [1, u32::MAX, u32::MAX, u32::MAX]);
  assert_eq!((file.nodes[1].x, file.nodes[1].y, file.nodes[1].z), (3, u16::MAX, -4));

  Ok(())
}

#[test]
fn a_node_sits_where_the_editor_unpacks_it() -> XrfResult {
  let file: BuildAimapFile = BuildAimapFile::read_from_bytes::<XRayByteOrder>(map_bytes(1, 3)?)?;

  assert_eq!(
    file.get_node_position(&file.nodes[1]),
    Vector3d::new(3.0 * 0.7, 3.0, -4.0 * 0.7)
  );
  assert_eq!(
    file.get_node_position(&file.nodes[0]).y,
    1.0,
    "height 0 is the box floor"
  );

  Ok(())
}

#[test]
fn a_map_of_either_version_is_written_back_byte_for_byte() -> XrfResult {
  for (version, link_size) in [(1, 3), (2, 4)] {
    let bytes: Vec<u8> = map_bytes(version, link_size)?;
    let file: BuildAimapFile = BuildAimapFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;
    let mut writer: ChunkWriter = ChunkWriter::new();

    file.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, bytes, "version {version}");
  }

  Ok(())
}

#[test]
fn a_link_too_wide_for_an_sdk_map_is_refused_rather_than_cut() -> XrfResult {
  let mut file: BuildAimapFile = BuildAimapFile::read_from_bytes::<XRayByteOrder>(map_bytes(1, 3)?)?;

  file.nodes[0].links[0] = 1 << 24;

  let error: String = file
    .write::<XRayByteOrder>(&mut ChunkWriter::new())
    .expect_err("expect the wide link to be refused")
    .to_string();

  assert!(error.contains("does not fit"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn a_map_of_a_version_no_editor_writes_is_refused() -> XrfResult {
  let error: String = BuildAimapFile::read_from_bytes::<XRayByteOrder>(map_bytes(3, 4)?)
    .expect_err("expect the version to be refused")
    .to_string();

  assert!(error.contains("version 3"), "Unexpected error: {error}");

  Ok(())
}
