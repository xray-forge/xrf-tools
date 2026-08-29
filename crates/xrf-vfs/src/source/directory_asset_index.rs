use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::source::DirectoryAsset;

#[derive(Debug)]
pub(crate) struct DirectoryAssetIndex {
  root: PathBuf,
  assets: Vec<DirectoryAsset>,
}

impl DirectoryAssetIndex {
  /// Recursively indexes files below `root` in relative-path order.
  ///
  /// Directory paths and symbolic links to directories are not added as assets.
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

    log::debug!("reading directory assets from {}", format_path(root));

    let mut assets: Vec<DirectoryAsset> = Vec::new();

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

      let relative_path = entry
        .path()
        .strip_prefix(root)
        .map_err(|_| XrfError::new_unexpected_error("failed to strip directory asset root"))?
        .to_path_buf();
      assets.push(DirectoryAsset::new(relative_path));
    }

    assets.sort_by(|left, right| left.relative_path().cmp(right.relative_path()));

    log::debug!("read {} directory assets from {}", assets.len(), format_path(root));

    Ok(Self {
      root: root.to_path_buf(),
      assets,
    })
  }

  /// Returns the root from which relative paths are measured.
  pub fn root(&self) -> &Path {
    &self.root
  }

  /// Iterates over all indexed files in relative-path order.
  pub fn assets(&self) -> impl Iterator<Item = &DirectoryAsset> {
    self.assets.iter()
  }

  pub(crate) fn asset(&self, index: usize) -> &DirectoryAsset {
    &self.assets[index]
  }
}
