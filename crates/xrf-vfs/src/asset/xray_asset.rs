use std::path::{Path, PathBuf};

use serde::Serialize;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::XrayAssetType;
use crate::path::XrayLogicalPath;

/// The physical container of a located asset.
///
/// Separate variants prevent callers from treating an archived entry as a loose file with a usable filesystem path.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
// `rename_all_fields` keeps struct-variant fields camel-cased alongside the variants.
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum XrayAssetContainer {
  /// A loose file, preserving its root so consumers can identify the winning overlay.
  Directory { root: PathBuf, relative_path: PathBuf },
  /// An entry inside the archive volume set at `path`.
  Archive { path: PathBuf },
}

/// One asset a mount resolved: its engine identity plus the container it came out of.
///
/// Owned rather than borrowed, so it can be stored, sorted or sent over IPC — which is what an editor that mounts and
/// writes needs, and why nothing borrowed reaches past this crate.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayAsset {
  /// Lower-case, backslash-separated engine identity, including the mount's logical base.
  logical_path: XrayLogicalPath,
  /// Physical container reported by the source that resolved the asset.
  container: XrayAssetContainer,
}

impl XrayAsset {
  /// Creates a location from an engine path and a source-reported container.
  ///
  /// The caller is responsible for passing the normalized logical path returned by the VFS. A loose container's
  /// `relative_path` is joined to its `root` only when [`Self::to_physical_path`] is asked for; the logical path stays the
  /// engine identity used for lookups and IPC.
  pub fn new(logical_path: XrayLogicalPath, container: XrayAssetContainer) -> Self {
    Self {
      container,
      logical_path,
    }
  }

  /// Returns the normalized X-Ray path, including any mount base.
  pub fn get_logical_path(&self) -> &XrayLogicalPath {
    &self.logical_path
  }

  /// Returns the physical container that supplied this location.
  pub fn get_container(&self) -> &XrayAssetContainer {
    &self.container
  }

  /// Returns the kind this asset's extension identifies, when it is one the tools recognize.
  ///
  /// Derived from the logical path rather than stored, because the path is the only evidence: a container says where the
  /// bytes are, not what they mean.
  pub fn get_asset_type(&self) -> Option<XrayAssetType> {
    XrayAssetType::from_logical_path(self.logical_path.as_str())
  }

  /// Whether this asset's extension identifies the requested kind.
  pub fn is_type(&self, asset_type: XrayAssetType) -> bool {
    self.get_asset_type() == Some(asset_type)
  }

  /// Returns the containing tree for a loose asset, or `None` for an archived asset.
  pub fn get_root(&self) -> Option<&Path> {
    match &self.container {
      XrayAssetContainer::Directory { root, .. } => Some(root),
      XrayAssetContainer::Archive { .. } => None,
    }
  }

  /// Returns a readable filesystem path for a loose asset.
  ///
  /// Archived assets return `None`; callers that support both containers should read through [`crate::XrayVfs`].
  pub fn to_physical_path(&self) -> Option<PathBuf> {
    match &self.container {
      XrayAssetContainer::Directory { relative_path, root } => Some(root.join(relative_path)),
      XrayAssetContainer::Archive { .. } => None,
    }
  }

  /// Returns the path an in-place edit would write to, refusing an archived winner by name.
  ///
  /// # Errors
  ///
  /// Returns an asset error when this asset is served out of an archive.
  pub fn to_writable_path(&self) -> XrfResult<PathBuf> {
    require_writable_path(self.logical_path.as_str(), self.to_physical_path())
  }

  /// Describes the containing tree or archive volume set for display.
  pub fn format_container(&self) -> String {
    match &self.container {
      XrayAssetContainer::Directory { root, .. } => root.display().to_string(),
      XrayAssetContainer::Archive { path } => format!("{} (archive)", format_path(path)),
    }
  }
}

/// The host path an in-place edit would write to, or one refusal for the case there is none.
///
/// Every editing domain reaches this same wall — an asset whose winner is packed has no file to
/// replace — and each was inventing its own wording and its own error kind for it. A surface deciding
/// whether a save failed *because the source is archived* would otherwise have to match three strings
/// that nothing keeps in step.
///
/// Takes the two parts rather than an [`XrayAsset`] because the refusal usually happens later than the
/// listing: a project holds the logical path and what it resolved to, and the asset itself is gone.
///
/// # Errors
///
/// Returns an asset error naming the file when `physical_path` is absent.
pub fn require_writable_path(logical_path: &str, physical_path: Option<PathBuf>) -> XrfResult<PathBuf> {
  physical_path.ok_or_else(|| {
    XrfError::new_asset_error(format!(
      "Cannot write '{logical_path}': it has no file on disk, being read out of an archive"
    ))
  })
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use crate::{XrayAsset, XrayAssetContainer, XrayLogicalPath};

  #[test]
  fn a_directory_asset_answers_a_physical_path() {
    // Host paths are built from components, never from a `\`-joined literal: `PathBuf::join` inserts the platform separator,
    // so a Windows-shaped literal compares unequal to a joined path on Linux while passing here.
    let root: PathBuf = PathBuf::from("gamedata");
    let relative: PathBuf = Path::new("textures").join("wpn").join("wpn_ak74.dds");
    let asset: XrayAsset = XrayAsset::new(
      XrayLogicalPath::new("textures\\wpn\\wpn_ak74.dds").expect("valid logical path"),
      XrayAssetContainer::Directory {
        relative_path: relative.clone(),
        root: root.clone(),
      },
    );

    assert_eq!(asset.get_root(), Some(root.as_path()));
    assert_eq!(
      asset.to_physical_path(),
      Some(root.join("textures").join("wpn").join("wpn_ak74.dds"))
    );
    assert_eq!(
      asset.get_logical_path().as_str(),
      "textures\\wpn\\wpn_ak74.dds",
      "the engine identity keeps backslashes on every platform, unlike the host path beside it"
    );
  }

  #[test]
  fn an_archived_asset_answers_no_physical_path_rather_than_a_plausible_one() {
    // Archive entries have no physical path; joining the volume directory to the logical path would invent one.
    let location: XrayAsset = XrayAsset::new(
      XrayLogicalPath::new("textures\\wpn\\wpn_ak74.dds").expect("valid logical path"),
      XrayAssetContainer::Archive {
        path: Path::new("anomaly").join("db").join("textures"),
      },
    );

    assert_eq!(location.get_root(), None);
    assert_eq!(location.to_physical_path(), None);
    assert!(matches!(location.get_container(), XrayAssetContainer::Archive { .. }));
    assert!(location.format_container().ends_with("(archive)"));
  }
}
