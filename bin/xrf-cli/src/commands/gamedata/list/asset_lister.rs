use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_utils::format_path;
use xrf_vfs::{XrayAsset, XrayPathCollision, XrayRoots, XraySearch, XraySearchPlan, XraySkippedMount, XraySourceKind};

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
    let search: XraySearch = self.plan()?.open()?.with_optional_prefix(self.prefix.as_deref())?;
    let entries: Vec<XrayAsset> = search.list_files();
    let shadowed: Vec<XrayAsset> = if self.is_shadowed_included {
      Self::shadowed(&search)
    } else {
      Vec::new()
    };

    Ok(AssetListing {
      collisions: search.list_collisions(),
      duration: started.elapsed(),
      entries,
      mounts: search
        .list_sources()
        .iter()
        .map(|source| {
          format!(
            "{:<9} {} ({})",
            format!("{:?}", source.mount.get_kind()),
            format_path(source.mount.get_source().get_root_path()),
            source.mount.get_label()
          )
        })
        .collect(),
      origin: self.roots.describe(),
      shadowed,
      skipped: search.list_skipped_sources().into_iter().cloned().collect(),
    })
  }

  /// What the listing mounts: the roots with the ignored prefixes left out, and only their loose trees where asked,
  /// so no volume the listing never shows is opened.
  fn plan(&self) -> XrfResult<XraySearchPlan> {
    let plan: XraySearchPlan = self.roots.to_search_plan()?.ignoring(&self.ignored)?;

    Ok(if self.is_loose_only {
      plan.of_kind(XraySourceKind::Directory)
    } else {
      plan
    })
  }

  /// Every copy a higher-priority one hides, ordered by engine identity and then by search order.
  fn shadowed(search: &XraySearch) -> Vec<XrayAsset> {
    search
      .list_shadowing_entries()
      .into_iter()
      .flat_map(|entry| entry.shadowed.into_iter().map(|copy| copy.asset))
      .collect()
  }
}
