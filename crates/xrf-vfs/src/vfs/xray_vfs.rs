use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::cache::{XrayAssetCache, XrayCacheKey, XrayCachePolicy, XrayCacheStats};
use crate::input::{XrayRevision, XrayRevisions};
use crate::path::{XrayLogicalPath, normalize};
use crate::search::{XrayDirectoryListing, XrayMountSelection, XrayMountedEntry, XrayShadowedCopy, XrayShadowingEntry};
use crate::source::XrayDirectorySource;
use crate::trace::XrayReadTrace;
use crate::{
  XrayAsset, XrayAssetContainer, XrayAssetRules, XrayAssetSource, XrayAssetType, XrayFileStamp, XrayLookupScope,
  XrayMount, XrayMountId, XraySkippedMount, XraySourceKind,
};

/// A VFS snapshot: the engine's view of assets as several mounted sources in priority order, with the revisions of
/// what they hold and what the snapshot retains after parsing.
///
/// Read only through an [`XraySearch`](crate::XraySearch), which names the scopes it searches and records what it
/// reads; the snapshot itself answers for its mounts, revisions and caches. A snapshot pins its index and revisions,
/// not the bytes of loose files, which it reads from disk as they are when read.
#[derive(Debug, Default)]
pub struct XrayVfs {
  mounts: Vec<XrayMount>,
  /// Parsed assets this snapshot and every fork of it retain, keyed by the copy each was parsed from, so a mount or a
  /// fork keeps what is still true; governed by one policy per lineage and empty unless a caller sets one.
  cache: Arc<XrayAssetCache>,
  /// Per-path account of what was physically read, absent unless a caller asked to be told; one account however many
  /// forks read.
  trace: Option<Arc<XrayReadTrace>>,
  skipped: Vec<XraySkippedMount>,
  /// Where in `skipped` each planned source that failed to open sits, by its path and then its kind, so a failure is
  /// settled rather than retried and a lookup borrows the path it is asked about.
  failed: HashMap<PathBuf, Vec<(XraySourceKind, usize)>>,
  /// Paths already mounted from a plan, so a later plan naming the same source reuses it.
  planned: HashMap<PathBuf, XrayMountId>,
  /// The revision of the last change this snapshot holds.
  revision: XrayRevision,
  /// When each loose file and indexed directory last changed, where one did.
  revisions: XrayRevisions,
}

impl XrayVfs {
  /// Creates an empty VFS with no searchable mounts.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets what this snapshot may retain after parsing an asset, starting a cache of its own that its forks share.
  pub fn with_cache_policy(mut self, policy: XrayCachePolicy) -> Self {
    self.cache = Arc::new(XrayAssetCache::new(policy));

    self
  }

  /// What this snapshot may retain after parsing an asset.
  pub fn get_cache_policy(&self) -> &XrayCachePolicy {
    self.cache.get_policy()
  }

  /// How well retention has served the reads of this snapshot and its lineage so far.
  pub fn get_cache_stats(&self) -> XrayCacheStats {
    self.cache.get_stats()
  }

  pub(crate) fn get_cache(&self) -> &XrayAssetCache {
    &self.cache
  }

  /// Accounts for every physical read this snapshot performs from here on.
  pub fn with_read_trace(mut self) -> Self {
    self.trace = Some(Arc::default());

    self
  }

  /// What this snapshot has read, or `None` when it was never asked to account for it.
  pub fn get_read_trace(&self) -> Option<&XrayReadTrace> {
    self.trace.as_deref()
  }

  /// A snapshot over the same mounts, sharing each source, the index read for it and the parsed assets retained rather
  /// than reading them again, to mount more into while this one goes on serving reads.
  pub fn fork(&self) -> Self {
    Self {
      cache: Arc::clone(&self.cache),
      failed: self.failed.clone(),
      mounts: self.mounts.clone(),
      planned: self.planned.clone(),
      revision: self.revision,
      revisions: self.revisions.clone(),
      skipped: self.skipped.clone(),
      trace: self.trace.clone(),
    }
  }

  /// The project revision this snapshot is a snapshot of: that of the last change it holds, a number its lineage gives
  /// one change alone and that grows with every change.
  pub fn get_revision(&self) -> XrayRevision {
    self.revision
  }

  pub(crate) fn get_revisions(&self) -> &XrayRevisions {
    &self.revisions
  }

  pub(crate) fn get_revisions_mut(&mut self) -> &mut XrayRevisions {
    &mut self.revisions
  }

  pub(crate) fn set_revision(&mut self, revision: XrayRevision) {
    self.revision = revision;
  }

  /// Replaces what a mount reads with a changed copy of its source, keeping its opening, so what is retained of the
  /// copies that did not change goes on being served.
  pub(crate) fn replace_source(&mut self, id: XrayMountId, source: Box<dyn XrayAssetSource>) {
    if let Some(mount) = self.mounts.get_mut(id.0) {
      *mount = mount.with_source(source);
    }
  }

  /// Drops what the lineage retains of a copy whose bytes changed: nothing reads it at its old revision once the
  /// change publishes, and it would otherwise hold the budget until nobody held it.
  pub(crate) fn forget_retained(&self, mount: &XrayMount, source_path: &str) {
    self.cache.forget_copy(mount.get_opening(), source_path);
  }

  /// Moves a mount's copy of a logical path to `revision`, for a test standing in for a commit.
  #[cfg(test)]
  pub(crate) fn set_file_revision(&mut self, mount: XrayMountId, logical_path: &str, revision: XrayRevision) {
    self.revisions.set_file(mount, logical_path, revision);
  }

  /// Reads through a mount, accounting for the read when this snapshot is tracing.
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

    // Nothing retained goes stale: a value names the copy it was parsed from, and a path this mount now wins is served
    // from the new copy rather than the old one.
    self.mounts.push(XrayMount::new(id, base, source)?);

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
  fn find_directory_mount(&self, root: &Path) -> Option<XrayMountId> {
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

  /// Iterates over mounts selected by a scope in its priority order: named mounts in the order named, once each, and
  /// every other selection in mount order.
  pub(crate) fn mounts_in<'a, 's>(
    &'a self,
    scope: &'s XrayLookupScope,
  ) -> impl Iterator<Item = &'a XrayMount> + use<'a, 's> {
    let named: Option<&[XrayMountId]> = match scope.get_selection() {
      XrayMountSelection::Only(ids) => Some(ids),
      _ => None,
    };
    let in_named_order = named.into_iter().flat_map(move |ids| {
      ids
        .iter()
        .enumerate()
        .filter(move |(index, id)| !ids[..*index].contains(id))
        .filter_map(|(_, id)| self.mounts.get(id.0))
    });
    let in_mount_order = named
      .is_none()
      .then_some(&self.mounts)
      .into_iter()
      .flatten()
      .filter(move |mount| scope.includes(mount));

    in_named_order.chain(in_mount_order)
  }

  /// The winning location for a logical path within a scope, or `None` when no mount in it holds the path.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not a valid X-Ray logical path. Absence is `Ok(None)`, not an error.
  pub(crate) fn find_in(&self, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Option<XrayAsset>> {
    let logical_path: Cow<str> = normalize(logical_path)?;

    Ok(
      self
        .get_winner_in_scope(scope, &logical_path)
        .and_then(|(mount, source_path)| Self::locate_at(mount, &logical_path, source_path)),
    )
  }

  /// Reads bytes from the winning entry for a logical path within a scope.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when nothing in scope holds the path, an invalid-path error when it is not a valid
  /// X-Ray logical path, or the source's own error when the bytes cannot be read.
  #[cfg(test)]
  pub(crate) fn read_in(&self, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Vec<u8>> {
    let logical_path: Cow<str> = normalize(logical_path)?;
    let (mount, source_path) = self.require_winner_in_scope(scope, &logical_path)?;

    self.read_from_mount(mount, source_path, &logical_path)
  }

  /// The highest-priority mount in scope holding a normalized logical path, as [`Self::get_winner_in_scope`] finds it.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when nothing in scope holds the path. Absence is `NotFound` throughout this crate, so a
  /// consumer can tell "the asset is not here" from "the source holding it failed" without reading the message.
  #[cfg(test)]
  fn require_winner_in_scope<'a>(
    &self,
    scope: &XrayLookupScope,
    logical_path: &'a str,
  ) -> XrfResult<(&XrayMount, &'a str)> {
    self.get_winner_in_scope(scope, logical_path).ok_or_else(|| {
      XrfError::new_not_found_error(format!(
        "no asset '{logical_path}' in scope across {} mount(s)",
        self.mounts_in(scope).count()
      ))
    })
  }

  /// Reads and parses the winning entry for a logical path within a scope, serving a retained value when this snapshot's
  /// lineage already holds one for that copy at its revision.
  ///
  /// # Errors
  ///
  /// Returns whatever reading the asset or parsing it answers with. Failures are not retained, so a broken asset is
  /// re-read by each caller and each reports the real error rather than a copy of the first one.
  #[cfg(test)]
  pub(crate) fn read_parsed_in<T, F>(
    &self,
    scope: &XrayLookupScope,
    kind: XrayAssetType,
    logical_path: &str,
    parse: F,
  ) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce(Vec<u8>) -> XrfResult<T>,
  {
    let logical_path: Cow<str> = normalize(logical_path)?;
    let (mount, source_path) = self.require_winner_in_scope(scope, &logical_path)?;

    self.read_parsed_at(mount, source_path, &logical_path, kind, parse)
  }

  /// Reads and parses the copy an asset this VFS already resolved names, serving what the lineage retains of it at its
  /// revision, so a parse and a raw read of one asset always read the same copy.
  ///
  /// # Errors
  ///
  /// As [`Self::read_asset_bytes`], or whatever parsing answers; failures are not retained.
  pub(crate) fn read_parsed_asset<T, F>(&self, asset: &XrayAsset, kind: XrayAssetType, parse: F) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce(Vec<u8>) -> XrfResult<T>,
  {
    let (mount, source_path) = self.locate_copy(asset)?;

    self.read_parsed_at(mount, source_path, asset.get_logical_path().as_str(), kind, parse)
  }

  /// Parses one mount's copy of a normalized logical path, serving what the lineage retains of it at its revision.
  fn read_parsed_at<T, F>(
    &self,
    mount: &XrayMount,
    source_path: &str,
    logical_path: &str,
    kind: XrayAssetType,
    parse: F,
  ) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce(Vec<u8>) -> XrfResult<T>,
  {
    let key: XrayCacheKey = XrayCacheKey::of::<T>(
      mount.get_opening(),
      source_path,
      self.revisions.get_file(mount.get_id(), logical_path),
    );

    self.cache.get_or_load(key, kind, || {
      // Looked at before the read, so a change landing between the two is seen by the next look rather than missed.
      let stamp: Option<XrayFileStamp> = self
        .cache
        .get_policy()
        .is_allowed(kind)
        .then(|| mount.get_source().get_stamp(source_path))
        .flatten();
      let bytes: Vec<u8> = self.read_from_mount(mount, source_path, logical_path)?;
      let length: u64 = bytes.len() as u64;

      Ok((parse(bytes)?, length, stamp))
    })
  }

  /// Reads the bytes of the copy an asset this VFS already resolved names.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when no mount in this VFS holds the asset's container — most often because the asset came
  /// from a different VFS, or its mount has since been replaced.
  pub(crate) fn read_asset_bytes(&self, asset: &XrayAsset) -> XrfResult<Vec<u8>> {
    let (mount, source_path) = self.locate_copy(asset)?;

    self.read_from_mount(mount, source_path, asset.get_logical_path().as_str())
  }

  /// The mount holding the copy an asset names, and its path there.
  ///
  /// # Errors
  ///
  /// As [`Self::read_asset_bytes`].
  fn locate_copy<'a>(&self, asset: &'a XrayAsset) -> XrfResult<(&XrayMount, &'a str)> {
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

    Ok((mount, source_path))
  }

  /// Size in bytes of the winning entry within a scope, without reading it.
  pub(crate) fn read_size_in(&self, scope: &XrayLookupScope, logical_path: &str) -> Option<u64> {
    let logical_path: Cow<str> = normalize(logical_path).ok()?;
    let (mount, source_path) = self.get_winner_in_scope(scope, &logical_path)?;

    mount.get_source().get_size(source_path)
  }

  /// CRC32 of the winning entry's payload within a scope, when its source already knows it without reading anything.
  pub(crate) fn read_recorded_crc_in(&self, scope: &XrayLookupScope, logical_path: &str) -> Option<u32> {
    let logical_path: Cow<str> = normalize(logical_path).ok()?;
    let (mount, source_path) = self.get_winner_in_scope(scope, &logical_path)?;

    mount.get_source().get_recorded_crc(source_path)
  }

  /// Winning entries within a scope, one per logical path, ordered by that path.
  pub(crate) fn list_entries_in(&self, scope: &XrayLookupScope) -> Vec<XrayAsset> {
    let mut located: Vec<XrayAsset> = self.list_entries_all_in(scope);

    // `list_entries_all_in` sorts stably by logical path after collecting in mount order, so within one path the
    // highest-priority mount comes first — which is exactly the entry `dedup_by` keeps. Deduping the sort we already
    // need costs nothing, where a seen-set cost a hash and an owned copy of every path enumerated.
    located.dedup_by(|first, second| first.get_logical_path() == second.get_logical_path());

    located
  }

  /// Winning entries within a scope whose extension identifies one kind.
  pub(crate) fn list_entries_of_type_in(&self, scope: &XrayLookupScope, asset_type: XrayAssetType) -> Vec<XrayAsset> {
    self
      .list_entries_in(scope)
      .into_iter()
      .filter(|entry| entry.is_type(asset_type))
      .collect()
  }

  /// What sits directly inside one logical directory within a scope, as a browser or a tree view needs it.
  ///
  /// # Errors
  ///
  /// Returns an error when `directory` is not a valid X-Ray logical path. An empty `directory` lists the logical root.
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

  /// Every entry within a scope, shadowed copies included, ordered by logical path.
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
          // in the snapshot's.
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

  /// The winning entry for every path a scope reaches, with its size, ordered by logical path.
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

  /// Winning entries within a scope with the sized copies each one hides, ordered by logical path.
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

  /// Resolves a raw engine reference of one kind within a scope, under that kind's directory and extension.
  ///
  /// # Errors
  ///
  /// Returns an error when `asset_type` has no canonical home, or when the reference cannot be normalized as an X-Ray path.
  fn resolve_in(
    &self,
    scope: &XrayLookupScope,
    asset_type: XrayAssetType,
    reference: &str,
  ) -> XrfResult<Option<XrayAsset>> {
    let rules: XrayAssetRules = Self::get_rules_of(asset_type)?;

    self.find_in(
      scope,
      &crate::path::join(rules.directory, &rules.to_logical_path(reference))?,
    )
  }

  /// Resolves every asset of one kind a reference names within a scope, which may be a `*` mask.
  ///
  /// # Errors
  ///
  /// Returns an error when the kind has no canonical home, the reference is not a valid X-Ray path, or a mask carries more
  /// than one `*`.
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

  /// The resolution rules of a kind that has a canonical home.
  ///
  /// # Errors
  ///
  /// Returns an error naming the kind when it has no single directory to resolve under.
  pub(crate) fn get_rules_of(asset_type: XrayAssetType) -> XrfResult<XrayAssetRules> {
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
  pub(crate) fn get_winner_in_scope<'a>(
    &self,
    scope: &XrayLookupScope,
    logical_path: &'a str,
  ) -> Option<(&XrayMount, &'a str)> {
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

  /// Pairs a logical path with its container, for a mount already known to hold the source path.
  fn locate_at(mount: &XrayMount, logical_path: &str, source_path: &str) -> Option<XrayAsset> {
    let container: XrayAssetContainer = mount.get_source().locate(source_path)?;

    Some(XrayAsset::new(
      XrayLogicalPath::from_normalized(logical_path.to_string()),
      container,
    ))
  }
}
