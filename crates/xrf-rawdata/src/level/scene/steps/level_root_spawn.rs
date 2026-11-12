use std::path::Path;

use byteorder::ByteOrder;
use xrf_chunk::ChunkReader;
use xrf_error::{XrfError, XrfResult};
use xrf_spawn::{AlifeObject, SpawnFile, SpawnLevelObjects};
use xrf_utils::format_path;

use crate::provenance::level_scene_provenance::LevelSceneRootSpawn;
use crate::provenance::sidecar::Sidecar;

/// A root spawn read for one level: what the sidecar records of it, and its objects on the level, which
/// the game step takes in place of `level.spawn`'s entities.
pub(crate) struct LevelRootSpawn {
  pub(crate) record: LevelSceneRootSpawn,
  pub(crate) objects: Vec<AlifeObject>,
}

impl LevelRootSpawn {
  /// The objects of the root spawn at `path` standing on the level `directory` is named after, by their game vertices.
  ///
  /// # Errors
  ///
  /// Returns an error when the spawn cannot be read or holds no objects chunk or graph, or the directory names no
  /// level.
  pub(crate) fn read<T: ByteOrder>(path: &Path, directory: &Path) -> XrfResult<Self> {
    let bytes: Vec<u8> = std::fs::read(path)
      .map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(path))))?;
    let sha256: String = Sidecar::get_sha256_hex(&bytes);
    let level: &str = directory
      .file_name()
      .and_then(|name| name.to_str())
      .ok_or_else(|| XrfError::new_invalid_error(format!("{} names no level", format_path(directory))))?;
    let chunks = ChunkReader::from_vec(bytes)?.read_children()?;
    let objects: SpawnLevelObjects = SpawnFile::read_level_objects_from_chunks::<T, _>(&chunks, level)?;

    Ok(Self {
      record: LevelSceneRootSpawn {
        file: format_path(path).to_string(),
        sha256,
        objects: objects.objects.len(),
        unread: objects.skipped.len(),
        disagreements: Vec::new(),
        update_packets: Vec::new(),
      },
      objects: objects.objects,
    })
  }

  /// The hash the scene's GUID takes of it, where it holds objects on the level: only then do the entities come from
  /// it.
  pub(crate) fn get_guid_hash(&self) -> Option<&str> {
    (!self.objects.is_empty()).then_some(self.record.sha256.as_str())
  }
}
