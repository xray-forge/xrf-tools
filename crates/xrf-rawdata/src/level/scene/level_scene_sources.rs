use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use byteorder::ByteOrder;
use sha2::{Digest, Sha256};
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_editable_level::EditableLevelGuid;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelFile, LevelVisualsChunk};
use xrf_utils::format_path;

use crate::provenance::level_scene_provenance::LevelSceneSource;
use crate::provenance::sidecar::Sidecar;

/// The compiled bundle (`level`), read once for every step that needs it: the file, and its visuals where it holds any.
pub(crate) struct LevelBundle {
  pub(crate) file: LevelFile,
  pub(crate) visuals: Option<LevelVisualsChunk>,
}

/// The files of a level's directory a decompile reads: each one's bytes until a step takes them, the bundle once
/// parsed, and what the sidecar records of each, hashed as it was read.
pub(crate) struct LevelSceneSources {
  directory: PathBuf,
  files: BTreeMap<&'static str, Vec<u8>>,
  recorded: Vec<LevelSceneSource>,
  /// The bundle, read the first time a step asks for it, and shared with every step after; `None` inside where the
  /// directory holds none.
  bundle: Option<Option<Rc<LevelBundle>>>,
}

impl LevelSceneSources {
  /// Those of `names` that `directory` holds.
  ///
  /// # Errors
  ///
  /// Returns an error when one cannot be read, or the directory holds none of them.
  pub(crate) fn read(directory: &Path, names: &[&'static str]) -> XrfResult<Self> {
    let mut files: BTreeMap<&'static str, Vec<u8>> = BTreeMap::new();

    for name in names {
      let path: PathBuf = directory.join(name);

      if !files.contains_key(name) && path.is_file() {
        files.insert(
          name,
          std::fs::read(&path)
            .map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(&path))))?,
        );
      }
    }

    if files.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "{} holds none of the level files a scene decompiles from",
        format_path(directory)
      )));
    }

    let recorded: Vec<LevelSceneSource> = files
      .iter()
      .map(|(name, bytes)| LevelSceneSource {
        file: (*name).to_owned(),
        sha256: Sidecar::get_sha256_hex(bytes),
      })
      .collect();

    Ok(Self {
      directory: directory.to_path_buf(),
      files,
      recorded,
      bundle: None,
    })
  }

  /// The scene's GUID (decision 22): the first 16 bytes of the SHA-256 of each source's name, a zero, its length and
  /// its bytes, in the order of their names, then `extra`, the hash of a source read beside the directory, where given.
  pub(crate) fn get_guid(&self, extra: Option<&str>) -> EditableLevelGuid {
    let mut hash = Sha256::new();

    for (name, bytes) in &self.files {
      hash.update(name.as_bytes());
      hash.update([0]);
      hash.update((bytes.len() as u64).to_le_bytes());
      hash.update(bytes);
    }

    if let Some(extra) = extra {
      hash.update(extra.as_bytes());
    }

    let digest = hash.finalize();
    let half = |offset: usize| {
      let mut bytes: [u8; 8] = [0; 8];

      bytes.copy_from_slice(&digest[offset..offset + 8]);
      u64::from_le_bytes(bytes)
    };

    EditableLevelGuid {
      g0: half(0),
      g1: half(8),
    }
  }

  /// The bytes of `name`, handed over once, or none where the directory holds no such file.
  pub(crate) fn take(&mut self, name: &str) -> Option<Vec<u8>> {
    self.files.remove(name)
  }

  /// The file `name` read by `read`, handed over once, or none where the directory holds no such file.
  ///
  /// # Errors
  ///
  /// Returns an error as `read` does.
  pub(crate) fn take_read<F>(
    &mut self,
    name: &str,
    read: impl FnOnce(Vec<u8>) -> XrfResult<F>,
  ) -> XrfResult<Option<F>> {
    self.take(name).map(read).transpose()
  }

  /// The file `name` read by `read` without handing it over, for a step that needs a file another step decompiles,
  /// or none where the directory holds no such file or it was handed over already.
  ///
  /// # Errors
  ///
  /// Returns an error as `read` does.
  pub(crate) fn peek_read<F>(&self, name: &str, read: impl FnOnce(Vec<u8>) -> XrfResult<F>) -> XrfResult<Option<F>> {
    self.files.get(name).cloned().map(read).transpose()
  }

  /// The compiled bundle, read the first time it is asked for, or none where the directory holds no `level`.
  ///
  /// # Errors
  ///
  /// Returns an error when the bundle is not one.
  pub(crate) fn get_bundle<T: ByteOrder>(&mut self) -> XrfResult<Option<Rc<LevelBundle>>> {
    if self.bundle.is_none() {
      let bundle: Option<Rc<LevelBundle>> = match self.take(LevelFile::FILE_NAME) {
        Some(bytes) => {
          let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bytes)?.read_children()?;

          Some(Rc::new(LevelBundle {
            file: LevelFile::read_from_chunks::<T, _>(&chunks)?,
            visuals: LevelFile::read_visuals_from_chunks::<T, _>(&chunks)?,
          }))
        }
        None => None,
      };

      self.bundle = Some(bundle);
    }

    Ok(self.bundle.clone().flatten())
  }

  /// Whether the directory held `name`, taken or not.
  pub(crate) fn holds(&self, name: &str) -> bool {
    self.recorded.iter().any(|source| source.file == name)
  }

  /// Those of `names` the directory held, in their order.
  pub(crate) fn get_held(&self, names: &[&'static str]) -> Vec<&'static str> {
    names.iter().copied().filter(|name| self.holds(name)).collect()
  }

  /// The SHA-256 of `name` as read, where the directory held it.
  pub(crate) fn get_sha256(&self, name: &str) -> Option<&str> {
    self
      .recorded
      .iter()
      .find(|source| source.file == name)
      .map(|source| source.sha256.as_str())
  }

  /// `name`'s path in the level's directory.
  pub(crate) fn get_path(&self, name: &str) -> PathBuf {
    self.directory.join(name)
  }

  /// What the sidecar records of every source read.
  pub(crate) fn get_recorded(&self) -> &[LevelSceneSource] {
    &self.recorded
  }
}
