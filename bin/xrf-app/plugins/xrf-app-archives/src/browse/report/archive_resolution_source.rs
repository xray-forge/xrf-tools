use std::path::PathBuf;

use serde::Serialize;
use xrf_vfs::{XrayAssetSource, XraySearchedSource, XraySourceKind};

use crate::browse::report::ArchiveResolutionVolume;

/// One source the open subject searches, and everything that says why it is searched where it is.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveResolutionSource {
  /// Where the source lives: a loose tree's root, or the directory holding a volume set.
  pub path: PathBuf,
  /// Short name for the source — the directory or volume-set name.
  pub label: String,
  /// Whether the source is a loose tree or a set of volumes.
  pub kind: XraySourceKind,
  /// How the mount plan described it: an `fsgame.ltx` alias such as `$game_data$`, or `root` or `volumes` for a path
  /// named directly. Absent for a source no plan named.
  pub origin: Option<String>,
  /// The root whose declaration reached this source, as the open named it.
  pub step: String,
  /// Logical base the source mounts at; empty for a complete root.
  pub base: String,
  /// Engine identities the source answers for, before anything searched ahead of it shadows one.
  pub entries: usize,
  /// Volumes merged behind this source, in the order a lookup reaches them. Empty for a loose tree.
  pub volumes: Vec<ArchiveResolutionVolume>,
}

impl From<XraySearchedSource<'_>> for ArchiveResolutionSource {
  fn from(source: XraySearchedSource<'_>) -> Self {
    let mounted: &dyn XrayAssetSource = source.mount.get_source();

    Self {
      base: source.mount.get_base().to_string(),
      entries: mounted.count_entries(),
      kind: mounted.get_kind(),
      label: mounted.get_label().to_string(),
      origin: source.mount.get_origin().map(str::to_string),
      path: mounted.get_root_path().to_path_buf(),
      step: source.step.to_string(),
      // Reversed for the same reason a set's own report is: the later volume of a merge is the one a lookup answers
      // out of, so the volume listed first is the one asked first.
      volumes: mounted.list_volumes().iter().rev().map(Into::into).collect(),
    }
  }
}
