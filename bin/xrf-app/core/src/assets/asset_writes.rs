use std::fs::{self, File};
use std::io::{ErrorKind, Result as IoResult, Write};
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_utils::{StagedFile, format_path};
use xrf_vfs::XrayInputRecord;

use crate::assets::AssetMountState;
use crate::tauri_result::TauriResult;

/// The files one run writes by path, each staged beside its target as it is written and all published together: the
/// one way the application writes a file into a game tree. Those a writable loose mount holds land as one commit, all
/// or none, moving every followed session and heard as any commit; the rest, outside every mount, are each moved into
/// place, since no snapshot reads them. Dropped unpublished, it removes everything it staged.
#[derive(Debug, Default)]
pub struct AssetWrites {
  staged: Vec<StagedFile>,
}

impl AssetWrites {
  /// Stages `bytes` to stand at `path`, for a file built whole in memory.
  ///
  /// # Errors
  ///
  /// As [`Self::stage_with`].
  pub fn stage_bytes(&mut self, path: &Path, bytes: &[u8]) -> TauriResult {
    self.stage_with(path, |file| file.write_all(bytes))
  }

  /// Stages what `write` streams into an open file to stand at `path`, making the folders above it where missing, for
  /// a file too large to hold in memory.
  ///
  /// # Errors
  ///
  /// Returns an error naming `path` when a folder cannot be made, `write` fails, or the file cannot be synced.
  pub fn stage_with(&mut self, path: &Path, write: impl FnOnce(&mut File) -> IoResult<()>) -> TauriResult {
    let staged: IoResult<StagedFile> = (|| {
      if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
      }

      StagedFile::stage_with(path, &StagedFile::name_sibling(path, "staged")?, write)
    })();

    self
      .staged
      .push(staged.map_err(|error| format!("Cannot stage '{}': {error}", format_path(path)))?);

    Ok(())
  }

  /// Stages what `write` makes as if at `destination`, a file or a folder of files, for a writer that writes by path
  /// itself. `write` is handed a hidden sibling of `destination` to write at instead, which no directory index lists;
  /// each file it leaves there is moved beside its place under `destination` once it returns, and the sibling is
  /// removed however the write ends.
  ///
  /// # Errors
  ///
  /// Returns `write`'s error, or an error naming `destination` when what it wrote cannot be staged.
  pub fn stage_written(&mut self, destination: &Path, write: impl FnOnce(&Path) -> XrfResult) -> TauriResult {
    let staging: PathBuf = StagedFile::name_sibling(destination, "staged")
      .map_err(|error| format!("Cannot stage '{}': {error}", format_path(destination)))?;
    let adopted: TauriResult<Vec<StagedFile>> = write(&staging).map_err(|error| error.to_string()).and_then(|()| {
      Self::adopt(&staging, destination)
        .map_err(|error| format!("Cannot stage '{}': {error}", format_path(destination)))
    });

    // What the writer left that was not adopted: its folders, or everything after a failure.
    let removed: IoResult<()> = if staging.is_dir() {
      fs::remove_dir_all(&staging)
    } else {
      fs::remove_file(&staging)
    };

    if let Err(error) = removed
      && error.kind() != ErrorKind::NotFound
    {
      log::warn!("Could not remove '{}': {error}", format_path(&staging));
    }

    self.staged.extend(adopted?);

    Ok(())
  }

  /// Publishes everything staged through `assets`, refused where what `expected` recorded moved since it was read;
  /// answers every file written, the mounted ones first.
  ///
  /// # Errors
  ///
  /// Returns an error, with no mounted file written, when the commit is refused or fails; a file outside every mount
  /// that cannot be moved into place is reported, and those after it are not written.
  pub fn publish(self, assets: &AssetMountState, expected: Option<&XrayInputRecord>) -> TauriResult<Vec<PathBuf>> {
    assets.write_files(self.staged, expected)
  }

  /// Stages every file a writer left at `staging` beside its place under `destination`, in path order: `staging`
  /// itself where the writer wrote one file, none where it wrote nothing.
  fn adopt(staging: &Path, destination: &Path) -> IoResult<Vec<StagedFile>> {
    if !staging.exists() {
      return Ok(Vec::new());
    }

    if staging.is_file() {
      return Ok(vec![StagedFile::adopt(destination, staging)?]);
    }

    let mut written: Vec<PathBuf> = Vec::new();
    let mut folders: Vec<PathBuf> = vec![staging.to_path_buf()];

    while let Some(folder) = folders.pop() {
      for entry in fs::read_dir(&folder)? {
        let path: PathBuf = entry?.path();

        if path.is_dir() {
          folders.push(path);
        } else {
          written.push(path);
        }
      }
    }

    written.sort();

    written
      .into_iter()
      .map(|path| {
        let target: PathBuf = destination.join(path.strip_prefix(staging).expect("a written file sits below staging"));

        if let Some(parent) = target.parent() {
          fs::create_dir_all(parent)?;
        }

        StagedFile::adopt(&target, &path)
      })
      .collect()
  }
}
