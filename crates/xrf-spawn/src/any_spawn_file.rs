use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::level_spawn_file::LevelSpawnFile;
use crate::spawn_file::SpawnFile;
use crate::spawn_file_kind::SpawnFileKind;

/// A `.spawn` file in whichever of the two formats sharing the extension it is ([`SpawnFileKind`]).
#[derive(Clone, Debug)]
pub enum AnySpawnFile {
  Root(Box<SpawnFile>),
  Level(LevelSpawnFile),
}

impl AnySpawnFile {
  /// Reads a root spawn or a level spawn list from `path`, walking its top-level chunks once.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened or walked as chunks, a root spawn does not read, or the file is
  /// neither kind.
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

  /// Imports the unpacked folder at `path` as the kind its files make it ([`SpawnFileKind::of_folder`]).
  ///
  /// # Errors
  ///
  /// Returns an error when `path` is not a folder, holds neither kind's files or both, or its files do not import.
  pub fn import_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    match SpawnFileKind::of_folder(path.as_ref())? {
      SpawnFileKind::Root => Ok(Self::Root(Box::new(SpawnFile::import_from_path::<T, _>(path)?))),
      SpawnFileKind::Level => LevelSpawnFile::import_from_path::<T, _>(path).map(Self::Level),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_chunk::XRayByteOrder;
  use xrf_error::{XrfError, XrfResult};
  use xrf_math::Vector3d;
  use xrf_test_utils::GeneratedResources;

  use super::AnySpawnFile;
  use crate::constants::NET_ACTION_SPAWN;
  use crate::fixtures::{EmptySpawnFile, LevelSpawnObjects};
  use crate::level::level_spawn_file::LevelSpawnFile;
  use crate::level::level_spawn_object::LevelSpawnObject;
  use crate::spawn_file::SpawnFile;
  use crate::spawn_file_kind::SpawnFileKind;

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

  // The empty spawn's graph holds no cross table, which the import must accept too.
  #[test]
  fn imports_a_root_spawns_folder_as_its_kind() -> XrfResult {
    let path: PathBuf = GeneratedResources::create_dir("any_spawn_file/root_folder")?;
    let spawn: SpawnFile = EmptySpawnFile::build();

    spawn.export_to_path::<XRayByteOrder, _>(&path)?;

    assert_eq!(SpawnFileKind::of_folder(&path)?, SpawnFileKind::Root);

    let AnySpawnFile::Root(imported) = AnySpawnFile::import_from_path::<XRayByteOrder, _>(&path)? else {
      panic!("Expected a root spawn");
    };
    let mut written: Vec<u8> = Vec::new();
    let mut rewritten: Vec<u8> = Vec::new();

    spawn.write_to::<XRayByteOrder>(&mut written)?;
    imported.write_to::<XRayByteOrder>(&mut rewritten)?;

    assert_eq!(rewritten, written);

    Ok(())
  }

  #[test]
  fn imports_a_level_spawn_lists_folder_as_its_kind() -> XrfResult {
    let path: PathBuf = GeneratedResources::create_dir("any_spawn_file/level_folder")?;
    let list: LevelSpawnFile = LevelSpawnFile {
      objects: vec![
        LevelSpawnObjects::bolt(0)?,
        LevelSpawnObjects::shadow_of_chernobyl_graph_point(1, "l04u_graph_point_0")?,
      ],
    };

    list.export_to_path::<XRayByteOrder, _>(&path)?;

    let AnySpawnFile::Level(imported) = AnySpawnFile::import_from_path::<XRayByteOrder, _>(&path)? else {
      panic!("Expected a level's spawn list");
    };

    assert_eq!(imported, list);

    Ok(())
  }

  #[test]
  fn refuses_a_folder_of_neither_kind_or_both_naming_what_each_holds() -> XrfResult {
    let path: PathBuf = GeneratedResources::create_dir("any_spawn_file/neither_folder")?;

    fs::write(path.join("notes.txt"), "unrelated")?;

    let neither: String = AnySpawnFile::import_from_path::<XRayByteOrder, _>(&path)
      .expect_err("a folder of neither kind")
      .to_string();

    assert!(
      neither.contains("header.ltx") && neither.contains("level_spawns.ltx") && neither.contains("holds neither"),
      "{neither}"
    );

    EmptySpawnFile::build().export_to_path::<XRayByteOrder, _>(&path)?;
    LevelSpawnFile { objects: Vec::new() }.export_to_path::<XRayByteOrder, _>(&path)?;

    let both: String = AnySpawnFile::import_from_path::<XRayByteOrder, _>(&path)
      .expect_err("a folder of both kinds")
      .to_string();

    assert!(both.contains("holds both"), "{both}");

    Ok(())
  }

  // A missing folder is an IO error, not a verdict on what it holds.
  #[test]
  fn answers_a_missing_folder_as_unread() -> XrfResult {
    let path: PathBuf = GeneratedResources::create_dir("any_spawn_file/missing_folder")?.join("absent");

    assert!(matches!(
      AnySpawnFile::import_from_path::<XRayByteOrder, _>(&path),
      Err(XrfError::Io { .. })
    ));

    Ok(())
  }
}
