use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::change::XrayDiskJournal;

/// The files below one physical root, each by its root-relative path.
#[derive(Clone, Debug)]
pub(crate) struct DirectoryAssetIndex {
  root: PathBuf,
  /// Root-relative paths, in relative-path order as walked, then any created since in the order they were added.
  assets: Vec<PathBuf>,
}

impl DirectoryAssetIndex {
  /// Recursively indexes files below `root` in relative-path order, after settling what a commit cut short left there.
  ///
  /// Directory paths, symbolic links to directories, and the hidden siblings a write stages or sets aside beside a
  /// file (`.<name>.xrf-…`) are not added as assets.
  ///
  /// An entry the walk cannot read — a permission-denied subdirectory, a broken link — is warned about and skipped
  /// rather than failing the index. Aborting would discard the whole mount over one unreadable corner of a tree, and the
  /// mount being absent then reads downstream as content that is missing.
  ///
  /// # Errors
  ///
  /// Returns an error when a file path cannot be made relative to `root`, which would mean the walk left its own root.
  pub fn read(root: impl AsRef<Path>) -> XrfResult<Self> {
    let root: &Path = root.as_ref();
    let (index, journals): (Self, Vec<PathBuf>) = Self::walk(root)?;

    if journals.is_empty() {
      return Ok(index);
    }

    for journal in &journals {
      XrayDiskJournal::recover(journal);
    }

    // Once more, since settling a commit moves files back or removes them.
    Ok(Self::walk(root)?.0)
  }

  /// An index of `assets` as a walk of `root` would have found them, for a test about files a host filesystem cannot
  /// hold side by side, such as two names differing only by case on Windows.
  #[cfg(test)]
  pub(crate) fn of(root: impl Into<PathBuf>, assets: &[&str]) -> Self {
    Self {
      root: root.into(),
      assets: assets.iter().map(PathBuf::from).collect(),
    }
  }

  /// Indexes the files below `root`, answering the commit journals found beside them too.
  fn walk(root: &Path) -> XrfResult<(Self, Vec<PathBuf>)> {
    log::debug!("reading directory assets from {}", format_path(root));

    let mut assets: Vec<PathBuf> = Vec::new();
    let mut journals: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(root).follow_links(false) {
      let entry = match entry {
        Ok(entry) => entry,
        Err(error) => {
          log::warn!(
            "Skipping unreadable directory entry under {}: {error}",
            format_path(root)
          );

          continue;
        }
      };

      if !entry.file_type().is_file() {
        continue;
      }

      if StagedFile::is_sibling(entry.file_name()) {
        if entry.file_name().to_str().is_some_and(XrayDiskJournal::is_journal) {
          journals.push(entry.path().to_path_buf());
        }

        continue;
      }

      assets.push(
        entry
          .path()
          .strip_prefix(root)
          .map_err(|_| XrfError::new_unexpected_error("failed to strip directory asset root"))?
          .to_path_buf(),
      );
    }

    assets.sort();

    log::debug!("read {} directory assets from {}", assets.len(), format_path(root));

    Ok((
      Self {
        root: root.to_path_buf(),
        assets,
      },
      journals,
    ))
  }

  /// Returns the root from which relative paths are measured.
  pub fn root(&self) -> &Path {
    &self.root
  }

  /// Iterates over all indexed files' root-relative paths in index order.
  pub fn assets(&self) -> impl Iterator<Item = &Path> {
    self.assets.iter().map(PathBuf::as_path)
  }

  /// The root-relative path of the file at `index`.
  pub(crate) fn asset(&self, index: usize) -> &Path {
    &self.assets[index]
  }

  /// The index of a file below the root, added where the index does not hold it yet.
  pub(crate) fn find_or_push(&mut self, relative_path: &Path) -> usize {
    match self.assets.iter().position(|asset| asset == relative_path) {
      Some(index) => index,
      None => self.push(relative_path.to_path_buf()),
    }
  }

  /// Adds a file created below the root since it was walked, answering its index.
  pub(crate) fn push(&mut self, relative_path: PathBuf) -> usize {
    self.assets.push(relative_path);

    self.assets.len() - 1
  }
}
