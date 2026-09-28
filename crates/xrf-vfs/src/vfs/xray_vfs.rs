use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::cache::{XrayAssetCache, XrayCachePolicy};
use crate::path::{XrayLogicalPath, normalize};
use crate::source::XrayDirectorySource;
use crate::trace::XrayReadTrace;
use crate::vfs::{XrayDirectoryListing, XrayMountedEntry, XrayShadowedCopy, XrayShadowingEntry};
use crate::{
  XrayAsset, XrayAssetContainer, XrayAssetRules, XrayAssetSource, XrayAssetType, XrayLookupScope, XrayMount,
  XrayMountId, XrayPathCollision, XraySkippedMount, XraySourceKind,
};

/// The engine's view of assets: several mounted sources, searched in order, first hit wins.
#[derive(Debug, Default)]
pub struct XrayVfs {
  mounts: Vec<XrayMount>,
  /// Parsed assets this world retains, governed by its own policy and empty unless a caller sets one.
  cache: XrayAssetCache,
  /// Per-path account of what was physically read, absent unless a caller asked to be told; one account however many
  /// forks read.
  trace: Option<Arc<XrayReadTrace>>,
  skipped: Vec<XraySkippedMount>,
  /// Where in `skipped` each planned source that failed to open sits, by its path and then its kind, so a failure is
  /// settled rather than retried and a lookup borrows the path it is asked about.
  failed: HashMap<PathBuf, Vec<(XraySourceKind, usize)>>,
  /// Paths already mounted from a plan, so a later plan naming the same source reuses it.
  planned: HashMap<PathBuf, XrayMountId>,
}

impl XrayVfs {
  /// Creates an empty VFS with no searchable mounts.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets what this world may retain after parsing an asset.
  pub fn with_cache_policy(mut self, policy: XrayCachePolicy) -> Self {
    self.cache = XrayAssetCache::new(policy);

    self
  }

  /// The parsed assets this world is holding.
  pub fn get_cache(&self) -> &XrayAssetCache {
    &self.cache
  }

  /// Accounts for every physical read this world performs from here on.
  pub fn with_read_trace(mut self) -> Self {
    self.trace = Some(Arc::default());

    self
  }

  /// What this world has read, or `None` when it was never asked to account for it.
  pub fn get_read_trace(&self) -> Option<&XrayReadTrace> {
    self.trace.as_deref()
  }

  /// A world over the same mounts, sharing each source and the index read for it rather than reading them again, to
  /// mount more into while this one goes on serving reads.
  ///
  /// It keeps this world's cache policy, read trace and settled failures, and starts with nothing cached, as a mount
  /// into it would leave it anyway.
  pub fn fork(&self) -> Self {
    Self {
      cache: XrayAssetCache::new(self.cache.get_policy().clone()),
      failed: self.failed.clone(),
      mounts: self.mounts.clone(),
      planned: self.planned.clone(),
      skipped: self.skipped.clone(),
      trace: self.trace.clone(),
    }
  }

  /// Reads through a mount, accounting for the read when this world is tracing.
  fn read_from_mount(&self, mount: &XrayMount, source_path: &str, logical_path: &str) -> XrfResult<Vec<u8>> {
    let bytes: Vec<u8> = mount.get_source().read(source_path)?;

    if let Some(trace) = &self.trace {
      trace.record(logical_path, bytes.len() as u64);
    }

    Ok(bytes)
  }

  /// Sources a plan named that could not be opened.
  pub fn get_skipped_mounts(&self) -> &[XraySkippedMount] {
    &self.skipped
  }

  /// Records a source that a plan named but could not open, once however many plans name it.
  pub(crate) fn record_skipped(&mut self, kind: XraySourceKind, skipped: XraySkippedMount) {
    if self.skipped_mount(&skipped.path, kind).is_some() {
      return;
    }

    let index: usize = self.skipped.len();

    self.failed.entry(skipped.path.clone()).or_default().push((kind, index));
    self.skipped.push(skipped);
  }

  /// The recorded failure of a planned source, which settles it until [`Self::forget_skipped_mounts_of`] forgets it.
  pub(crate) fn skipped_mount(&self, path: &Path, kind: XraySourceKind) -> Option<&XraySkippedMount> {
    self
      .failed
      .get(path)?
      .iter()
      .find(|(failed, _)| *failed == kind)
      .and_then(|(_, index)| self.skipped.get(*index))
  }

  /// Forgets the failures of the planned sources `is_forgotten` picks by path and kind, keeping the rest in the order
  /// they were recorded, so the next plan naming one of them tries it again.
  pub(crate) fn forget_skipped(&mut self, is_forgotten: impl Fn(&Path, XraySourceKind) -> bool) {
    let mut kinds: Vec<Option<XraySourceKind>> = vec![None; self.skipped.len()];

    for (kind, index) in self.failed.values().flatten() {
      if let Some(slot) = kinds.get_mut(*index) {
        *slot = Some(*kind);
      }
    }

    let recorded: Vec<XraySkippedMount> = std::mem::take(&mut self.skipped);

    self.failed.clear();

    for (skipped, kind) in recorded.into_iter().zip(kinds) {
      if let Some(kind) = kind.filter(|kind| !is_forgotten(&skipped.path, *kind)) {
        self.record_skipped(kind, skipped);
      }
    }
  }

  /// The mount already opened from a planned path, when its kind still matches.
  pub(crate) fn planned_mount(&self, path: &Path, kind: XraySourceKind) -> Option<XrayMountId> {
    self
      .planned
      .get(path)
      .copied()
      .filter(|id| self.mounts.get(id.0).is_some_and(|mount| mount.get_kind() == kind))
  }

  /// Remembers which mount a planned path produced.
  pub(crate) fn record_planned(&mut self, path: PathBuf, id: XrayMountId) {
    self.planned.insert(path, id);
  }

  /// Records how a plan described one of its mounts, for a report explaining why the mount is searched.
  pub(crate) fn record_origin(&mut self, id: XrayMountId, origin: &str) {
    if let Some(mount) = self.mounts.get_mut(id.0) {
      mount.set_origin(origin);
    }
  }

  /// Appends a source at a logical base with lower priority than existing mounts.
  ///
  /// # Errors
  ///
  /// Returns an error when a non-empty base is not a valid X-Ray logical path.
  pub fn mount(&mut self, base: &str, source: Box<dyn XrayAssetSource>) -> XrfResult<XrayMountId> {
    let id: XrayMountId = XrayMountId(self.mounts.len());

    log::info!("Mounting {} at base '{base}' as {id:?}", source.get_label());

    self.mounts.push(XrayMount::new(id, base, source)?);

    // A new mount can win any path, so nothing retained can be trusted to describe this world any more. Per-path
    // reasoning is not available here: the mount is indexed, but which paths it now shadows is exactly the question a
    // resolve answers, and answering it for every retained entry costs more than parsing them again.
    self.cache.clear();

    Ok(id)
  }

  /// Mounts a directory once, reusing the existing mount for the same root.
  ///
  /// # Errors
  ///
  /// Returns an error when the base is invalid or the directory cannot be indexed.
  pub fn mount_directory(&mut self, base: &str, root: impl AsRef<Path>) -> XrfResult<XrayMountId> {
    let root: &Path = root.as_ref();

    if let Some(mount) = self.find_directory_mount(root) {
      return Ok(mount);
    }

    self.mount(base, Box::new(XrayDirectorySource::read(root)?))
  }

  /// Returns the mount already covering a directory root.
  pub fn find_directory_mount(&self, root: &Path) -> Option<XrayMountId> {
    self
      .mounts
      .iter()
      .find(|mount| mount.get_kind() == XraySourceKind::Directory && mount.get_source().get_root_path() == root)
      .map(XrayMount::get_id)
  }

  /// Returns mounts in search priority order.
  pub fn get_mounts(&self) -> &[XrayMount] {
    &self.mounts
  }

  /// Returns whether no source has been mounted.
  pub fn is_empty(&self) -> bool {
    self.mounts.is_empty()
  }

  /// Iterates over mounts selected by a scope, preserving priority order.
  pub(crate) fn mounts_in(&self, scope: &XrayLookupScope) -> impl Iterator<Item = &XrayMount> {
    self.mounts.iter().filter(move |mount| scope.includes(mount))
  }

  /// The winning location for a logical path, or `None` when no mount holds it.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not a valid X-Ray logical path. Absence is `Ok(None)`, not an error.
  pub fn find(&self, logical_path: &str) -> XrfResult<Option<XrayAsset>> {
    self.find_in(&XrayLookupScope::default(), logical_path)
  }

  pub(crate) fn find_in(&self, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Option<XrayAsset>> {
    let logical_path: Cow<str> = normalize(logical_path)?;

    Ok(
      self
        .get_winner_in_scope(scope, &logical_path)
        .and_then(|(mount, source_path)| Self::locate_at(mount, &logical_path, source_path)),
    )
  }

  /// Every mount holding a logical path, winner first.
  ///
  /// Includes shadowed copies for override auditing.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not a valid X-Ray logical path.
  pub fn find_all(&self, logical_path: &str) -> XrfResult<Vec<XrayAsset>> {
    self.find_all_in(&XrayLookupScope::default(), logical_path)
  }

  pub(crate) fn find_all_in(&self, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Vec<XrayAsset>> {
    let logical_path: Cow<str> = normalize(logical_path)?;

    if !Self::get_within_prefix(scope, &logical_path) {
      return Ok(Vec::new());
    }

    Ok(
      self
        .mounts_in(scope)
        .filter_map(|mount| Self::locate_in(mount, &logical_path))
        .collect(),
    )
  }

  /// Reads bytes from the winning entry for a logical path.
  ///
  /// Prefer [`Self::read_asset`] when the asset has already been resolved.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when nothing holds the path, an invalid-path error when it is not a valid X-Ray
  /// logical path, or the source's own error when the bytes cannot be read.
  pub fn read_bytes(&self, logical_path: &str) -> XrfResult<Vec<u8>> {
    self.read_in(&XrayLookupScope::default(), logical_path)
  }

  pub(crate) fn read_in(&self, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Vec<u8>> {
    let logical_path: Cow<str> = normalize(logical_path)?;

    match self.get_winner_in_scope(scope, &logical_path) {
      Some((mount, source_path)) => self.read_from_mount(mount, source_path, &logical_path),
      // Absence is `NotFound` throughout this crate, so a consumer can tell "the asset is not here" from "the source
      // holding it failed" without reading the message.
      None => Err(XrfError::new_not_found_error(format!(
        "no asset '{logical_path}' in scope across {} mount(s)",
        self.mounts_in(scope).count()
      ))),
    }
  }

  /// Reads the bytes of an asset this VFS already resolved.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when no mount in this VFS holds the asset's container — most often because the asset came
  /// from a different VFS, or its mount has since been replaced.
  pub fn read_asset_bytes(&self, asset: &XrayAsset) -> XrfResult<Vec<u8>> {
    let container_root: &Path = match asset.get_container() {
      XrayAssetContainer::Directory { root, .. } => root,
      XrayAssetContainer::Archive { path } => path,
    };

    // A loose container names its mount's root, but an archived one names the volume the entry sits in, which is a
    // file inside the mount rather than the mount itself. Matching the volume is also what tells two mounts planned
    // from single volumes of one directory apart, where a root match alone would hand both to the first.
    let Some(mount) = self.mounts.iter().find(|mount| {
      let source: &dyn XrayAssetSource = mount.get_source();

      source.get_root_path() == container_root
        || source.list_volumes().iter().any(|volume| volume.path == container_root)
    }) else {
      return Err(XrfError::new_not_found_error(format!(
        "cannot read '{}': no mount in this VFS holds {}",
        asset.get_logical_path(),
        format_path(container_root)
      )));
    };

    let Some(source_path) = mount.to_source_path(asset.get_logical_path().as_str()) else {
      return Err(XrfError::new_not_found_error(format!(
        "cannot read '{}': it falls outside the base of the mount holding it",
        asset.get_logical_path()
      )));
    };

    self.read_from_mount(mount, source_path, asset.get_logical_path().as_str())
  }

  /// Size in bytes of the winning entry, without reading it.
  ///
  /// Answers `None` both for an absent asset and for a path that is not a valid logical path — a size gate has nothing
  /// useful to do with the difference, and every caller would discard it.
  pub fn read_size(&self, logical_path: &str) -> Option<u64> {
    self.read_size_in(&XrayLookupScope::default(), logical_path)
  }

  pub(crate) fn read_size_in(&self, scope: &XrayLookupScope, logical_path: &str) -> Option<u64> {
    let logical_path: Cow<str> = normalize(logical_path).ok()?;
    let (mount, source_path) = self.get_winner_in_scope(scope, &logical_path)?;

    mount.get_source().get_size(source_path)
  }

  /// CRC32 of the winning entry's payload, when its source already knows it without reading anything.
  ///
  /// `None` covers an absent asset, an unreadable logical path, and a source that simply does not record one; a
  /// caller that must tell those apart has [`Self::find`] for the first two.
  pub fn read_recorded_crc(&self, logical_path: &str) -> Option<u32> {
    self.read_recorded_crc_in(&XrayLookupScope::default(), logical_path)
  }

  pub(crate) fn read_recorded_crc_in(&self, scope: &XrayLookupScope, logical_path: &str) -> Option<u32> {
    let logical_path: Cow<str> = normalize(logical_path).ok()?;
    let (mount, source_path) = self.get_winner_in_scope(scope, &logical_path)?;

    mount.get_source().get_recorded_crc(source_path)
  }

  /// Returns winning entries, one per logical path, ordered by that path.
  pub fn list_entries(&self) -> Vec<XrayAsset> {
    self.list_entries_in(&XrayLookupScope::default())
  }

  pub(crate) fn list_entries_in(&self, scope: &XrayLookupScope) -> Vec<XrayAsset> {
    let mut located: Vec<XrayAsset> = self.list_entries_all_in(scope);

    // `list_entries_all_in` sorts stably by logical path after collecting in mount order, so within one path the
    // highest-priority mount comes first — which is exactly the entry `dedup_by` keeps. Deduping the sort we already
    // need costs nothing, where a seen-set cost a hash and an owned copy of every path enumerated.
    located.dedup_by(|first, second| first.get_logical_path() == second.get_logical_path());

    located
  }

  /// Returns winning entries whose extension identifies one kind.
  pub fn list_entries_of_type(&self, asset_type: XrayAssetType) -> Vec<XrayAsset> {
    self.list_entries_of_type_in(&XrayLookupScope::default(), asset_type)
  }

  pub(crate) fn list_entries_of_type_in(&self, scope: &XrayLookupScope, asset_type: XrayAssetType) -> Vec<XrayAsset> {
    self
      .list_entries_in(scope)
      .into_iter()
      .filter(|entry| entry.is_type(asset_type))
      .collect()
  }

  /// Returns winning entries whose logical path ends with `suffix` on a component boundary.
  ///
  /// # Errors
  ///
  /// Returns an error when `suffix` is not a valid X-Ray logical path fragment.
  pub fn list_entries_with_suffix(&self, suffix: &str) -> XrfResult<Vec<XrayAsset>> {
    self.list_entries_with_suffix_in(&XrayLookupScope::default(), suffix)
  }

  pub(crate) fn list_entries_with_suffix_in(&self, scope: &XrayLookupScope, suffix: &str) -> XrfResult<Vec<XrayAsset>> {
    let suffix: Cow<str> = normalize(suffix)?;

    Ok(
      self
        .list_entries_in(scope)
        .into_iter()
        .filter(|entry| {
          entry
            .get_logical_path()
            .as_str()
            .strip_suffix(suffix.as_ref())
            .is_some_and(|rest| rest.is_empty() || rest.ends_with('\\'))
        })
        .collect(),
    )
  }

  /// Files any mount holds but cannot reach, because another file in the same mount claims their identity.
  pub fn list_collisions(&self) -> Vec<XrayPathCollision> {
    self.list_collisions_in(&XrayLookupScope::default())
  }

  pub(crate) fn list_collisions_in(&self, scope: &XrayLookupScope) -> Vec<XrayPathCollision> {
    self
      .mounts_in(scope)
      .flat_map(|mount| mount.get_source().get_collisions().iter().cloned())
      .collect()
  }

  /// Returns what sits directly inside one logical directory, as a browser or a tree view needs it.
  ///
  /// # Errors
  ///
  /// Returns an error when `directory` is not a valid X-Ray logical path. An empty `directory` lists the logical root.
  pub fn list_children(&self, directory: &str) -> XrfResult<XrayDirectoryListing> {
    self.list_children_in(&XrayLookupScope::default(), directory)
  }

  pub(crate) fn list_children_in(&self, scope: &XrayLookupScope, directory: &str) -> XrfResult<XrayDirectoryListing> {
    let directory: Cow<str> = if directory.is_empty() {
      Cow::Borrowed("")
    } else {
      normalize(directory)?
    };

    let Some(scope) = Self::get_listing_scope(scope, &directory)? else {
      return Ok(XrayDirectoryListing::default());
    };

    let mut listing: XrayDirectoryListing = Default::default();
    let mut directories: HashSet<String> = HashSet::new();

    for entry in self.list_entries_in(&scope) {
      let Some(remainder) = Self::remainder_under(entry.get_logical_path().as_str(), &directory) else {
        continue;
      };

      match remainder.split_once('\\') {
        Some((child, _)) => {
          if directories.insert(child.to_string()) {
            listing.directories.push(child.to_string());
          }
        }
        None => listing.files.push(entry),
      }
    }

    listing.directories.sort();
    listing
      .files
      .sort_by(|a, b| a.get_logical_path().cmp(b.get_logical_path()));

    Ok(listing)
  }

  /// The scope a listing runs under: the narrower of the view's subtree and the directory asked for.
  ///
  /// `directory` must already be normalized, and is empty for the logical root.
  fn get_listing_scope(scope: &XrayLookupScope, directory: &str) -> XrfResult<Option<XrayLookupScope>> {
    let Some(prefix) = scope.get_prefix() else {
      return Ok(Some(if directory.is_empty() {
        scope.clone()
      } else {
        scope.clone().with_prefix(directory)?
      }));
    };

    // The view is already at or below the directory, so its own prefix is the narrower of the two.
    if directory.is_empty() || crate::path::is_component_prefix(prefix, directory) {
      return Ok(Some(scope.clone()));
    }

    if crate::path::is_component_prefix(directory, prefix) {
      return Ok(Some(scope.clone().with_prefix(directory)?));
    }

    Ok(None)
  }

  /// The part of a logical path below `directory`, or `None` when it does not sit under it.
  fn remainder_under<'a>(logical_path: &'a str, directory: &str) -> Option<&'a str> {
    if directory.is_empty() {
      return Some(logical_path);
    }

    logical_path
      .strip_prefix(directory)
      .and_then(|rest| rest.strip_prefix('\\'))
  }

  /// Returns every entry, including shadowed copies, ordered by logical path.
  ///
  /// Copies of one path stay in mount priority order, so the winner precedes the entries it shadows.
  pub fn list_entries_all(&self) -> Vec<XrayAsset> {
    self.list_entries_all_in(&XrayLookupScope::default())
  }

  pub(crate) fn list_entries_all_in(&self, scope: &XrayLookupScope) -> Vec<XrayAsset> {
    let mut located: Vec<XrayAsset> = Vec::new();

    self.for_each_copy(scope, |_, _, asset, _| located.push(asset));

    // Stable, so copies of one logical path keep the mount order they were enumerated in.
    located.sort_by(|first, second| first.get_logical_path().cmp(second.get_logical_path()));

    located
  }

  /// Visits every copy every mount in scope holds: the one each mount answers with, then the ones it holds behind it.
  fn for_each_copy(&self, scope: &XrayLookupScope, mut visit: impl FnMut(&XrayMount, &str, XrayAsset, Option<u64>)) {
    for mount in self.mounts_in(scope) {
      let Some(source_prefix) = mount.to_source_prefix(scope.get_prefix()) else {
        continue;
      };

      // Grouped once per mount rather than looked up per entry, and never built at all by a source that hides
      // nothing - every loose tree, and every volume set no patch overrides.
      let hidden: HashMap<&str, Vec<&XrayShadowedCopy>> = Self::group_hidden(mount.get_source());

      for source_path in mount.get_source().list_entries(source_prefix.as_deref()) {
        let Ok(logical_path) = mount.to_logical_path(&source_path) else {
          continue;
        };

        let Some(asset) = Self::locate_at(mount, &logical_path, &source_path) else {
          continue;
        };

        visit(mount, &source_path, asset, None);

        for copy in hidden.get(source_path.as_str()).into_iter().flatten() {
          // Re-addressed under this mount's base: a source names its copies in its own terms, and a listing answers
          // in the world's.
          let asset: XrayAsset = XrayAsset::new(
            XrayLogicalPath::from_normalized(logical_path.to_string()),
            copy.asset.get_container().clone(),
          );

          visit(mount, &source_path, asset, Some(copy.size));
        }
      }
    }
  }

  /// Copies a source holds behind the ones it answers with, keyed by the path they would have answered for.
  fn group_hidden(source: &dyn XrayAssetSource) -> HashMap<&str, Vec<&XrayShadowedCopy>> {
    let mut hidden: HashMap<&str, Vec<&XrayShadowedCopy>> = HashMap::new();

    for copy in source.list_shadowed() {
      hidden
        .entry(copy.asset.get_logical_path().as_str())
        .or_default()
        .push(copy);
    }

    hidden
  }

  /// Returns the winning entry for every reachable path with its size, ordered by logical path.
  pub fn list_mounted_entries(&self) -> Vec<XrayMountedEntry> {
    self.list_mounted_entries_in(&XrayLookupScope::default())
  }

  pub(crate) fn list_mounted_entries_in(&self, scope: &XrayLookupScope) -> Vec<XrayMountedEntry> {
    let mut entries: Vec<XrayMountedEntry> = Vec::new();

    for (asset, size) in self.locate_sized_in(scope) {
      match entries.last() {
        // The copy behind a winner is dropped here rather than collected: the priority rule has already answered, and
        // nothing in this shape reports what it answered against.
        Some(previous) if previous.get_logical_path() == asset.get_logical_path().as_str() => {}
        _ => entries.push(XrayMountedEntry::new(asset, size)),
      }
    }

    entries
  }

  /// Returns winning entries with the sized copies each one hides, ordered by logical path.
  pub fn list_shadowing_entries(&self) -> Vec<XrayShadowingEntry> {
    self.list_shadowing_entries_in(&XrayLookupScope::default())
  }

  pub(crate) fn list_shadowing_entries_in(&self, scope: &XrayLookupScope) -> Vec<XrayShadowingEntry> {
    let mut entries: Vec<XrayShadowingEntry> = Vec::new();

    for (asset, size) in self.locate_sized_in(scope) {
      match entries.last_mut() {
        Some(previous) if previous.get_logical_path() == asset.get_logical_path().as_str() => {
          previous.shadowed.push(XrayShadowedCopy::new(asset, size))
        }
        _ => entries.push(XrayShadowingEntry::new(asset, size)),
      }
    }

    entries
  }

  /// Every copy every mount in scope holds, with its own size, ordered by logical path and then by mount priority.
  fn locate_sized_in(&self, scope: &XrayLookupScope) -> Vec<(XrayAsset, u64)> {
    let mut located: Vec<(XrayAsset, u64)> = Vec::new();

    self.for_each_copy(scope, |mount, source_path, asset, recorded| {
      let size: u64 = recorded.unwrap_or_else(|| mount.get_source().get_size(source_path).unwrap_or_default());

      located.push((asset, size));
    });

    // Stable, so copies of one logical path keep the mount order they were enumerated in and the winner stays first.
    located.sort_by(|first, second| first.0.get_logical_path().cmp(second.0.get_logical_path()));

    located
  }

  /// Writes bytes to the winning entry within a scope.
  pub fn write(&self, scope: &XrayLookupScope, logical_path: &str, bytes: &[u8]) -> XrfResult<()> {
    let logical_path: Cow<str> = normalize(logical_path)?;

    let Some((mount, source_path)) = self.get_winner_in_scope(scope, &logical_path) else {
      return Err(XrfError::new_asset_error(format!(
        "cannot write '{logical_path}': no mount in scope holds it"
      )));
    };

    if !mount.is_writable() {
      return Err(XrfError::new_asset_error(format!(
        "cannot write '{logical_path}': it is held by {} '{}', which is read only",
        match mount.get_kind() {
          XraySourceKind::Archive => "archive",
          XraySourceKind::Directory => "directory",
        },
        mount.get_label()
      )));
    }

    mount.get_source().write(source_path, bytes)?;

    // Whatever was parsed from the old bytes now describes a file that no longer exists in that form. Dropped for every
    // type and scope holding the path rather than for the ones that resolved to this mount: deciding which those were
    // needs a resolve per scope, where a write is rare and dropping is cheap.
    self.cache.forget(&logical_path);

    Ok(())
  }

  /// Creates a loose override in the highest-priority writable mount in scope.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is invalid or out of scope, no writable mount can contain it, the target is already
  /// indexed there, creation or remounting fails, or the new entry does not resolve.
  pub fn write_override(&mut self, scope: &XrayLookupScope, logical_path: &str, bytes: &[u8]) -> XrfResult<XrayAsset> {
    let logical_path: Cow<str> = normalize(logical_path)?;

    if !Self::get_within_prefix(scope, &logical_path) {
      return Err(XrfError::new_asset_error(format!(
        "cannot override '{logical_path}': it falls outside the scope's subtree"
      )));
    }

    let Some((id, source_path)) = self
      .mounts_in(scope)
      .find(|mount| mount.is_writable())
      .map(|mount| (mount.get_id(), mount.to_source_path(&logical_path)))
    else {
      return Err(XrfError::new_asset_error(format!(
        "cannot override '{logical_path}': no writable mount is in scope"
      )));
    };

    let Some(source_path) = source_path else {
      return Err(XrfError::new_asset_error(format!(
        "cannot override '{logical_path}': it falls outside the writable mount's base"
      )));
    };

    self.mounts[id.0].get_source().create(source_path, bytes)?;
    self.remount(id)?;

    self.find_in(scope, &logical_path)?.ok_or_else(|| {
      XrfError::new_asset_error(format!("override '{logical_path}' did not resolve after being written"))
    })
  }

  /// Reindexes a directory mount so newly created files resolve.
  ///
  /// # Errors
  ///
  /// Returns an error when the mount does not exist or its root can no longer be indexed.
  pub fn remount(&mut self, id: XrayMountId) -> XrfResult<()> {
    // Reindexing changes which paths a mount answers for, so retained values may describe entries it no longer wins.
    // This is also the choke point `write_override` passes through, which is why that path needs no hook of its own.
    self.cache.clear();

    let Some(mount) = self.mounts.get(id.0) else {
      return Err(XrfError::new_asset_error(format!("no mount {id:?} to remount")));
    };

    if mount.get_kind() != XraySourceKind::Directory {
      return Ok(());
    }

    let base: String = mount.get_base().to_string();
    let origin: Option<String> = mount.get_origin().map(str::to_string);
    let root: PathBuf = mount.get_source().get_root_path().to_path_buf();

    self.mounts[id.0] = XrayMount::new(id, &base, Box::new(XrayDirectorySource::read(&root)?))?;

    // Reindexing changes what the mount holds and nothing about why it is there, so the plan's name for it survives.
    if let Some(origin) = origin {
      self.record_origin(id, &origin);
    }

    Ok(())
  }

  /// Resolves a raw engine reference of one kind, under that kind's directory and extension.
  ///
  /// # Errors
  ///
  /// Returns an error when `asset_type` has no canonical home, or when the reference cannot be normalized as an X-Ray path.
  pub fn resolve(&self, asset_type: XrayAssetType, reference: &str) -> XrfResult<Option<XrayAsset>> {
    self.resolve_in(&XrayLookupScope::default(), asset_type, reference)
  }

  pub(crate) fn resolve_in(
    &self,
    scope: &XrayLookupScope,
    asset_type: XrayAssetType,
    reference: &str,
  ) -> XrfResult<Option<XrayAsset>> {
    let rules: XrayAssetRules = Self::get_rules_of(asset_type)?;

    self.find_under(scope, rules.directory, &rules.to_logical_path(reference))
  }

  /// Resolves every asset of one kind a reference names, which may be a `*` mask.
  ///
  /// # Errors
  ///
  /// Returns an error when the kind has no canonical home, the reference is not a valid X-Ray path, or a mask carries more
  /// than one `*`.
  pub fn resolve_all(&self, asset_type: XrayAssetType, reference: &str) -> XrfResult<Vec<XrayAsset>> {
    self.resolve_all_in(&XrayLookupScope::default(), asset_type, reference)
  }

  pub(crate) fn resolve_all_in(
    &self,
    scope: &XrayLookupScope,
    asset_type: XrayAssetType,
    reference: &str,
  ) -> XrfResult<Vec<XrayAsset>> {
    if !reference.contains('*') {
      return Ok(self.resolve_in(scope, asset_type, reference)?.into_iter().collect());
    }

    let rules: XrayAssetRules = Self::get_rules_of(asset_type)?;

    let mask: String = crate::path::join(rules.directory, &rules.to_logical_path(reference))?;
    let Some((start, end)) = mask.split_once('*') else {
      return Ok(Vec::new());
    };

    if end.contains('*') {
      return Err(XrfError::new_asset_error(
        "X-Ray asset mask must contain exactly one '*'",
      ));
    }

    Ok(
      self
        .list_entries_in(&scope.clone().with_prefix(rules.directory)?)
        .into_iter()
        .filter(|entry| {
          let path: &str = entry.get_logical_path().as_str();

          path.starts_with(start) && path.ends_with(end)
        })
        .collect(),
    )
  }

  /// Resolves a texture reference, appending `.dds` or replacing its authoring extension.
  ///
  /// # Errors
  ///
  /// Returns an error when the reference cannot be normalized as an X-Ray path.
  pub fn resolve_dds_texture(&self, reference: &str) -> XrfResult<Option<XrayAsset>> {
    self.resolve(XrayAssetType::Dds, reference)
  }

  pub(crate) fn resolve_dds_texture_in(
    &self,
    scope: &XrayLookupScope,
    reference: &str,
  ) -> XrfResult<Option<XrayAsset>> {
    self.resolve_in(scope, XrayAssetType::Dds, reference)
  }

  /// Resolves an OGF reference under the `meshes` namespace.
  ///
  /// # Errors
  ///
  /// Returns an error when the reference cannot be normalized as an X-Ray path.
  pub fn resolve_ogf(&self, reference: &str) -> XrfResult<Option<XrayAsset>> {
    self.resolve(XrayAssetType::Ogf, reference)
  }

  pub(crate) fn resolve_ogf_in(&self, scope: &XrayLookupScope, reference: &str) -> XrfResult<Option<XrayAsset>> {
    self.resolve_in(scope, XrayAssetType::Ogf, reference)
  }

  /// Resolves an OMF reference under the `meshes` namespace.
  ///
  /// # Errors
  ///
  /// Returns an error when the reference cannot be normalized as an X-Ray path.
  pub fn resolve_omf(&self, reference: &str) -> XrfResult<Option<XrayAsset>> {
    self.resolve(XrayAssetType::Omf, reference)
  }

  pub(crate) fn resolve_omf_in(&self, scope: &XrayLookupScope, reference: &str) -> XrfResult<Option<XrayAsset>> {
    self.resolve_in(scope, XrayAssetType::Omf, reference)
  }

  fn find_under(&self, scope: &XrayLookupScope, prefix: &str, path: &str) -> XrfResult<Option<XrayAsset>> {
    self.find_in(scope, &crate::path::join(prefix, path)?)
  }

  /// The resolution rules of a kind that has a canonical home.
  ///
  /// # Errors
  ///
  /// Returns an error naming the kind when it has no single directory to resolve under.
  fn get_rules_of(asset_type: XrayAssetType) -> XrfResult<XrayAssetRules> {
    asset_type.get_rules().ok_or_else(|| {
      XrfError::new_asset_error(format!(
        "asset kind {asset_type:?} has no single directory to resolve under"
      ))
    })
  }

  /// Checks whether a logical path falls inside the scope's subtree.
  fn get_within_prefix(scope: &XrayLookupScope, logical_path: &str) -> bool {
    scope
      .get_prefix()
      .is_none_or(|prefix| crate::path::is_component_prefix(logical_path, prefix))
  }

  /// The highest-priority mount in scope holding a logical path, with that path in the mount's own namespace.
  ///
  /// `logical_path` must already be normalized.
  fn get_winner_in_scope<'a>(&self, scope: &XrayLookupScope, logical_path: &'a str) -> Option<(&XrayMount, &'a str)> {
    if !Self::get_within_prefix(scope, logical_path) {
      return None;
    }

    self.mounts_in(scope).find_map(|mount| {
      mount
        .to_source_path(logical_path)
        .filter(|source_path| mount.get_source().contains(source_path))
        .map(|source_path| (mount, source_path))
    })
  }

  /// Pairs a logical path with the physical container reported by the mount's source.
  fn locate_in(mount: &XrayMount, logical_path: &str) -> Option<XrayAsset> {
    let source_path: &str = mount.to_source_path(logical_path)?;

    Self::locate_at(mount, logical_path, source_path)
  }

  /// Pairs a logical path with its container, for a mount already known to hold the source path.
  fn locate_at(mount: &XrayMount, logical_path: &str, source_path: &str) -> Option<XrayAsset> {
    let container: XrayAssetContainer = mount.get_source().locate(source_path)?;

    Some(XrayAsset::new(
      XrayLogicalPath::from_normalized(logical_path.to_string()),
      container,
    ))
  }
}
