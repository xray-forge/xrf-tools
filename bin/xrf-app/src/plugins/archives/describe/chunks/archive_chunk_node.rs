use serde::Serialize;
use xrf_chunk::{CHUNK_ID_COMPRESSED_MASK, ChunkDataSource, ChunkReader};

/// How deep the walk descends before it stops calling a payload a container.
const MAXIMUM_DEPTH: usize = 8;

/// How many nodes one description carries at most.
const MAXIMUM_NODES: usize = 4096;

/// One chunk of a container, and whatever its payload turned out to hold.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveChunkNode {
  /// Chunk id with the compression flag masked off, which is what a format's own constants compare against.
  pub id: u32,
  /// Bytes of payload, which the children below account for in full when there are any.
  pub size: u64,
  /// Whether the id carried `CFS_CompressMark`, in which case the payload stands for data it is not and is not walked.
  pub is_compressed: bool,
  /// Chunks the payload is made of, empty for one holding data rather than a container.
  pub children: Vec<ArchiveChunkNode>,
}

impl ArchiveChunkNode {
  /// The container a file is, or nothing for bytes that are not one.
  pub fn of_bytes(bytes: Vec<u8>) -> Vec<Self> {
    let mut budget: usize = MAXIMUM_NODES;

    ChunkReader::from_vec(bytes).map_or_else(|_| Vec::new(), |mut reader| Self::walk(&mut reader, &mut budget, 0))
  }

  /// Every node of a tree, the nested ones included.
  pub fn count(nodes: &[Self]) -> usize {
    nodes.iter().map(|node| 1 + Self::count(&node.children)).sum()
  }

  /// How far the deepest branch descends, counting a flat list as one.
  pub fn depth(nodes: &[Self]) -> usize {
    nodes
      .iter()
      .map(|node| 1 + Self::depth(&node.children))
      .max()
      .unwrap_or_default()
  }

  /// Walks one reader's children, descending into a payload only where it is a container in its own right.
  fn walk<D: ChunkDataSource>(reader: &mut ChunkReader<D>, budget: &mut usize, depth: usize) -> Vec<Self> {
    let Ok(chunks) = reader.read_children_including_compressed() else {
      return Vec::new();
    };

    if !reader.is_ended() {
      return Vec::new();
    }

    let mut nodes: Vec<Self> = Vec::new();

    for mut chunk in chunks {
      if *budget == 0 {
        break;
      }

      *budget -= 1;

      let is_compressed: bool = chunk.id & CHUNK_ID_COMPRESSED_MASK != 0;

      nodes.push(Self {
        id: chunk.id & !CHUNK_ID_COMPRESSED_MASK,
        size: chunk.size,
        is_compressed,
        children: if is_compressed || depth + 1 >= MAXIMUM_DEPTH {
          Vec::new()
        } else {
          Self::walk(&mut chunk, budget, depth + 1)
        },
      });
    }

    nodes
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::CHUNK_ID_COMPRESSED_MASK;

  use super::ArchiveChunkNode;

  /// One chunk as a container frames it: the id, the payload length, then the payload.
  fn chunk(id: u32, payload: &[u8]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend_from_slice(&id.to_le_bytes());
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
  }

  #[test]
  fn a_flat_container_reads_as_its_chunks() {
    let mut bytes: Vec<u8> = chunk(1, b"first");

    bytes.extend(chunk(2, b"second"));

    let nodes: Vec<ArchiveChunkNode> = ArchiveChunkNode::of_bytes(bytes);

    assert_eq!(nodes.len(), 2);
    assert_eq!((nodes[0].id, nodes[0].size), (1, 5));
    assert_eq!((nodes[1].id, nodes[1].size), (2, 6));
    assert_eq!(ArchiveChunkNode::depth(&nodes), 1);
  }

  #[test]
  fn a_payload_that_is_itself_a_container_is_descended_into() {
    let nested: Vec<u8> = [chunk(10, b"a"), chunk(11, b"b")].concat();
    let nodes: Vec<ArchiveChunkNode> = ArchiveChunkNode::of_bytes(chunk(1, &nested));

    assert_eq!(ArchiveChunkNode::count(&nodes), 3);
    assert_eq!(ArchiveChunkNode::depth(&nodes), 2);
    assert_eq!(nodes[0].children.len(), 2);
  }

  #[test]
  fn a_payload_its_children_do_not_account_for_is_left_as_data() {
    // A header-shaped prefix followed by bytes nothing claims. Drawing two children here would be inventing
    // structure, which is the one thing a description of an unknown format must not do.
    let mut payload: Vec<u8> = chunk(10, b"a");

    payload.extend_from_slice(b"trailing");

    let nodes: Vec<ArchiveChunkNode> = ArchiveChunkNode::of_bytes(chunk(1, &payload));

    assert_eq!(nodes.len(), 1);
    assert!(nodes[0].children.is_empty(), "a partial walk is data, not a container");
  }

  #[test]
  fn bytes_that_are_not_a_container_read_as_none() {
    assert!(ArchiveChunkNode::of_bytes(b"not a chunk at all".to_vec()).is_empty());
  }

  #[test]
  fn a_compressed_chunk_is_reported_by_its_unmasked_id_and_not_walked() {
    // `gamemtl.xr` carries one, and a walk that refused it would report nothing for the richest container there is.
    let nested: Vec<u8> = chunk(10, b"a");
    let nodes: Vec<ArchiveChunkNode> = ArchiveChunkNode::of_bytes(chunk(7 | CHUNK_ID_COMPRESSED_MASK, &nested));

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, 7);
    assert!(nodes[0].is_compressed);
    assert!(
      nodes[0].children.is_empty(),
      "its payload is not the data it stands for"
    );
  }
}
