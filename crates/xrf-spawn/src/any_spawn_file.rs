use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level_spawn_file::LevelSpawnFile;
use crate::spawn_file::SpawnFile;
use crate::spawn_file_kind::SpawnFileKind;

/// A `.spawn` file as whichever of the two formats sharing the extension it is, told by its own chunks
/// ([`SpawnFileKind`]) since its name cannot say.
#[derive(Clone, Debug)]
pub enum AnySpawnFile {
  Root(Box<SpawnFile>),
  Level(LevelSpawnFile),
}

impl AnySpawnFile {
  /// Reads the file at `path` once, walking its top-level chunks a single time for both the kind and the read.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened or walked as chunks, when a root spawn does not read, or when the
  /// file is neither a root spawn nor a level's spawn list.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> =
      ChunkReader::from_file(File::open(path.as_ref())?)?.read_children()?;

    match SpawnFileKind::of_chunks::<T, _>(&chunks) {
      SpawnFileKind::Root => Ok(Self::Root(Box::new(SpawnFile::read_from_chunks::<T, _>(&chunks)?))),
      SpawnFileKind::Level => LevelSpawnFile::read_from_chunks::<T, _>(chunks)
        .map(Self::Level)
        .map_err(|error| {
          XrfError::new_invalid_error(format!(
            "Expected a root spawn opening with its header chunk, or a level's spawn list of spawn packets: {} is \
             neither, {error}",
            format_path(path.as_ref())
          ))
        }),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;
  use xrf_test_utils::GeneratedResources;

  use super::AnySpawnFile;
  use crate::constants::NET_ACTION_SPAWN;
  use crate::fixtures::EmptySpawnFile;
  use crate::level_spawn_file::LevelSpawnFile;
  use crate::level_spawn_object::LevelSpawnObject;

  #[test]
  fn reads_a_root_spawn_as_its_kind() -> XrfResult {
    let path: PathBuf = GeneratedResources::create_dir("any_spawn_file/root")?.join("all.spawn");

    EmptySpawnFile::build().write_to_path::<XRayByteOrder, _>(&path)?;

    assert!(matches!(
      AnySpawnFile::read_from_path::<XRayByteOrder, _>(&path)?,
      AnySpawnFile::Root(_)
    ));

    Ok(())
  }

  #[test]
  fn reads_a_level_spawn_list_as_its_kind() -> XrfResult {
    let path: PathBuf = GeneratedResources::create_dir("any_spawn_file/level")?.join("level.spawn");
    let list: LevelSpawnFile = LevelSpawnFile {
      objects: vec![LevelSpawnObject {
        id: 0,
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

    list.write_to_path::<XRayByteOrder, _>(&path)?;

    let AnySpawnFile::Level(read) = AnySpawnFile::read_from_path::<XRayByteOrder, _>(&path)? else {
      panic!("Expected a level's spawn list");
    };

    assert_eq!(read, list);

    Ok(())
  }

  #[test]
  fn refuses_a_file_of_neither_kind_naming_both() -> XrfResult {
    let path: PathBuf = GeneratedResources::create_dir("any_spawn_file/neither")?.join("container.spawn");

    // One chunk holding four bytes: a container, but neither a root's header nor a spawn packet.
    fs::write(&path, [7, 0, 0, 0, 4, 0, 0, 0, 1, 2, 3, 4])?;

    let error: String = AnySpawnFile::read_from_path::<XRayByteOrder, _>(&path)
      .expect_err("neither kind")
      .to_string();

    assert!(
      error.contains("Expected a root spawn") && error.contains("is neither"),
      "{error}"
    );

    Ok(())
  }
}
