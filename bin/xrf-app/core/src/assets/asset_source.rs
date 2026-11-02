use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayRoots, XraySearch};

use crate::assets::AssetMountState;

/// Where a document is read from: a file on disk, read by its path and mounted nowhere, or an asset of the roots, loose
/// or archived, read by its logical path.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AssetSource {
  /// A file on disk, named by its filesystem path.
  File { path: String },
  /// An asset of the roots, named by its logical path such as `meshes\actors\stalker.ogf`.
  Asset { logical_path: String },
}

impl AssetSource {
  /// What to call the source on screen and in the log: the path or the logical path, as given.
  pub fn label(&self) -> &str {
    match self {
      Self::File { path } => path,
      Self::Asset { logical_path } => logical_path,
    }
  }

  /// The source's filesystem path, for a file; an asset has none until the roots locate it.
  pub fn physical_path(&self) -> Option<&Path> {
    match self {
      Self::File { path } => Some(Path::new(path)),
      Self::Asset { .. } => None,
    }
  }

  /// The bytes the source names: a file read from disk, an asset through `search`.
  ///
  /// # Errors
  ///
  /// Returns an error where the file cannot be read, or the asset resolves to nothing or cannot be read.
  pub fn read(&self, search: &XraySearch) -> XrfResult<Vec<u8>> {
    match self {
      Self::File { path } => Ok(fs::read(path)?),
      Self::Asset { logical_path } => search.read(logical_path),
    }
  }

  /// The bytes the source names, searching `roots` only for an asset: a file is read from disk, and nothing is mounted
  /// for it.
  ///
  /// # Errors
  ///
  /// Returns an error where the file cannot be read, the roots cannot be mounted, or the asset resolves to nothing or
  /// cannot be read.
  pub fn read_in(&self, assets: &AssetMountState, roots: &XrayRoots) -> XrfResult<Vec<u8>> {
    match self {
      Self::File { path } => Ok(fs::read(path)?),
      Self::Asset { logical_path } => assets
        .search(roots)
        .map_err(XrfError::new_asset_error)?
        .read(logical_path),
    }
  }
}
