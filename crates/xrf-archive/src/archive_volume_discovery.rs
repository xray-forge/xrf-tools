use std::path::{Path, PathBuf};

use walkdir::{DirEntry, Error as WalkError, WalkDir};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path_or;

use crate::ArchiveDescriptor;

/// Finds archive volumes and orders them for the project's merge.
pub(super) struct ArchiveVolumeDiscovery;

impl ArchiveVolumeDiscovery {
  /// Volume paths at a path, in merge order: a named file is itself, a directory is walked to `depth`.
  pub(super) fn discover(path: &Path, depth: usize) -> XrfResult<Vec<PathBuf>> {
    if path.is_file() {
      return Ok(vec![path.to_path_buf()]);
    }

    Self::collect_volumes(
      WalkDir::new(path)
        .max_depth(depth)
        .into_iter()
        .map(|entry| entry.map(DirEntry::into_path).map_err(Self::describe_walk_failure)),
    )
  }

  /// The volumes among walked entries, in merge order, failing on the first entry the walk could not reach.
  fn collect_volumes(entries: impl IntoIterator<Item = XrfResult<PathBuf>>) -> XrfResult<Vec<PathBuf>> {
    let mut volumes: Vec<PathBuf> = Vec::new();

    for entry in entries {
      let path: PathBuf = entry?;

      if ArchiveDescriptor::is_valid_db_path(&path) {
        volumes.push(path);
      }
    }

    Self::sort_volumes(&mut volumes);

    Ok(volumes)
  }

  /// Keeps the failing path and the io kind, which `walkdir` carries only as display text.
  fn describe_walk_failure(error: WalkError) -> XrfError {
    let at: String = format_path_or(error.path(), "an unnamed entry").to_string();

    match error.io_error() {
      Some(cause) => XrfError::new_io_error(format!("Unable to walk archive path {at}: {cause}"), cause.kind()),
      None => XrfError::new_read_error(format!("Unable to walk archive path {at}: {error}")),
    }
  }

  /// Orders volumes the way the engine registers them, so the last one merged is the one it would answer with.
  fn sort_volumes(volumes: &mut [PathBuf]) {
    volumes.sort();
  }
}

#[cfg(test)]
mod tests {
  use std::io::ErrorKind;
  use std::path::{Path, PathBuf};

  use xrf_error::{XrfError, XrfResult};

  use super::ArchiveVolumeDiscovery;
  use crate::ArchiveDescriptor;

  #[test]
  fn volumes_merge_in_the_order_the_engine_registers_them() {
    // `Recurse` sorts each directory's entries by name and descends as the name comes up, so a subdirectory is read
    // where its own name sorts rather than before or after every file. `patches` is not special to any engine.
    let mut volumes: [PathBuf; 5] = [
      PathBuf::from("/game/db/patches/xpatch_1.db"),
      PathBuf::from("/game/db/textures/textures.db0"),
      PathBuf::from("/game/db/configs.db0"),
      PathBuf::from("/game/db/textures.db0"),
      PathBuf::from("/game/db/addons/mod.db0"),
    ];

    ArchiveVolumeDiscovery::sort_volumes(&mut volumes);

    let order: Vec<&str> = volumes.iter().map(|it| it.to_str().expect("utf-8")).collect();

    assert_eq!(
      order,
      vec![
        "/game/db/addons/mod.db0",
        "/game/db/configs.db0",
        "/game/db/patches/xpatch_1.db",
        // The directory sorts before the file whose name it prefixes, as `xr_strcmp` orders the two entries.
        "/game/db/textures/textures.db0",
        "/game/db/textures.db0",
      ]
    );
  }

  #[test]
  fn recognizes_volume_extensions_without_case() {
    assert!(ArchiveDescriptor::is_valid_db_path(Path::new("game.db0")));
    assert!(ArchiveDescriptor::is_valid_db_path(Path::new("GAME.DB0")));
    assert!(ArchiveDescriptor::is_valid_db_path(Path::new("mod.xdb1")));
    assert!(!ArchiveDescriptor::is_valid_db_path(Path::new("readme.txt")));
    assert!(!ArchiveDescriptor::is_valid_db_path(Path::new("noextension")));
  }

  #[test]
  fn recognizes_a_volume_whose_whole_name_is_its_extension() {
    assert!(ArchiveDescriptor::is_valid_db_path(Path::new("db/.db0")));
    assert!(ArchiveDescriptor::is_valid_db_path(Path::new(".xdb1")));

    // A dot in a directory name is still not the extension of an extensionless file.
    assert!(!ArchiveDescriptor::is_valid_db_path(Path::new("game.db0/header")));
  }

  /// A walk failure used to be filtered away, so an unreadable descendant left the project quietly short of the volumes
  /// below it whenever a readable sibling kept the open succeeding.
  #[test]
  fn discovery_fails_on_an_entry_the_walk_cannot_reach() {
    let entries: [XrfResult<PathBuf>; 3] = [
      Ok(PathBuf::from("/game/db/configs.db0")),
      Err(XrfError::new_io_error("locked", ErrorKind::PermissionDenied)),
      Ok(PathBuf::from("/game/db/textures.db0")),
    ];

    let error: XrfError = ArchiveVolumeDiscovery::collect_volumes(entries).expect_err("walk failure fails discovery");

    assert!(
      matches!(
        error,
        XrfError::Io {
          kind: ErrorKind::PermissionDenied,
          ..
        }
      ),
      "walk failure keeps its io kind, got {error}"
    );
  }

  #[test]
  fn discovery_keeps_the_volumes_among_walked_entries() -> XrfResult {
    let entries: [XrfResult<PathBuf>; 3] = [
      Ok(PathBuf::from("/game/db/textures.db0")),
      Ok(PathBuf::from("/game/db/readme.txt")),
      Ok(PathBuf::from("/game/db/configs.db0")),
    ];

    assert_eq!(
      ArchiveVolumeDiscovery::collect_volumes(entries)?,
      vec![
        PathBuf::from("/game/db/configs.db0"),
        PathBuf::from("/game/db/textures.db0")
      ]
    );

    Ok(())
  }

  /// The end of the same contract on a real walk: only Unix can make a directory the process cannot enter, and a run as
  /// root ignores the mode, so the check states what it skipped rather than passing without making it.
  #[test]
  #[cfg(unix)]
  fn discovery_fails_on_an_unreadable_directory() -> XrfResult {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

    use crate::ArchiveProject;

    let root: PathBuf = build_absolute_generated_test_resource_path("archive_project_unreadable");
    let locked: PathBuf = root.join("locked");

    fs::create_dir_all(&locked)?;
    fs::write(root.join("configs.db0"), b"")?;
    fs::write(locked.join("textures.db0"), b"")?;
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000))?;

    let is_enforced: bool = fs::read_dir(&locked).is_err();
    let discovered: XrfResult<Vec<PathBuf>> = ArchiveProject::get_discover_volumes(&root);

    // Restored before asserting, or the failure leaves a directory the run cannot clean up.
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o700))?;
    fs::remove_dir_all(&root)?;

    if is_enforced {
      assert!(
        discovered.is_err(),
        "an unreadable directory fails discovery instead of hiding the volume below it"
      );
    } else {
      eprintln!("skipped: this process reads a 0o000 directory, so the mode enforces nothing");
    }

    Ok(())
  }
}
