use std::collections::HashSet;
use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_utils::format_path;
use xrf_vfs::{
  XrayAsset, XrayAssetContainer, XrayLogicalPath, XrayLookupScope, XrayPathCollision, XrayRoots, XraySkippedMount,
  XraySourceKind, XrayVfs,
};

/// Resolved assets and source metadata for one listing.
pub struct AssetListing {
  /// How the path was interpreted when planning its mounts.
  pub origin: String,
  /// One line per mount, in search order.
  pub mounts: Vec<String>,
  /// Winning entries, one per logical path.
  pub entries: Vec<XrayAsset>,
  /// Entries shadowed by a higher-priority mount, absent unless asked for.
  pub shadowed: Vec<XrayAsset>,
  /// Files a mount holds but cannot reach, because another file in it claims their identity.
  pub collisions: Vec<XrayPathCollision>,
  /// Declared sources that could not be opened, so the listing does not cover them.
  pub skipped: Vec<XraySkippedMount>,
  /// Time spent planning, mounting, and enumerating.
  pub duration: Duration,
}

/// Builds a VFS from an installation or bare root and lists its resolved assets.
///
/// Entries identify their physical containers. Optional shadowed entries expose lower-priority copies of winning paths.
pub struct AssetLister {
  roots: XrayRoots,
  prefix: Option<String>,
  ignored: Vec<String>,
  is_loose_only: bool,
  is_shadowed_included: bool,
}

impl AssetLister {
  /// Lists what roots resolves.
  ///
  /// The roots carries a mode per root, so what used to be one path and one mode is now several of
  /// each — which is what lets a listing show a loose tree shadowing the installation behind it.
  pub fn new(roots: &XrayRoots) -> Self {
    Self {
      is_loose_only: false,
      is_shadowed_included: false,
      ignored: Vec::new(),
      prefix: None,
      roots: roots.clone(),
    }
  }

  /// Logical prefixes the directory mounts omit, as `verify-gamedata --ignore` means them.
  pub fn with_ignored(mut self, ignored: &[String]) -> Self {
    self.ignored = ignored.to_vec();

    self
  }

  /// Narrows the listing to one logical subtree, such as `configs` or `textures\wpn`.
  pub fn with_prefix(mut self, prefix: Option<&str>) -> Self {
    self.prefix = prefix.map(ToString::to_string);

    self
  }

  /// Restricts the listing to directory mounts, excluding archives.
  pub fn with_loose_only(mut self, is_loose_only: bool) -> Self {
    self.is_loose_only = is_loose_only;

    self
  }

  /// Includes entries hidden by higher-priority mounts.
  pub fn with_shadowed(mut self, is_shadowed_included: bool) -> Self {
    self.is_shadowed_included = is_shadowed_included;

    self
  }

  /// Plans and enumerates the path's asset sources.
  ///
  /// # Errors
  ///
  /// Returns an error when installation metadata cannot be read, decoded, or parsed, or when the requested prefix is not
  /// a valid X-Ray logical path.
  pub fn run(&self) -> XrfResult<AssetListing> {
    let started: Instant = Instant::now();
    let vfs: XrayVfs = XrayVfs::from_plan(&self.roots.to_mount_plan()?.ignoring(&self.ignored)?)?;
    let scope: XrayLookupScope = self.scope()?;
    let entries: Vec<XrayAsset> = vfs.scoped(&scope).list_entries();
    let shadowed: Vec<XrayAsset> = if self.is_shadowed_included {
      Self::shadowed(&vfs, &scope, &entries)
    } else {
      Vec::new()
    };

    Ok(AssetListing {
      collisions: vfs.scoped(&scope).list_collisions(),
      duration: started.elapsed(),
      entries,
      mounts: vfs
        .scoped(&scope)
        .list_mounts()
        .map(|mount| {
          format!(
            "{:<9} {} ({})",
            format!("{:?}", mount.get_kind()),
            format_path(mount.get_source().get_root_path()),
            mount.get_label()
          )
        })
        .collect(),
      origin: self.roots.describe(),
      shadowed,
      skipped: vfs.get_skipped_mounts().to_vec(),
    })
  }

  fn scope(&self) -> XrfResult<XrayLookupScope> {
    let scope: XrayLookupScope = if self.is_loose_only {
      XrayLookupScope::of_kind(XraySourceKind::Directory)
    } else {
      XrayLookupScope::all()
    };

    match self.prefix.as_deref() {
      Some(prefix) => scope.with_prefix(prefix),
      None => Ok(scope),
    }
  }

  /// Returns entries hidden by a higher-priority mount.
  ///
  /// The result removes winning path and container pairs from the complete enumeration. Indexed rather than scanned: an
  /// installation enumerates ~48,000 entries, and a linear search per entry made this quadratic.
  fn shadowed(vfs: &XrayVfs, scope: &XrayLookupScope, winners: &[XrayAsset]) -> Vec<XrayAsset> {
    let mut shadowed: Vec<XrayAsset> = vfs.scoped(scope).list_entries_all();

    let winning: HashSet<(&XrayLogicalPath, &XrayAssetContainer)> = winners
      .iter()
      .map(|winner| (winner.get_logical_path(), winner.get_container()))
      .collect();

    shadowed.retain(|entry| !winning.contains(&(entry.get_logical_path(), entry.get_container())));

    shadowed
  }
}
