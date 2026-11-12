use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader};

use crate::chunks::spawn_header_chunk::SpawnHeaderChunk;

/// Which of the two formats sharing `.spawn` a file is, which its name cannot say: a root spawn such as `all.spawn`
/// ([`SpawnFile`](crate::SpawnFile)), or the run of spawn packets a level's `level.spawn` holds
/// ([`LevelSpawnFile`](crate::LevelSpawnFile)).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpawnFileKind {
  /// A root spawn, which opens with its header chunk.
  Root,
  /// A level's spawn list: anything not opening with a root's header, which the list's own read then confirms or
  /// refuses.
  Level,
}

impl SpawnFileKind {
  /// The kind a file's top-level chunks make it, reading no more than the header chunk's own bytes.
  pub fn of_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> Self {
    match Self::read_root_header::<T, D>(chunks) {
      Some(_) => Self::Root,
      None => Self::Level,
    }
  }

  /// The header that makes top-level chunks a root spawn, or `None` where they carry none.
  ///
  /// A level's list numbers its packets from zero, so its first chunk has the header's id as well: what tells the two
  /// apart is whether that chunk reads whole as a header.
  pub fn read_root_header<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> Option<SpawnHeaderChunk> {
    ChunkReader::find_optional_by_id(chunks, SpawnHeaderChunk::CHUNK_ID)?
      .read_xr::<T, SpawnHeaderChunk>()
      .ok()
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;

  use super::SpawnFileKind;
  use crate::chunks::spawn_header_chunk::SpawnHeaderChunk;
  use crate::constants::NET_ACTION_SPAWN;
  use crate::fixtures::EmptySpawnFile;
  use crate::level_spawn_file::LevelSpawnFile;
  use crate::level_spawn_object::LevelSpawnObject;

  fn read_kind(bytes: Vec<u8>) -> XrfResult<SpawnFileKind> {
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bytes)?.read_children()?;

    Ok(SpawnFileKind::of_chunks::<XRayByteOrder, _>(&chunks))
  }

  #[test]
  fn tells_a_root_spawn_by_its_header() -> XrfResult {
    let mut bytes: Vec<u8> = Vec::new();

    EmptySpawnFile::build().write_to::<XRayByteOrder>(&mut bytes)?;

    assert_eq!(read_kind(bytes)?, SpawnFileKind::Root);

    Ok(())
  }

  #[test]
  fn tells_a_level_spawn_list_though_its_first_chunk_has_the_header_id() -> XrfResult {
    // A list numbers its packets from zero, so its first chunk carries the header's id and only the bytes differ.
    let list: LevelSpawnFile = LevelSpawnFile {
      objects: vec![LevelSpawnObject {
        id: SpawnHeaderChunk::CHUNK_ID,
        net_action: NET_ACTION_SPAWN,
        section: String::from("graph_point"),
        name: String::from("fake_graph_point"),
        script_game_id: 0,
        script_rp: 0xFF,
        position: Vector3d::new(1.0, 2.0, 3.0),
        direction: Vector3d::ZERO,
        payload: vec![7; 12],
      }],
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    list.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(read_kind(writer.flush_raw_into_buffer()?)?, SpawnFileKind::Level);

    Ok(())
  }
}
