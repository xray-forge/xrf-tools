use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::cache::{XrayAssetCache, XrayCachePolicy};
use crate::path::{XrayLogicalPath, normalize};
use crate::source::XrayDirectorySource;
use crate::trace::XrayReadTrace;
use crate::vfs::XrayDirectoryListing;
use crate::{
  XrayAsset, XrayAssetContainer, XrayAssetRules, XrayAssetSource, XrayAssetType, XrayLookupScope, XrayMount,
  XrayMountId, XrayPathCollision, XraySkippedMount, XraySourceKind,
};

/// The engine's view of assets: several mounted sources, searched in order, first hit wins.
///
/// Open one with [`XrayVfs::open`], then read and resolve directly — every lookup spans all mounts. Narrow a lookup by
/// applying an [`XrayLookupScope`] once through [`XrayVfs::scoped`] rather than threading it into each call.
///
/// Mount higher-priority sources first. This produces the same winner as `CLocatorAPI` when callers reverse the engine's
/// last-registration-wins order, while retaining shadowed entries for inspection.
///
/// Mounting indexes sources eagerly. Duplicate logical paths remain errors within one source and become ordinary
/// shadowing across mounts.
#[derive(Debug, Default)]
pub struct XrayVfs {
  mounts: Vec<XrayMount>,
  /// Parsed assets this world retains, governed by its own policy and empty unless a caller sets one.
  cache: XrayAssetCache,
  /// Per-path account of what was physically read, absent unless a caller asked to be told.
  trace: Option<XrayReadTrace>,
  skipped: Vec<XraySkippedMount>,
  /// Paths already mounted from a plan, so a later plan naming the same source reuses it.
  ///
  /// Keyed by the planned path rather than the source's root, because a volume set's root is the common parent of its
  /// volumes: two single-volume plans in one directory share a root while naming different sources.
  planned: HashMap<PathBuf, XrayMountId>,
}

impl XrayVfs {
  /// Creates an empty VFS with no searchable mounts.
  pub fn new() -> Self {
    Self::default()
  }

  /// Sets what this world may retain after parsing an asset.
  ///
  /// Named at construction because retention is a property of the session, not of a call site: a verification sweep and
  /// an editing session want opposite answers, and neither should be inferred from whichever consumer reads first.
  pub fn with_cache_policy(mut self, policy: XrayCachePolicy) -> Self {
    self.cache = XrayAssetCache::new(policy);

    self
  }

  /// The parsed assets this world is holding.
  pub fn get_cache(&self) -> &XrayAssetCache {
    &self.cache
  }

  /// Accounts for every physical read this world performs from here on.
  ///
  /// Off by default and deliberately opt-in: the account is a lock taken on the read path, which is where a sweep
  /// spends its time. A run that wants the numbers pays for them; every other run pays a null check.
  pub fn with_read_trace(mut self) -> Self {
    self.trace = Some(XrayReadTrace::default());

    self
  }

  /// What this world has read, or `None` when it was never asked to account for it.
  pub fn get_read_trace(&self) -> Option<&XrayReadTrace> {
    self.trace.as_ref()
  }

  /// Reads through a mount, accounting for the read when this world is tracing.
  ///
  /// The single place bytes leave a source, so both the path-keyed read and the asset-keyed one are counted without
  /// either having to remember to.
  fn read_from_mount(&self, mount: &XrayMount, source_path: &str, logical_path: &str) -> XrfResult<Vec<u8>> {
    let bytes: Vec<u8> = mount.get_source().read(source_path)?;

    if let Some(trace) = &self.trace {
      trace.record(logical_path, bytes.len() as u64);
    }

    Ok(bytes)
  }

  /// Sources a plan named that could not be opened.
  ///
  /// Empty for a VFS assembled by hand. Populated by [`Self::mount_plan`], which is tolerant of a source that fails to
  /// open — so a caller that reports on what this VFS holds must report these too, or a mount that silently vanished
  /// looks like content that is silently missing.
  pub fn get_skipped_mounts(&self) -> &[XraySkippedMount] {
    &self.skipped
  }

  /// Records a source that a plan named but could not open.
  pub(crate) fn record_skipped(&mut self, skipped: XraySkippedMount) {
    self.skipped.push(skipped);
  }

  /// The mount already opened from a planned path, when its kind still matches.
  ///
  /// Opening a source indexes it, so a caller that keeps one VFS across requests — a viewer resolving one model after
  /// another — would otherwise re-walk the same tree and append a duplicate mount every time.
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
  /// The first mount's base and priority are retained when a root is reused.
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
  /// Prefer this over [`Self::read`] whenever a lookup or an enumeration already produced the asset. It reads from the
  /// source that *answered* rather than searching the mounts again, which is both cheaper and more truthful: between a
  /// resolve and a path-keyed read, a remount or a new override can change which mount wins, so the bytes need not be
  /// the ones described by the asset in hand.
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

    // todo: A container names a source by its root, and an archive source's root is the common parent of its volumes,
    //   so two mounts planned from single volumes in one directory are indistinguishable here and the first one wins.
    //   No plan constructor produces that today — `from_fsgame` plans directories — but disambiguating needs the
    //   container to carry the mount, or this find to prefer a root-matching mount that also holds the path.
    let Some(mount) = self
      .mounts
      .iter()
      .find(|mount| mount.get_source().get_root_path() == container_root)
    else {
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
  /// For a size gate that exists to avoid parsing a truncated asset: reading the bytes to measure them would defeat it, and
  /// for an archived entry would decompress the whole thing.
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

  /// Returns winning entries, one per logical path, ordered by that path.
  ///
  /// Sorted here rather than left to callers: an archive source keys its name table by hash, so enumeration order is
  /// otherwise arbitrary and unstable between runs. Every consumer that shows or diffs a listing needs a deterministic
  /// order, and two shipped defects came from a caller forgetting to impose one. Sorting ~47,000 entries costs
  /// milliseconds against the enumeration itself.
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
  ///
  /// Not narrowed to the kind's own directory. That directory is where a *reference* resolves, not where
  /// every instance lives: a level ships its own `.dds` files under `levels\<name>\`, and narrowing would drop them from any
  /// enumeration that means "every texture in this project".
  ///
  /// Narrow with a scope prefix when a caller wants one subtree.
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
  /// For assets named by convention rather than by extension alone — `particles.xr` libraries, a level's `level.spawn` — where
  /// the tail of the path is the identity and no kind describes it. The boundary matters: a suffix of `particles.xr`
  /// names that file anywhere in the tree, and must not also match a neighbour named `old_particles.xr`.
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
  ///
  /// An authoring problem to report rather than a reason to refuse the VFS: nothing here affects what resolves, only what a
  /// person should be told is unreachable.
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
  /// Separate from [`Self::list_entries`], which answers everything *below* a prefix: listing `textures` with a prefix scope
  /// yields every texture in the tree, while this yields its handful of folders and files. That is the difference between
  /// expanding one node and loading the whole tree.
  ///
  /// Directories are not entries — a volume records them, and treating them as assets inflates every count — so folder
  /// names are derived from the path segments of entries. Cost is therefore proportional to the entries under `directory`,
  /// not to the number of children returned.
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
  /// `None` means the directory falls outside the view's subtree, which lists nothing. Replacing the scope's prefix
  /// instead of intersecting it would let a view narrowed to `configs` list the children of `textures`, which is the
  /// reach past its own subtree that every other read-path operation refuses.
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

    for mount in self.mounts_in(scope) {
      let Some(source_prefix) = mount.to_source_prefix(scope.get_prefix()) else {
        continue;
      };

      for source_path in mount.get_source().list_entries(source_prefix.as_deref()) {
        if let Ok(logical_path) = mount.to_logical_path(&source_path)
          && let Some(location) = Self::locate_in(mount, &logical_path)
        {
          located.push(location);
        }
      }
    }

    // Stable, so copies of one logical path keep the mount order they were enumerated in.
    located.sort_by(|first, second| first.get_logical_path().cmp(second.get_logical_path()));

    located
  }

  /// Writes bytes to the winning entry within a scope.
  ///
  /// A write names its scope explicitly where lookups default to everything: changing bytes deserves the ceremony, and the
  /// scope is how a caller states which mounts may take the write. Pass [`XrayLookupScope::all()`] to mean the whole VFS.
  ///
  /// The operation refuses read-only winners and absent paths instead of creating a loose override.
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
  /// Unlike [`Self::write`], this creates a new entry instead of modifying the current winner. The mount is rebuilt so the
  /// override resolves immediately. This is how an archived asset is changed: a volume never takes a write, so the
  /// override shadows it from a loose mount in front.
  ///
  /// ```rust,no_run
  /// use xrf_vfs::{XrayLookupScope, XrayMountMode, XrayVfs};
  ///
  /// # fn main() -> xrf_error::XrfResult {
  /// let mut vfs: XrayVfs = XrayVfs::open(XrayMountMode::Installation, "C:\\Games\\Anomaly")?;
  ///
  /// // write() refuses an archive winner; the override lands in gamedata and shadows it.
  /// let overridden = vfs.write_override(&XrayLookupScope::all(), "configs\\my_tweak.ltx", b"[tweak]")?;
  ///
  /// assert!(overridden.to_physical_path().is_some(), "an override is always a loose file");
  /// # Ok(())
  /// # }
  /// ```
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
  /// Non-directory mounts are left unchanged.
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
    let root: PathBuf = mount.get_source().get_root_path().to_path_buf();

    self.mounts[id.0] = XrayMount::new(id, &base, Box::new(XrayDirectorySource::read(&root)?))?;

    Ok(())
  }

  /// Resolves a raw engine reference of one kind, under that kind's directory and extension.
  ///
  /// This is how an editor resolves any kind the table knows without the VFS growing a method per kind. `reference` is
  /// untrusted engine text — from a config field or a mesh header — so normalizing it is this call's job, which is why it
  /// takes `&str` rather than an [`crate::XrayLogicalPath`].
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
  /// A motion reference is allowed to name a set — `wpn\wpn_ak74_*.omf` means every matching animation file — so this
  /// answers a list where [`Self::resolve`] answers at most one. A reference without `*` resolves to a single asset or none,
  /// which is why this is not two separate calls at the consumer.
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
  /// The one place a path-keyed operation picks a winner. [`Self::find`], [`Self::read`], [`Self::read_size`] and
  /// [`Self::write`] each used to walk the mounts themselves and apply the scope's subtree guard inline — four copies of
  /// one decision, two of which asked `locate` where the others asked `contains`. A source answering those two
  /// differently would have sent a read to a mount the preceding lookup did not choose;
  /// [`XrayAssetSource::contains`] now derives from `locate` by default so it cannot.
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
