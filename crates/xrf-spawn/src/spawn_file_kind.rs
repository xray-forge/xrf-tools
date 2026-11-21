use std::io::ErrorKind;
use std::path::Path;

use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::chunks::spawn_header_chunk::SpawnHeaderChunk;
use crate::level::level_spawn_file::LevelSpawnFile;

/// Which of the two formats sharing `.spawn` a file or its unpacked folder is: a root spawn such as `all.spawn`
/// ([`SpawnFile`](crate::SpawnFile)), or a level's `level.spawn` run of spawn packets ([`LevelSpawnFile`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpawnFileKind {
  /// A root spawn, which opens with its header chunk.
  Root,
  /// A level's spawn list: anything not opening with a root's header, until the list's own read confirms or refuses it.
  Level,
}

impl SpawnFileKind {
  /// The kind a file's top-level chunks make it, reading no more than the header chunk's own bytes.
  pub fn of_chunks<D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> Self {
    match Self::read_root_header::<D>(chunks) {
      Some(_) => Self::Root,
      None => Self::Level,
    }
  }

  /// The header that makes top-level chunks a root spawn, or `None` where they carry none.
  ///
  /// A level list numbers its packets from zero, so its first chunk shares the header's id: only whether that chunk
  /// reads whole as a header tells the two apart.
  pub fn read_root_header<D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> Option<SpawnHeaderChunk> {
    ChunkReader::find_optional_by_id(chunks, SpawnHeaderChunk::CHUNK_ID)?
      .read_xr::<SpawnHeaderChunk>()
      .ok()
  }

  /// The kind an unpacked folder holds: a root spawn's holds its header file, a level list's its objects file.
  ///
  /// # Errors
  ///
  /// Returns an IO error when `path` is not a folder, and an invalid error when it holds both kinds' files or neither.
  pub fn of_folder(path: &Path) -> XrfResult<Self> {
    if !path.is_dir() {
      return Err(XrfError::new_io_error(
        format!("Unpacked spawn folder {} is not a folder", format_path(path)),
        ErrorKind::NotFound,
      ));
    }

    let is_root: bool = path.join(SpawnHeaderChunk::UNPACKED_FILE_NAME).is_file();
    let is_level: bool = path.join(LevelSpawnFile::UNPACKED_FILE_NAME).is_file();

    if is_root != is_level {
      return Ok(if is_root { Self::Root } else { Self::Level });
    }

    Err(XrfError::new_invalid_error(format!(
      "Expected an unpacked spawn folder, a root spawn's holding {} beside its other chunks' files or a level spawn \
       list's holding {}: {} holds {}",
      SpawnHeaderChunk::UNPACKED_FILE_NAME,
      LevelSpawnFile::UNPACKED_FILE_NAME,
      format_path(path),
      if is_root { "both" } else { "neither" }
    )))
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;

  use super::SpawnFileKind;
  use crate::chunks::spawn_header_chunk::SpawnHeaderChunk;
  use crate::constants::NET_ACTION_SPAWN;
  use crate::fixtures::EmptySpawnFile;
  use crate::level::level_spawn_file::LevelSpawnFile;
  use crate::level::level_spawn_object::LevelSpawnObject;

  fn read_kind(bytes: Vec<u8>) -> XrfResult<SpawnFileKind> {
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bytes)?.read_children()?;

    Ok(SpawnFileKind::of_chunks(&chunks))
  }

  #[test]
  fn tells_a_root_spawn_by_its_header() -> XrfResult {
    let mut bytes: Vec<u8> = Vec::new();

    EmptySpawnFile::build().write_to(&mut bytes)?;

    assert_eq!(read_kind(bytes)?, SpawnFileKind::Root);

    Ok(())
  }

  #[test]
  fn tells_a_level_spawn_list_though_its_first_chunk_has_the_header_id() -> XrfResult {
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

    list.write(&mut writer)?;

    assert_eq!(read_kind(writer.flush_raw_into_buffer()?)?, SpawnFileKind::Level);

    Ok(())
  }
}
